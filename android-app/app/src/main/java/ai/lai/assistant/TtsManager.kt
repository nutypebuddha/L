package ai.lai.assistant

import android.content.Context
import android.speech.tts.TextToSpeech
import android.speech.tts.UtteranceProgressListener
import android.util.Log
import java.util.Locale

/**
 * Thin wrapper around Android [TextToSpeech]. L.ai previously rendered replies
 * as text only; the assistant surface needs to *speak*, matching Gemini.
 *
 * Initialization is asynchronous; [speak] queues text and flushes it once the
 * engine reports ready. [onDone] fires after an utterance finishes so the
 * caller (voice session) can advance its state machine.
 */
class TtsManager(context: Context) {

    private var tts: TextToSpeech? = null

    @Volatile
    private var ready: Boolean = false
    private var pending: String? = null

    var onDone: (() -> Unit)? = null

    companion object {
        private const val TAG = "lai-assistant-tts"
        private const val UTTERANCE_ID = "lai-reply"
    }

    init {
        tts = TextToSpeech(context.applicationContext) { status ->
            if (status == TextToSpeech.SUCCESS) {
                tts?.language = Locale.getDefault()
                tts?.setOnUtteranceProgressListener(object : UtteranceProgressListener() {
                    override fun onStart(utteranceId: String?) {}
                    override fun onDone(utteranceId: String?) {
                        onDone?.invoke()
                    }

                    @Deprecated("Deprecated in Java")
                    override fun onError(utteranceId: String?) {
                        onDone?.invoke()
                    }
                })
                ready = true
                pending?.let { speak(it); pending = null }
            } else {
                Log.w(TAG, "TTS init failed: status=$status")
            }
        }
    }

    /** Speak [text] immediately, interrupting any current utterance. */
    fun speak(text: String) {
        if (text.isBlank()) {
            onDone?.invoke()
            return
        }
        val engine = tts
        if (!ready || engine == null) {
            pending = text
            return
        }
        engine.speak(text, TextToSpeech.QUEUE_FLUSH, null, UTTERANCE_ID)
    }

    fun stop() {
        try {
            tts?.stop()
        } catch (_: Exception) {
        }
    }

    fun shutdown() {
        try {
            tts?.stop()
            tts?.shutdown()
        } catch (_: Exception) {
        }
        tts = null
        ready = false
    }
}
