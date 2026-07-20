package ai.lai.assistant

import android.os.Bundle
import android.service.voice.VoiceInteractionSession
import android.service.voice.VoiceInteractionSessionService

/**
 * Factory service that creates a [LaiVoiceInteractionSession] each time the
 * system starts a voice interaction (long-press power / assist gesture / any
 * app's ACTION_ASSIST while L.ai is the default assistant).
 *
 * Runs in a separate process from [LaiVoiceInteractionService], per Android's
 * voice-interaction architecture.
 */
class LaiVoiceInteractionSessionService : VoiceInteractionSessionService() {
    override fun onNewSession(args: Bundle?): VoiceInteractionSession {
        return LaiVoiceInteractionSession(this)
    }
}
