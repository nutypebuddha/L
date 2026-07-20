package ai.lai.assistant

import android.content.Context
import android.content.Intent
import android.graphics.Color
import android.graphics.drawable.GradientDrawable
import android.os.Bundle
import android.os.Handler
import android.os.Looper
import android.speech.RecognitionListener
import android.speech.RecognizerIntent
import android.speech.SpeechRecognizer
import android.service.voice.VoiceInteractionSession
import android.util.Log
import android.util.TypedValue
import android.view.Gravity
import android.view.View
import android.view.ViewGroup
import android.widget.LinearLayout
import android.widget.ScrollView
import android.widget.TextView
import kotlin.concurrent.thread

/**
 * The L.ai assistant overlay shown when the user invokes the assistant
 * (long-press power, assist gesture, or any app's ACTION_ASSIST while L.ai is
 * the default assistant).
 *
 * Pipeline: [SpeechRecognizer] (STT) → [LaiDaemon.chat] (on-device reasoning +
 * local LLM over MCP stdio) → [TtsManager] (spoken reply). Everything renders
 * in a compact card so the user sees the transcript and the answer.
 */
class LaiVoiceInteractionSession(context: Context) : VoiceInteractionSession(context) {

    private lateinit var statusView: TextView
    private lateinit var transcriptView: TextView
    private lateinit var replyView: TextView
    private lateinit var replyScroll: ScrollView

    private var recognizer: SpeechRecognizer? = null
    private var tts: TtsManager? = null
    private var daemon: LaiDaemon? = null

    private val main = Handler(Looper.getMainLooper())

    companion object {
        private const val TAG = "lai-assistant-session"
    }

    override fun onCreateContentView(): View {
        val ctx = context
        val pad = dp(20)

        val card = LinearLayout(ctx).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(pad, pad, pad, pad)
            background = GradientDrawable().apply {
                cornerRadius = dp(24).toFloat()
                setColor(Color.parseColor("#F2111122"))
                setStroke(dp(1), Color.parseColor("#FF7C6AFF"))
            }
        }

        val title = TextView(ctx).apply {
            text = "L.ai"
            setTextColor(Color.parseColor("#FFF0F0F8"))
            setTextSize(TypedValue.COMPLEX_UNIT_SP, 20f)
            typeface = android.graphics.Typeface.DEFAULT_BOLD
        }

        statusView = TextView(ctx).apply {
            text = "Listening…"
            setTextColor(Color.parseColor("#FF22D3EE"))
            setTextSize(TypedValue.COMPLEX_UNIT_SP, 13f)
            setPadding(0, dp(4), 0, dp(12))
        }

        transcriptView = TextView(ctx).apply {
            text = ""
            setTextColor(Color.parseColor("#FF9595B8"))
            setTextSize(TypedValue.COMPLEX_UNIT_SP, 15f)
        }

        replyView = TextView(ctx).apply {
            text = ""
            setTextColor(Color.parseColor("#FFF0F0F8"))
            setTextSize(TypedValue.COMPLEX_UNIT_SP, 16f)
            setPadding(0, dp(10), 0, 0)
        }

        replyScroll = ScrollView(ctx).apply {
            addView(replyView)
            layoutParams = LinearLayout.LayoutParams(
                ViewGroup.LayoutParams.MATCH_PARENT,
                dp(220),
            )
        }

        card.addView(title)
        card.addView(statusView)
        card.addView(transcriptView)
        card.addView(replyScroll)

