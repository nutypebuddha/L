use anyhow::Result;
use tokio::process::Command;

/// Play a WAV file through the speakers.
pub async fn play_file(path: &str) -> Result<()> {
    // Try paplay (PulseAudio), then aplay (ALSA), then play (sox)
    for cmd in &["paplay", "aplay", "play"] {
        let result = Command::new(cmd).args([path]).output().await;

        if let Ok(o) = result {
            if o.status.success() {
                return Ok(());
            }
        }
    }

    eprintln!("[audio] no playback tool found (install pulseaudio, alsa-utils, or sox)");
    Ok(())
}
