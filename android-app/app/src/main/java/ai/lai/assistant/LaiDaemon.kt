package ai.lai.assistant

import android.content.Context
import android.util.Log
import java.io.File
import java.util.concurrent.locks.ReentrantLock

/**
 * Reusable client for the native `lai assistant --mcp` daemon (`liblai.so`).
 *
 * Spawns the native process, performs the MCP stdio handshake, and exposes a
 * synchronous [chat] call. Both the WebView host ([MainActivity]) and the
 * voice-assistant session ([LaiVoiceInteractionSession]) drive the daemon
 * through this class so the transport lives in exactly one place.
 *
 * Transport is line-delimited JSON-RPC 2.0 over the process's stdin/stdout,
 * framed via [Rpc]. Responses are matched by id; stray frames are skipped.
 *
 * Not thread-safe across instances of the daemon, but every RPC round-trip is
 * guarded by an internal lock so concurrent callers serialize safely.
 */
class LaiDaemon(private val appContext: Context) {

    private var process: java.lang.Process? = null
    private var stdin: java.io.OutputStream? = null
    private var stdout: java.io.BufferedReader? = null
    private val lock = ReentrantLock()
    private var rpcId: Int = 0

    @Volatile
    var started: Boolean = false
        private set

    @Volatile
    var lastLog: String = ""
        private set

    companion object {
        private const val TAG = "LaiDaemon"
        const val LLM_BASE_URL = "http://127.0.0.1:11434/v1"
        const val LLM_MODEL = "qwen2.5:0.5b"
    }

    /** True while the underlying process is still running. */
    fun isAlive(): Boolean = try {
        process?.exitValue()
        false
    } catch (_: IllegalThreadStateException) {
        true
    } catch (_: Exception) {
        false
    }

    /**
     * Launch the daemon and complete the MCP handshake. Idempotent when already
     * started and alive. Returns true on success.
     *
     * @param modelPath optional GGUF path exported as LAVERNA_LLAMA_MODEL.
     */
    @Synchronized
    fun start(modelPath: String? = null): Boolean {
        if (started && isAlive()) return true

        val libDir = File(appContext.applicationInfo.nativeLibraryDir)
        val bin = File(libDir, "liblai.so")
        if (!bin.exists()) {
            started = false
            lastLog = "liblai.so not found in $libDir"
            Log.e(TAG, lastLog)
            return false
        }

        val proc: java.lang.Process = try {
            ProcessBuilder(bin.absolutePath, "assistant", "--mcp")
                .directory(appContext.filesDir)
                .apply {
                    val env = environment()
                    env["HOME"] = appContext.filesDir.absolutePath
                    env["TMPDIR"] = appContext.cacheDir.absolutePath
                    if (modelPath != null) env["LAVERNA_LLAMA_MODEL"] = modelPath
                    env["LAI_LLM_BASE_URL"] = LLM_BASE_URL
                    env["LAI_LLM_MODEL"] = LLM_MODEL
                }
                .start()
        } catch (e: Exception) {
            started = false
            lastLog = "exec failed: ${e.javaClass.simpleName}: ${e.message}"
            Log.e(TAG, lastLog)
            return false
        }

        // Drain stderr so the process never blocks on a full pipe.
        Thread {
            try {
                proc.errorStream.bufferedReader().forEachLine { Log.d(TAG, it) }
            } catch (_: Exception) {
            }
        }.apply { isDaemon = true }.start()

        process = proc
        stdin = proc.outputStream
        stdout = proc.inputStream.bufferedReader()

        return try {
            rpcId = 1
            sendRpc(
                "initialize",
                """{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"lai-android","version":"1.2.0"}}""",
            )
            sendRpc(null, """{"method":"notifications/initialized"}""")
            started = true
            lastLog = "launched (stdio MCP)"
            Log.i(TAG, lastLog)
            true
        } catch (e: Exception) {
            started = false
            lastLog = "MCP handshake failed: ${e.message}"
            Log.e(TAG, lastLog)
            false
        }
    }

    /**
     * Send a chat query to the daemon and return the plain reply text.
     *
     * Extracts the `response` field from the assistant's JSON payload (falling
     * back to the raw content text). Throws on transport failure so callers can
     * surface a spoken/visible error.
     */
    fun chat(text: String): String {
        val params = Rpc.chatParamsJson(text)
        val frame = sendRpc("tools/call", params)
        val contentText = try {
            val root = org.json.JSONObject(frame)
            val result = root.getJSONObject("result")
            result.getJSONArray("content").getJSONObject(0).getString("text")
        } catch (_: Exception) {
            frame
        }
        // The chat tool wraps the answer as {"response": "...", ...}; unwrap it
        // for a clean spoken/rendered string.
        return try {
            org.json.JSONObject(contentText).optString("response", contentText)
        } catch (_: Exception) {
            contentText
        }
    }

    private fun sendRpc(method: String?, paramsJson: String): String {
        lock.lock()
        try {
            val id = if (method != null) ++rpcId else null
            val msg = Rpc.buildFrame(method, paramsJson, id)
            val out = stdin ?: throw IllegalStateException("daemon stdin closed")
            out.write((msg + "\n").toByteArray(Charsets.UTF_8))
            out.flush()
            if (method == null) return ""

            val reader = stdout ?: throw IllegalStateException("daemon stdout closed")
            val deadline = System.currentTimeMillis() + Rpc.RPC_READ_TIMEOUT_MS
            while (System.currentTimeMillis() < deadline) {
                val line = reader.readLine() ?: throw java.io.EOFException("daemon stdout EOF")
                val f = line.trim()
                if (f.isEmpty()) continue
                if (id == null || Rpc.frameMatchesId(f, id)) return f
            }
            throw java.io.IOException("daemon RPC timeout (id=$id)")
        } finally {
            lock.unlock()
        }
    }

    /** Terminate the daemon process. Safe to call multiple times. */
    fun stop() {
        try {
            process?.destroyForcibly()
        } catch (_: Exception) {
        }
        process = null
        stdin = null
        stdout = null
        started = false
    }
}