        // Center the card with a scrim margin around it.
        val root = LinearLayout(ctx).apply {
            orientation = LinearLayout.VERTICAL
            gravity = Gravity.CENTER
            setPadding(dp(16), dp(16), dp(16), dp(16))
            addView(
                card,
                LinearLayout.LayoutParams(
                    ViewGroup.LayoutParams.MATCH_PARENT,
                    ViewGroup.LayoutParams.WRAP_CONTENT,
                ),
            )
        }
        return root
    }

    override fun onShow(args: Bundle?, showFlags: Int) {
        super.onShow(args, showFlags)
        daemon = LaiDaemon(context.applicationContext).also {
            thread(isDaemon = true) { it.start(modelPathOrNull()) }
        }
        tts = TtsManager(context)
        startListening()
    }

    private fun startListening() {
        val ctx = context
        if (!SpeechRecognizer.isRecognitionAvailable(ctx)) {
            setStatus("Speech recognition unavailable")
            return
        }
        val sr = SpeechRecognizer.createSpeechRecognizer(ctx)
        recognizer = sr
        sr.setRecognitionListener(object : RecognitionListener {
            override fun onReadyForSpeech(params: Bundle?) { setStatus("Listening…") }
            override fun onBeginningOfSpeech() { setStatus("Listening…") }
            override fun onRmsChanged(rmsdB: Float) {}
            override fun onBufferReceived(buffer: ByteArray?) {}
            override fun onEndOfSpeech() { setStatus("Thinking…") }
            override fun onError(error: Int) {
                setStatus("Didn't catch that")
                Log.w(TAG, "STT error=$error")
            }

            override fun onResults(results: Bundle?) {
                val text = results
                    ?.getStringArrayList(SpeechRecognizer.RESULTS_RECOGNITION)
                    ?.firstOrNull()
                    ?.trim()
                    .orEmpty()
                if (text.isEmpty()) {
                    setStatus("Didn't catch that")
                    return
                }
                main.post { transcriptView.text = "\u201C$text\u201D" }
                dispatch(text)
            }

            override fun onPartialResults(partialResults: Bundle?) {
                val text = partialResults
                    ?.getStringArrayList(SpeechRecognizer.RESULTS_RECOGNITION)
                    ?.firstOrNull()
                    .orEmpty()
                if (text.isNotEmpty()) main.post { transcriptView.text = text }
            }

            override fun onEvent(eventType: Int, params: Bundle?) {}
        })

        val intent = Intent(RecognizerIntent.ACTION_RECOGNIZE_SPEECH).apply {
            putExtra(
                RecognizerIntent.EXTRA_LANGUAGE_MODEL,
                RecognizerIntent.LANGUAGE_MODEL_FREE_FORM,
            )
            putExtra(RecognizerIntent.EXTRA_PARTIAL_RESULTS, true)
            putExtra(RecognizerIntent.EXTRA_CALLING_PACKAGE, ctx.packageName)
        }
        try {
            sr.startListening(intent)
        } catch (e: Exception) {
            setStatus("Mic unavailable — grant microphone permission")
            Log.e(TAG, "startListening failed: ${e.message}")
        }
    }

    /** Send the transcript to the daemon off the main thread, then speak. */
    private fun dispatch(text: String) {
        thread(isDaemon = true) {
            val d = daemon
            val reply = try {
                if (d == null || !d.start(modelPathOrNull())) {
                    "The L.ai engine isn't ready yet — try again in a moment."
                } else {
                    d.chat(text)
                }
            } catch (e: Exception) {
                Log.e(TAG, "chat failed: ${e.message}")
                "Something went wrong reaching the L.ai engine."
            }
            main.post {
                setStatus("")
                replyView.text = reply
                tts?.speak(reply)
            }
        }
    }

    private fun modelPathOrNull(): String? {
        val f = java.io.File(context.filesDir, "models/qwen2.5-0.5b-instruct-q4_k_m.gguf")
        return if (f.exists()) f.absolutePath else null
    }

    private fun setStatus(s: String) = main.post { statusView.text = s }

    override fun onHide() {
        try {
            recognizer?.destroy()
        } catch (_: Exception) {
        }
        recognizer = null
        tts?.shutdown()
        tts = null
        // Keep the daemon process for reuse across quick re-invocations? No —
        // the session is short-lived and runs in its own process, so release it.
        daemon?.stop()
        daemon = null
        super.onHide()
    }

    private fun dp(v: Int): Int =
        (v * context.resources.displayMetrics.density).toInt()
}
