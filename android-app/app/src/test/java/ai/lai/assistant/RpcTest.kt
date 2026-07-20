package ai.lai.assistant

import org.junit.jupiter.api.Assertions.*
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.assertThrows

/**
 * Regression coverage for the extracted pure helpers in [Rpc].
 * These back T64 (multiline RPC framing), T65 (no chat-text mangling), and
 * T68 (model SHA-256 / size integrity + resume Range header).
 */
class RpcTest {

    // ── T64: JSON-RPC frame building ──────────────────────────────

    @Test
    fun `buildFrame escapes embedded newlines`() {
        // Real user text containing a newline (T64). chatParamsJson builds the
        // params; buildFrame must serialize it as a single valid JSON object
        // (newline JSON-escaped, not splitting the line-delimited frame).
        val text = "line1\nline2"
        val frame = Rpc.buildFrame("tools/call", Rpc.chatParamsJson(text), 1)
        // Must be parseable as a single JSON object.
        val obj = org.json.JSONObject(frame)
        assertEquals("2.0", obj.getString("jsonrpc"))
        assertEquals(1, obj.getInt("id"))
        assertEquals("tools/call", obj.getString("method"))
        val params = obj.getJSONObject("params")
        assertEquals("line1\nline2", params.getJSONObject("arguments").getString("text"))
    }

    @Test
    fun `buildFrame escapes quotes and backslashes`() {
        val frame = Rpc.buildFrame("tools/call", """{"text":"he said \"hi\" \\ end"}""", 7)
        val params = org.json.JSONObject(frame).getJSONObject("params")
        assertEquals("he said \"hi\" \\ end", params.getString("text"))
    }

    @Test
    fun `buildFrame notification has no id or method`() {
        val frame = Rpc.buildFrame(null, """{"method":"notifications/initialized"}""", null)
        val obj = org.json.JSONObject(frame)
        assertFalse(obj.has("id"))
        assertFalse(obj.has("method"))
        assertTrue(obj.has("params"))
    }

    @Test
    fun `frameMatchesId matches by id and ignores stray frames`() {
        assertTrue(Rpc.frameMatchesId("""{"id":3,"result":{}}""", 3))
        assertFalse(Rpc.frameMatchesId("""{"id":9,"result":{}}""", 3))
        // Non-JSON / notification frames are not matches.
        assertFalse(Rpc.frameMatchesId("not json", 3))
        assertFalse(Rpc.frameMatchesId("""{"method":"foo"}""", 3))
    }

    // ── T65: chat param marshalling must not mangle text ──────────

    @Test
    fun `chatParamsJson preserves --format json substring`() {
        // The old code deleted "--format json" anywhere in the text.
        val text = "what does --format json do?"
        val json = Rpc.chatParamsJson(text)
        val obj = org.json.JSONObject(json)
        assertEquals("chat", obj.getString("name"))
        assertEquals(text, obj.getJSONObject("arguments").getString("text"))
    }

    @Test
    fun `chatParamsJson preserves leading and trailing quotes`() {
        val text = "\"quoted message\""
        val json = Rpc.chatParamsJson(text)
        assertEquals(text, org.json.JSONObject(json).getJSONObject("arguments").getString("text"))
    }

    @Test
    fun `chatParamsJson preserves backslashes and newlines`() {
        val text = "a\\b\nc"
        val json = Rpc.chatParamsJson(text)
        assertEquals(text, org.json.JSONObject(json).getJSONObject("arguments").getString("text"))
    }

    // ── T68: SHA-256 + model integrity ────────────────────────────

    @Test
    fun `sha256Hex matches known vector`() {
        // SHA-256("abc") — NIST test vector.
        val expected = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        assertEquals(expected, Rpc.sha256Hex("abc".toByteArray(Charsets.UTF_8)))
    }

    @Test
    fun `isModelValid requires exact size and matching sha`() {
        val sha = "deadbeef"
        assertTrue(Rpc.isModelValid(length = 100, expectedBytes = 100, actualSha = sha, expectedSha = sha))
        // Wrong size.
        assertFalse(Rpc.isModelValid(length = 99, expectedBytes = 100, actualSha = sha, expectedSha = sha))
        // Wrong sha.
        assertFalse(Rpc.isModelValid(length = 100, expectedBytes = 100, actualSha = "bad", expectedSha = sha))
        // Empty expected sha accepts any size-matched file (dev/unsigned).
        assertTrue(Rpc.isModelValid(length = 100, expectedBytes = 100, actualSha = "anything", expectedSha = ""))
    }

    @Test
    fun `rangeHeader builds correct value and null when nothing to resume`() {
        assertEquals("bytes=4096-", Rpc.rangeHeader(4096))
        assertNull(Rpc.rangeHeader(0))
    }
}
