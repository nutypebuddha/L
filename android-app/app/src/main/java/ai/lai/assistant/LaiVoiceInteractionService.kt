package ai.lai.assistant

import android.service.voice.VoiceInteractionService
import android.util.Log

/**
 * The always-on system service that makes L.ai selectable as the device's
 * default digital assistant (ROLE_ASSISTANT), replacing Gemini.
 *
 * The system binds and keeps this service alive once L.ai holds the assistant
 * role. It must stay lightweight — no heavy work here. All interaction UI and
 * logic live in [LaiVoiceInteractionSession] (spawned via
 * [LaiVoiceInteractionSessionService]) which runs in a separate process.
 */
class LaiVoiceInteractionService : VoiceInteractionService() {

    companion object {
        private const val TAG = "lai-assistant-service"
    }

    override fun onReady() {
        super.onReady()
        Log.i(TAG, "VoiceInteractionService ready — L.ai is the active assistant")
    }
}
