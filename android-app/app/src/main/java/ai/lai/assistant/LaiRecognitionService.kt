package ai.lai.assistant

import android.content.Intent
import android.speech.RecognitionService
import android.speech.SpeechRecognizer

/**
 * Minimal [RecognitionService].
 *
 * Android intentionally couples [android.service.voice.VoiceInteractionService]
 * with a [RecognitionService]: an app cannot register as the digital-assistant
 * VoiceInteractionService without also declaring a recognition provider. This
 * class satisfies that requirement.
 *
 * L.ai does the actual speech capture inside [LaiVoiceInteractionSession] using
 * the platform [android.speech.SpeechRecognizer], so this service does not
 * itself run a recognizer; it reports "not available" for direct clients. A
 * future phase can back this with an offline engine (Vosk / whisper.cpp).
 */
class LaiRecognitionService : RecognitionService() {

    override fun onStartListening(recognizerIntent: Intent?, listener: Callback?) {
        try {
            listener?.error(SpeechRecognizer.ERROR_RECOGNIZER_BUSY)
        } catch (_: Exception) {
        }
    }

    override fun onCancel(listener: Callback?) {
        // No in-flight recognition to cancel.
    }

    override fun onStopListening(listener: Callback?) {
        // No in-flight recognition to stop.
    }
}
