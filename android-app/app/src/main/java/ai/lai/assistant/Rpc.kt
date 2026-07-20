package ai.lai.assistant

import org.json.JSONObject
import java.io.File
import java.io.InputStream
import java.security.MessageDigest

/**
 * Pure, side-effect-free helpers extracted from [MainActivity] so the RPC
 * framing, chat-param marshalling, and model-integrity logic can be unit-tested
 * without an Android runtime (T64/T65/T68 regression coverage).
 *
 * These mirror the exact behavior used by the activity; the activity methods
 * delegate to them. No Android SDK types are referenced here.
 */
object Rpc {

    /** Max time to wait for a daemon RPC response frame, in milliseconds. */
    const val RPC_READ_TIMEOUT_MS: Long = 30_000L

    /**
     * Build a single line-delimited JSON-RPC 2.0 frame.
     *
     * Uses [JSONObject] so embedded newlines / control chars / quotes are
     * JSON-escaped — a hand-built string would split the line-delimited framing
     * on a literal newline (T64). `params` must already be a JSON object string.
     *
     * @param method RPC method, or null for a notification (no id, no method).
     * @param paramsJson a JSON object literal, e.g. `{"text":"hi"}`.
     * @param id request id when [method] != null.
     */
    fun buildFrame(method: String?, paramsJson: String, id: Int?): String {
        return JSONObject().apply {
            put("jsonrpc", "2.0")
            if (id != null) put("id", id)
            if (method != null) put("method", method)
            put("params", JSONObject(paramsJson))
        }.toString()
    }

    /**
     * Marshal user chat text into the `tools/call` params object.
     *
     * Single escaping layer (T65): the raw text is placed verbatim into the
     * JSON value via [JSONObject], so substrings like "--format json", leading
     * quotes, or backslashes are preserved rather than mangled.
     */
    fun chatParamsJson(text: String): String {
        return JSONObject().apply {
            put("name", "chat")
            put("arguments", JSONObject().put("text", text))
        }.toString()
    }

    /**
     * True when [frame] is a JSON object carrying the expected [id]. Used to
     * match responses by id and ignore stray/notification frames (T64).
     */
    fun frameMatchesId(frame: String, id: Int): Boolean {
        return try {
            JSONObject(frame).optInt("id", -1) == id
        } catch (_: Exception) {
            false
        }
    }

    /** SHA-256 of a byte array, hex-encoded lowercase. */
    fun sha256Hex(bytes: ByteArray): String {
        val md = MessageDigest.getInstance("SHA-256")
        md.update(bytes)
        return md.digest().joinToString("") { "%02x".format(it) }
    }

    /** SHA-256 of a stream, hex-encoded lowercase. Caller closes [input]. */
    fun sha256Hex(input: InputStream): String {
        val md = MessageDigest.getInstance("SHA-256")
        val buf = ByteArray(64 * 1024)
        var read: Int
        while (input.read(buf).also { read = it } != -1) md.update(buf, 0, read)
        return md.digest().joinToString("") { "%02x".format(it) }
    }

    /** SHA-256 of a file, hex-encoded lowercase. */
    fun sha256Hex(file: File): String = file.inputStream().use { sha256Hex(it) }

    /**
     * Model integrity check (T68): valid only when size matches exactly AND,
     * if a [expectedSha] is pinned, the [actualSha] matches it. Empty expected
     * SHA means "accept any size-matched file" (dev/unsigned builds).
     */
    fun isModelValid(
        length: Long,
        expectedBytes: Long,
        actualSha: String,
        expectedSha: String
    ): Boolean {
        if (length != expectedBytes) return false
        if (expectedSha.isNotEmpty() && actualSha != expectedSha) return false
        return true
    }

    /**
     * Build the HTTP `Range` header value for resuming a download from [offset].
     * Returns null when there is nothing to resume.
     */
    fun rangeHeader(offset: Long): String? =
        if (offset > 0) "bytes=$offset-" else null
}
