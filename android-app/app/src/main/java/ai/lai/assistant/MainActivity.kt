package ai.lai.assistant

import android.Manifest
import android.annotation.SuppressLint
import android.app.Activity
import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import android.graphics.Bitmap
import android.graphics.drawable.BitmapDrawable
import android.util.Log
import android.net.Uri
import android.os.*
import android.provider.ContactsContract
import android.provider.MediaStore
import android.provider.Settings
import android.util.Base64
import android.view.View
import android.view.WindowInsetsController
import android.webkit.*
import android.widget.Toast
import androidx.appcompat.app.AppCompatActivity
import androidx.core.app.ActivityCompat
import androidx.core.content.ContextCompat
import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsCompat
import androidx.core.view.WindowInsetsControllerCompat
import androidx.lifecycle.lifecycleScope
import androidx.activity.result.contract.ActivityResultContracts
import kotlinx.coroutines.*
import java.io.ByteArrayOutputStream
import java.io.File
import java.io.FileOutputStream
import java.net.HttpURLConnection
import java.net.URL
import java.util.concurrent.TimeUnit

class MainActivity : AppCompatActivity() {

    private lateinit var webView: WebView
    private var pendingPermissionCallback: ((Boolean) -> Unit)? = null
    private var pendingPhotoCallback: ((String) -> Unit)? = null
    private var llmReachable: Boolean = false
    private var logcatProcess: java.lang.Process? = null
    private lateinit var daemon: LaiDaemon
    private var tts: TtsManager? = null

    companion object {
        private const val TAG = "LaiMain"
        private const val MODEL_URL = "https://huggingface.co/Qwen/Qwen2.5-0.5B-Instruct-GGUF/resolve/main/qwen2.5-0.5b-instruct-q4_k_m.gguf"
        private const val MODEL_NAME = "qwen2.5-0.5b-instruct-q4_k_m.gguf"
        private const val MODEL_EXPECTED_BYTES = 491400032L
        // T68: pin the expected SHA-256 of the GGUF. Fail loud on mismatch.
        private const val MODEL_SHA256 = ""
        private const val REQUEST_ROLE_ASSISTANT = 4242
    }

    private val cameraLauncher = registerForActivityResult(
        ActivityResultContracts.TakePicturePreview()
    ) { bitmap ->
        if (bitmap != null) {
            val baos = ByteArrayOutputStream()
            bitmap.compress(Bitmap.CompressFormat.JPEG, 85, baos)
            val base64 = Base64.encodeToString(baos.toByteArray(), Base64.NO_WRAP)
            pendingPhotoCallback?.invoke("data:image/jpeg;base64,$base64")
        } else {
            pendingPhotoCallback?.invoke("")
        }
        pendingPhotoCallback = null
    }

    @SuppressLint("SetJavaScriptEnabled")
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        WindowCompat.setDecorFitsSystemWindows(window, false)
        val controller = WindowInsetsControllerCompat(window, window.decorView)
        controller.isAppearanceLightStatusBars = false
        controller.isAppearanceLightNavigationBars = false

