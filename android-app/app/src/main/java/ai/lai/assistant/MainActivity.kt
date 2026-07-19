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
    private var daemonProcess: java.lang.Process? = null
    private var daemonStdin: java.io.OutputStream? = null
    private var daemonStdout: java.io.BufferedReader? = null
    private val daemonLock = java.util.concurrent.locks.ReentrantLock()
    private var rpcId: Int = 0

    companion object {
        private const val TAG = "LaiMain"
        private const val MODEL_URL = "https://huggingface.co/Qwen/Qwen2.5-0.5B-Instruct-GGUF/resolve/main/qwen2.5-0.5b-instruct-q4_k_m.gguf"
        private const val MODEL_NAME = "qwen2.5-0.5b-instruct-q4_k_m.gguf"
        private const val MODEL_EXPECTED_BYTES = 491400032L
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
            settings.allowFileAccess = true
            settings.mediaPlaybackRequiresUserGesture = false
            settings.mixedContentMode = WebSettings.MIXED_CONTENT_ALWAYS_ALLOW
            settings.setSupportZoom(false)
            settings.builtInZoomControls = false
            settings.displayZoomControls = false
            settings.loadWithOverviewMode = true
            settings.useWideViewPort = true
            setLayerType(View.LAYER_TYPE_HARDWARE, null)

            webViewClient = object : WebViewClient() {
                override fun shouldOverrideUrlLoading(view: WebView?, request: WebResourceRequest?): Boolean {
                    return false
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
                override fun onPermissionRequest(request: PermissionRequest?) {
                    request?.grant(request.resources)
                }
            }

            addJavascriptInterface(LaiBridge(this@MainActivity), "LaiBridge")
            loadUrl("file:///android_asset/index.html")
            overScrollMode = View.OVER_SCROLL_NEVER
        }

        installLogcatFile()
        startLaiService()

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
        return f.exists() && f.length() >= MODEL_EXPECTED_BYTES - 1000
    }

    private suspend fun ensureModelReady() {
        if (modelReady()) {
            Log.i(TAG, "Model already present: ${modelFile().absolutePath}")
            return
        }
        val dir = File(filesDir, "models"); dir.mkdirs()
        val tmp = File(dir, "$MODEL_NAME.tmp")
        notifyJs("model-progress", 0)

        try {
            val conn = URL(MODEL_URL).openConnection() as HttpURLConnection
            conn.connectTimeout = 15_000
            conn.readTimeout = 60_000
            conn.connect()

            val total = conn.contentLength.toLong().coerceAtLeast(1)
            var downloaded = 0L
            conn.inputStream.use { input ->
                FileOutputStream(tmp).use { output ->
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

            tmp.renameTo(modelFile())
            Log.i(TAG, "Model ready: ${modelFile().absolutePath} (${modelFile().length()} bytes)")
            notifyJs("model-progress", 100)
        } catch (e: Exception) {
            Log.e(TAG, "Model download failed: ${e.message}")
            tmp.delete()
            notifyJs("model-error", 0)
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

    private fun installLogcatFile() {
        try {
            val logFile = File(filesDir, "lai.log")
            Runtime.getRuntime().exec(arrayOf("logcat", "-f", logFile.absolutePath, "-s", "LaiDaemon:LaiMain:*"))
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

        val candidates = listOf(src.absolutePath, run {
            val execDir = File(filesDir, "bin"); execDir.mkdirs()
            val bin = File(execDir, "lai")
            try { src.copyTo(bin, overwrite = true); bin.setExecutable(true, false) } catch (_: Exception) {}
            bin.absolutePath
        })

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
        Log.i(TAG, "Daemon ready: $daemonLog")
    }

    private fun sendRpcSync(method: String?, params: String): String {
        daemonLock.lock()
        try {
            val id = if (method != null) { rpcId++; rpcId } else null
            val msg = buildString {
                append("{\"jsonrpc\":\"2.0\"")
                if (id != null) append(",\"id\":$id")
                if (method != null) append(",\"method\":\"$method\"")
                append(",\"params\":$params}")
            }
            val stdin = daemonStdin ?: throw IllegalStateException("daemon stdin closed")
            stdin.write((msg + "\n").toByteArray(Charsets.UTF_8))
            stdin.flush()
            if (method == null) return ""

            val stdout = daemonStdout ?: throw IllegalStateException("daemon stdout closed")
            return stdout.readLine() ?: throw java.io.EOFException("daemon stdout EOF")
        } finally {
            daemonLock.unlock()
        }
    }

    private fun daemonStatus(): String {
        return buildString {
            append("started=$daemonStarted\n")
            append("log=$daemonLog\n")
            val alive = try { daemonProcess?.exitValue(); false } catch (_: IllegalThreadStateException) { true }
            append("alive=$alive\n")
            append("model=${if (modelReady()) modelFile().absolutePath else "none"}\n")
        }
    }

    private fun startLaiService() {
        try {
            val serviceIntent = Intent(this, LaiService::class.java)
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
                startForegroundService(serviceIntent)
            } else {
                startService(serviceIntent)
            }
        } catch (e: Exception) {
            e.printStackTrace()
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
                val raw = command
                    .removePrefix("assistant --text")
                    .removePrefix("--text")
                    .replace("--format json", "")
                    .trim()
                    .trim('"')
                val text = raw
                    .replace("\\\"", "\"")
                    .replace("\\\\\"", "\"")
                val params = """{"name":"chat","arguments":{"text":"${text.replace("\\", "\\\\").replace("\"", "\\\"")}"}}"""
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
