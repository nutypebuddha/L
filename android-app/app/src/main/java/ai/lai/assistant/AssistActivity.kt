package ai.lai.assistant

import android.content.Intent
import android.os.Bundle
import androidx.appcompat.app.AppCompatActivity

/**
 * Handles the system ASSIST / VOICE_COMMAND intent so L.ai can be selected as
 * the device assistant (ROLE_ASSISTANT). It simply forwards into [MainActivity],
 * which hosts the WebView + daemon. Some OEM builds don't honor the role-request
 * dialog for ASSISTANT, so [MainActivity] also offers a Settings fallback.
 */
class AssistActivity : AppCompatActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        startActivity(
            Intent(this, MainActivity::class.java).apply {
                addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
                intent?.action?.let { action = it }
                intent?.extras?.let { putExtras(it) }
            }
        )
        finish()
    }
}