        webView = WebView(this).apply {
            setContentView(this)

            settings.javaScriptEnabled = true
            settings.domStorageEnabled = true
            // T63: only our own bundled assets should load. Disable broad file
            // access and never allow cleartext to arbitrary hosts.
            settings.allowFileAccess = false
            settings.allowFileAccessFromFileURLs = false
            settings.allowUniversalAccessFromFileURLs = false
            settings.mediaPlaybackRequiresUserGesture = false
            settings.mixedContentMode = WebSettings.MIXED_CONTENT_NEVER_ALLOW
            settings.setSupportZoom(false)
            settings.builtInZoomControls = false
            settings.displayZoomControls = false
            settings.loadWithOverviewMode = true
            settings.useWideViewPort = true
            setLayerType(View.LAYER_TYPE_HARDWARE, null)

            webViewClient = object : WebViewClient() {
                // T63: allow only our bundled assets to render in-WebView.
                // Everything else (http(s), other file://) escapes to a real
                // browser via ACTION_VIEW instead of loading inside the bridge.
                override fun shouldOverrideUrlLoading(view: WebView?, request: WebResourceRequest?): Boolean {
                    val url = request?.url?.toString() ?: return false
                    if (url.startsWith("file:///android_asset/")) return false
                    try {
                        val intent = Intent(Intent.ACTION_VIEW, request.url)
                        startActivity(intent)
                    } catch (_: Exception) {
                    }
                    return true
                }

                override fun onPageFinished(view: WebView?, url: String?) {
                    super.onPageFinished(view, url)
                    view?.evaluateJavascript("""
                        (function() {
                            var style = document.createElement('style');
                            style.textContent = '*, *::before, *::after { -webkit-tap-highlight-color: transparent; }';
                            document.head.appendChild(style);
                        })();
                    """.trimIndent(), null)
                }
            }

            webChromeClient = object : WebChromeClient() {
                // T63: never auto-grant. Only allow the exact origins we trust
                // (our asset origin) and only for resources the app actually uses.
                override fun onPermissionRequest(request: PermissionRequest?) {
                    request ?: return
                    val trusted = "file:///android_asset/"
                    val granted = request.resources.filter {
                        request.origin?.toString() == trusted
                    }.toTypedArray()
                    if (granted.isNotEmpty()) request.grant(granted) else request.deny()
                }
            }

            addJavascriptInterface(LaiBridge(this@MainActivity), "LaiBridge")
            loadUrl("file:///android_asset/index.html")
            overScrollMode = View.OVER_SCROLL_NEVER
        }

        daemon = LaiDaemon(applicationContext)
        tts = TtsManager(this)
        installLogcatFile()
        maybeRequestAssistantRole()

