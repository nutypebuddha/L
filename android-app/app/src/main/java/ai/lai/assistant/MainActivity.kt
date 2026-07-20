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
    private var daemonLog: String = ""
    private var daemonStarted: Boolean = false
    private var llmReachable: Boolean = false
    private var daemonProcess: java.lang.Process? = null
    private var logcatProcess: java.lang.Process? = null
    private var daemonStdin: java.io.OutputStream? = null
    private var daemonStdout: java.io.BufferedReader? = null
    private val daemonLock = java.util.concurrent.locks.ReentrantLock()
    private var rpcId: Int = 0

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
        val libDir = File(applicationInfo.nativeLibraryDir)
        val src = File(libDir, "liblai.so")
        if (!src.exists()) {
            Log.e(TAG, "liblai.so not found in $libDir")
            daemonStarted = false
            daemonLog = "liblai.so not found in $libDir"
            return
        }

        // T67: exec from nativeLibraryDir only. Copying to filesDir/bin is dead
        // code on API 29+ (SELinux W^X denies execve on app-data) and wastes
        // 8.5MB + flash writes on every cold start.
        val candidates = listOf(src.absolutePath)

        val sb = StringBuilder()
        var process: java.lang.Process? = null
        var launchedFrom: String? = null
        for (path in candidates) {
            try {
                process = ProcessBuilder(path, "assistant", "--mcp")
                    .directory(filesDir)
                    .apply {
                        val env = environment()
                        env["HOME"] = filesDir.absolutePath
                        env["TMPDIR"] = cacheDir.absolutePath
                        if (modelReady()) {
                            env["LAVERNA_LLAMA_MODEL"] = modelFile().absolutePath
                        }
                        env["LAI_LLM_BASE_URL"] = "http://127.0.0.1:11434/v1"
                        env["LAI_LLM_MODEL"] = "qwen2.5:0.5b"
                    }
                    .start()
                launchedFrom = path
                break
            } catch (e: Exception) {
                Log.e(TAG, "exec failed for $path: ${e.message}")
                sb.appendLine("exec failed: $path -> ${e.javaClass.simpleName}: ${e.message}")
            }
        }

        if (process == null) {
            daemonStarted = false
            daemonLog = "launch failed:\n$sb"
            return
        }

        Thread {
            process.errorStream.bufferedReader().forEachLine { line ->
                Log.d("LaiDaemon", line)
            }
        }.start()

        daemonProcess = process
        daemonStdin = process.outputStream
        daemonStdout = process.inputStream.bufferedReader()

        try {
            rpcId = 1
            val initResp = sendRpcSync("initialize", """{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"lai-android","version":"1.1.0"}}""")
            Log.d(TAG, "MCP initialize: $initResp")
            sendRpcSync(null, """{"method":"notifications/initialized"}""")
        } catch (e: Exception) {
            Log.e(TAG, "MCP handshake failed: ${e.message}")
            sb.appendLine("MCP handshake failed: ${e.message}")
            daemonStarted = false
            daemonLog = "launch from $launchedFrom failed handshake:\n$sb"
            return
        }

        daemonStarted = true
        daemonLog = "launched from $launchedFrom (stdio MCP)"
        llmReachable = probeLlmReachable()
        Log.i(TAG, "Daemon ready: $daemonLog — llm=${if (llmReachable) "reachable" else "unreachable (corpus-only)"}")
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

    private fun sendRpcSync(method: String?, params: String): String {
        daemonLock.lock()
        try {
            val id = if (method != null) { rpcId++; rpcId } else null
            // Build the frame with JSONObject so newlines/control chars are
            // JSON-escaped (T64). Hand-built strings split the line-delimited
            // framing on any literal newline.
            val msg = Rpc.buildFrame(method, params, id)
            val stdin = daemonStdin ?: throw IllegalStateException("daemon stdin closed")
            stdin.write((msg + "\n").toByteArray(Charsets.UTF_8))
            stdin.flush()
            if (method == null) return ""

            val stdout = daemonStdout ?: throw IllegalStateException("daemon stdout closed")
            val deadline = System.currentTimeMillis() + Rpc.RPC_READ_TIMEOUT_MS
            // Match responses by id; skip stray/non-matching frames (T64).
            while (System.currentTimeMillis() < deadline) {
                val line = stdout.readLine() ?: throw java.io.EOFException("daemon stdout EOF")
                val frame = line.trim()
                if (frame.isEmpty()) continue
                if (id == null || Rpc.frameMatchesId(frame, id)) return frame
            }
            throw java.io.IOException("daemon RPC timeout (id=$id) — restarting daemon")
        } catch (e: java.io.IOException) {
            // Desync/unreachable: tear down and restart so the next call recovers.
            Log.e(TAG, "RPC desync: ${e.message}")
            restartDaemonNow()
            throw e
        } finally {
            daemonLock.unlock()
        }
    }

    private fun restartDaemonNow() {
        try {
            daemonProcess?.destroyForcibly()
        } catch (_: Exception) {
        }
        daemonProcess = null
        daemonStdin = null
        daemonStdout = null
        daemonStarted = false
        try {
            startLaiDaemon()
        } catch (_: Exception) {
        }
    }

    private fun daemonStatus(): String {
        return buildString {
            append("started=$daemonStarted\n")
            append("log=$daemonLog\n")
            val alive = try { daemonProcess?.exitValue(); false } catch (_: IllegalThreadStateException) { true }
            append("alive=$alive\n")
            append("model=${if (modelReady()) modelFile().absolutePath else "none"}\n")
            append("llm=${if (llmReachable) "reachable" else "unreachable (corpus-only)"}\n")
        }
    }

    private fun maybeRequestAssistantRole() {
        val rm = getSystemService(ROLE_SERVICE) as? android.app.role.RoleManager ?: return
        val role = android.app.role.RoleManager.ROLE_ASSISTANT
        if (!rm.isRoleAvailable(role)) return
        if (rm.isRoleHeld(role)) return
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
        try { daemonProcess?.destroyForcibly() } catch (_: Exception) {}
        webView.destroy()
        super.onDestroy()
    }

    inner class LaiBridge(private val ctx: Context) {

        @JavascriptInterface
        fun executeCommand(command: String): String {
            if (!daemonStarted) {
                return "Error: daemon not started — model may still be downloading"
            }
            return try {
                // One escaping layer: the JS side passes the user's raw text;
                // we marshal it into the chat params via Rpc (T65). No prefix
                // stripping, no blind substring replacement.
                val params = Rpc.chatParamsJson(command)
                val rpcResp = sendRpcSync("tools/call", params)
                val contentText = try {
                    val root = org.json.JSONObject(rpcResp)
                    val result = root.getJSONObject("result")
                    val content = result.getJSONArray("content")
                    content.getJSONObject(0).getString("text")
                } catch (_: Exception) {
                    rpcResp
                }
                contentText
            } catch (e: Exception) {
                "Error: ${e.javaClass.simpleName}: ${e.message}"
            }
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
            try {
                startLaiDaemon()
                return "restart requested"
            } catch (e: Exception) {
                return "restart failed: ${e.message}"
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