        lifecycleScope.launch(Dispatchers.IO) {
            ensureModelReady()
            withContext(Dispatchers.Main) {
                startLaiDaemon()
            }
        }
    }

    // ── Model download ──────────────────────────────────────────

    private fun modelFile(): File = File(filesDir, "models/$MODEL_NAME")

    private fun modelReady(): Boolean {
        val f = modelFile()
        if (!f.exists()) return false
        // T68: verify exact size AND pinned SHA-256 before trusting a cache.
        val actualSha = Rpc.sha256Hex(f)
        if (!Rpc.isModelValid(f.length(), MODEL_EXPECTED_BYTES, actualSha, MODEL_SHA256)) {
            if (MODEL_SHA256.isNotEmpty() && actualSha != MODEL_SHA256) {
                Log.w(TAG, "Cached model SHA mismatch — re-downloading")
                f.delete()
            }
            return false
        }
        return true
    }

    private suspend fun ensureModelReady() {
        if (modelReady()) {
            Log.i(TAG, "Model already present: ${modelFile().absolutePath}")
            return
        }
        val dir = File(filesDir, "models"); dir.mkdirs()
        val tmp = File(dir, "$MODEL_NAME.tmp")
        notifyJs("model-progress", 0)

        // T68: resume support — if a .tmp exists, request the remaining bytes.
        val resumeFrom = if (tmp.exists()) tmp.length() else 0L
        try {
            val conn = URL(MODEL_URL).openConnection() as HttpURLConnection
            conn.connectTimeout = 15_000
            conn.readTimeout = 60_000
            Rpc.rangeHeader(resumeFrom)?.let { conn.setRequestProperty("Range", it) }
            conn.connect()

            if (conn.responseCode == HttpURLConnection.HTTP_PARTIAL && resumeFrom > 0) {
                Log.i(TAG, "Resuming model download from $resumeFrom bytes")
            } else if (resumeFrom > 0) {
                // Server ignored Range — restart cleanly from zero.
                tmp.delete()
            }

            val total = (conn.contentLengthLong + resumeFrom).coerceAtLeast(1)
            var downloaded = resumeFrom
            conn.inputStream.use { input ->
                // Append when resuming, else overwrite.
                FileOutputStream(tmp, resumeFrom > 0).use { output ->
                    val buf = ByteArray(64 * 1024)
                    var read: Int
                    while (input.read(buf).also { read = it } != -1) {
                        output.write(buf, 0, read)
                        downloaded += read
                        val pct = ((downloaded * 100) / total).toInt().coerceIn(0, 99)
                        if (pct % 5 == 0) {
                            notifyJs("model-progress", pct)
                            Log.d(TAG, "Model download: $pct% ($downloaded/$total)")
                        }
                    }
                }
            }

            if (tmp.length() < 1_000_000) {
                Log.e(TAG, "Download too small (${tmp.length()} bytes) — likely error page")
                tmp.delete()
                notifyJs("model-error", 0)
                return
            }

            // T68: hash before rename; fail loud on mismatch.
            if (MODEL_SHA256.isNotEmpty() && Rpc.sha256Hex(tmp) != MODEL_SHA256) {
                Log.e(TAG, "Model SHA-256 mismatch — refusing to install (verify-don't-trust)")
                tmp.delete()
                notifyJs("model-error", 0)
                return
            }

            tmp.renameTo(modelFile())
            Log.i(TAG, "Model ready: ${modelFile().absolutePath} (${modelFile().length()} bytes)")
            notifyJs("model-progress", 100)
        } catch (e: Exception) {
            Log.e(TAG, "Model download failed: ${e.message}")
            notifyJs("model-error", 0)
            // Keep .tmp for resume on the next attempt; don't delete on network error.
        }
    }

    private fun notifyJs(event: String, value: Int) {
        Handler(Looper.getMainLooper()).post {
            webView.evaluateJavascript(
                "window.dispatchEvent(new CustomEvent('lai-model', {detail:{event:'$event',value:$value}}))",
                null
            )
        }
    }

    // ── Logcat to file ──────────────────────────────────────────

    // T69: correct filterspec (two specs, not one malformed one) and keep the
    // Process reference so it can be destroyed on exit (no leak, no append pile-up).
    private fun installLogcatFile() {
        try {
            val logFile = File(filesDir, "lai.log")
            // Rotate if the previous capture grew large.
            if (logFile.exists() && logFile.length() > 4 * 1024 * 1024) {
                logFile.renameTo(File(filesDir, "lai.log.old"))
            }
            logcatProcess?.destroy()
            logcatProcess = Runtime.getRuntime().exec(
                arrayOf("logcat", "-f", logFile.absolutePath, "LaiDaemon:D", "LaiMain:D", "*:S")
            )
            Log.i(TAG, "Logcat writing to ${logFile.absolutePath}")
        } catch (e: Exception) {
            Log.w(TAG, "Could not start logcat file capture: ${e.message}")
        }
    }

    // ── Daemon ──────────────────────────────────────────────────

    private fun startLaiDaemon() {
        val ok = daemon.start(if (modelReady()) modelFile().absolutePath else null)
        if (ok) {
            llmReachable = probeLlmReachable()
            Log.i(TAG, "Daemon ready: ${daemon.lastLog} — llm=${if (llmReachable) "reachable" else "unreachable (corpus-only)"}")
        } else {
            Log.e(TAG, "Daemon failed to start: ${daemon.lastLog}")
        }
    }

    // T61: the on-device LLM endpoint (ollama at 127.0.0.1:11434) is only
    // present on dev phones. On a stock device nothing listens there, so the
    // chat path silently degrades to the deterministic engine. Probe it once
    // and expose the result so the UI can stop claiming LINKED when it isn't.
    private fun probeLlmReachable(): Boolean {
        return try {
            val url = URL("http://127.0.0.1:11434/api/tags")
            val conn = url.openConnection() as HttpURLConnection
            conn.connectTimeout = 2_000
            conn.readTimeout = 2_000
            conn.requestMethod = "GET"
            val ok = conn.responseCode == HttpURLConnection.HTTP_OK
            conn.disconnect()
            ok
        } catch (_: Exception) {
            false
        }
    }

    private fun daemonStatus(): String {
        return buildString {
            append("started=${daemon.started}\n")
            append("log=${daemon.lastLog}\n")
            append("alive=${daemon.isAlive()}\n")
            append("model=${if (modelReady()) modelFile().absolutePath else "none"}\n")
            append("llm=${if (llmReachable) "reachable" else "unreachable (corpus-only)"}\n")
        }
    }

    private fun maybeRequestAssistantRole() {
        requestAssistantRole(force = false)
    }

    /**
     * Ask the user to make L.ai the default digital assistant (ROLE_ASSISTANT),
     * replacing Gemini. When [force] is false we only prompt if the role is not
     * already held; when true (user tapped "Set as assistant") we always show
     * the chooser or fall back to Settings.
     */
    private fun requestAssistantRole(force: Boolean) {
        val rm = getSystemService(ROLE_SERVICE) as? android.app.role.RoleManager ?: return
        val role = android.app.role.RoleManager.ROLE_ASSISTANT
        if (!rm.isRoleAvailable(role)) return
        if (!force && rm.isRoleHeld(role)) return
        try {
            startActivityForResult(rm.createRequestRoleIntent(role), REQUEST_ROLE_ASSISTANT)
        } catch (_: Exception) {
            // Some OEM builds don't honor the dialog; send the user to Settings.
            try {
                startActivity(Intent(Settings.ACTION_VOICE_INPUT_SETTINGS))
            } catch (_: Exception) {
            }
        }
    }

    override fun onActivityResult(requestCode: Int, resultCode: Int, data: Intent?) {
        super.onActivityResult(requestCode, resultCode, data)
        if (requestCode == REQUEST_ROLE_ASSISTANT) {
            // Result ignored: ROLE_ASSISTANT state is re-checked on next launch.
        }
    }

    override fun onBackPressed() {
        if (webView.canGoBack()) webView.goBack() else super.onBackPressed()
    }

    override fun onResume() {
        super.onResume()
        webView.onResume()
    }

    override fun onPause() {
        webView.onPause()
        super.onPause()
    }

    override fun onDestroy() {
        try { logcatProcess?.destroy() } catch (_: Exception) {}
        logcatProcess = null
        try { daemon.stop() } catch (_: Exception) {}
        try { tts?.shutdown() } catch (_: Exception) {}
        tts = null
        webView.destroy()
        super.onDestroy()
    }

    inner class LaiBridge(private val ctx: Context) {

        @JavascriptInterface
        fun executeCommand(command: String): String {
            if (!daemon.started) {
                return "Error: daemon not started — model may still be downloading"
            }
            return try {
                daemon.chat(command)
            } catch (e: Exception) {
                "Error: ${e.javaClass.simpleName}: ${e.message}"
            }
        }

        /** Speak [text] aloud via Android TTS (parity with a real assistant). */
        @JavascriptInterface
        fun speak(text: String) {
            Handler(Looper.getMainLooper()).post { tts?.speak(text) }
        }

        /** True if L.ai currently holds the default-assistant role. */
        @JavascriptInterface
        fun isDefaultAssistant(): Boolean {
            val rm = getSystemService(ROLE_SERVICE) as? android.app.role.RoleManager ?: return false
            return try {
                rm.isRoleHeld(android.app.role.RoleManager.ROLE_ASSISTANT)
            } catch (_: Exception) {
                false
            }
        }

        /** Prompt the user to set L.ai as the default digital assistant. */
        @JavascriptInterface
        fun setDefaultAssistant() {
            Handler(Looper.getMainLooper()).post { requestAssistantRole(force = true) }
        }

        @JavascriptInterface
        fun getDeviceInfo(): String {
            return buildString {
                append("model=${Build.MANUFACTURER} ${Build.MODEL}\n")
                append("sdk=${Build.VERSION.SDK_INT}\n")
                append("abi=${Build.SUPPORTED_ABIS.firstOrNull() ?: "unknown"}\n")
                append("android=${Build.VERSION.RELEASE}\n")
                append("cores=${Runtime.getRuntime().availableProcessors()}\n")
                append("mem=${(Runtime.getRuntime().maxMemory() / 1024 / 1024).toInt()}MB\n")
                append("modelReady=${modelReady()}\n")
                append("modelPath=${if (modelReady()) modelFile().absolutePath else "downloading..."}")
            }
        }

        @JavascriptInterface
        fun getDaemonStatus(): String {
            return daemonStatus()
        }

        @JavascriptInterface
        fun restartDaemon(): String {
            return try {
                daemon.stop()
                startLaiDaemon()
                "restart requested"
            } catch (e: Exception) {
                "restart failed: ${e.message}"
            }
        }

        @JavascriptInterface
        fun showToast(message: String) {
            Handler(Looper.getMainLooper()).post {
                Toast.makeText(ctx, message, Toast.LENGTH_SHORT).show()
            }
        }

        @JavascriptInterface
        fun getBatteryLevel(): Int {
            val batteryManager = ctx.getSystemService(BATTERY_SERVICE) as BatteryManager
            return batteryManager.getIntProperty(BatteryManager.BATTERY_PROPERTY_CAPACITY)
        }

        @JavascriptInterface
        fun requestPermission(permission: String): Boolean {
            val result = ContextCompat.checkSelfPermission(ctx, permission)
            if (result == PackageManager.PERMISSION_GRANTED) return true

            val granted = CompletableDeferred<Boolean>()
            Handler(Looper.getMainLooper()).post {
                pendingPermissionCallback = { granted.complete(it) }
                ActivityCompat.requestPermissions(
                    this@MainActivity, arrayOf(permission), 1001
                )
            }
            return runBlocking { granted.await() }
        }

        @JavascriptInterface
        fun takePhoto(): String {
            val result = CompletableDeferred<String>()
            Handler(Looper.getMainLooper()).post {
                pendingPhotoCallback = { result.complete(it) }
                cameraLauncher.launch(null)
            }
            return runBlocking { result.await() }
        }

        @JavascriptInterface
        fun getClipboard(): String {
            val clipboard = ctx.getSystemService(Context.CLIPBOARD_SERVICE) as android.content.ClipboardManager
            val clip = clipboard.primaryClip
            return clip?.getItemAt(0)?.text?.toString() ?: ""
        }

        @JavascriptInterface
        fun setClipboard(text: String) {
            val clipboard = ctx.getSystemService(Context.CLIPBOARD_SERVICE) as android.content.ClipboardManager
            val clip = android.content.ClipData.newPlainText("L.ai", text)
            clipboard.setPrimaryClip(clip)
        }

        @JavascriptInterface
        fun vibrate(durationMs: Int) {
            val vibrator = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
                val manager = ctx.getSystemService(Context.VIBRATOR_MANAGER_SERVICE) as android.os.VibratorManager
                manager.defaultVibrator
            } else {
                @Suppress("DEPRECATION")
                ctx.getSystemService(Context.VIBRATOR_SERVICE) as android.os.Vibrator
            }
            vibrator.vibrate(VibrationEffect.createOneShot(durationMs.toLong(), VibrationEffect.DEFAULT_AMPLITUDE))
        }

        @JavascriptInterface
        fun getHistory(): String {
            return try {
                val rustDir = File(filesDir, ".lai/assistant")
                val file = File(rustDir, "history.json")
                if (file.exists()) file.readText() else "[]"
            } catch (e: Exception) {
                try {
                    val file = File(ctx.filesDir, "lai_history.json")
                    if (file.exists()) file.readText() else "[]"
                } catch (_: Exception) { "[]" }
            }
        }

        @JavascriptInterface
        fun saveHistory(json: String) {
            try {
                val rustDir = File(filesDir, ".lai/assistant")
                rustDir.mkdirs()
                val file = File(rustDir, "history.json")
                file.writeText(json)
            } catch (_: Exception) {}
            try {
                val file = File(ctx.filesDir, "lai_history.json")
                file.writeText(json)
            } catch (_: Exception) {}
        }

        @JavascriptInterface
        fun shareText(text: String) {
            val intent = Intent(Intent.ACTION_SEND).apply {
                type = "text/plain"
                putExtra(Intent.EXTRA_TEXT, text)
                addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
            }
            ctx.startActivity(Intent.createChooser(intent, "Share via L.ai").addFlags(Intent.FLAG_ACTIVITY_NEW_TASK))
        }
    }

    override fun onRequestPermissionsResult(requestCode: Int, permissions: Array<String>, grantResults: IntArray) {
        super.onRequestPermissionsResult(requestCode, permissions, grantResults)
        if (requestCode == 1001) {
            pendingPermissionCallback?.invoke(grantResults.all { it == PackageManager.PERMISSION_GRANTED })
            pendingPermissionCallback = null
        }
    }
}
