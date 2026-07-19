use anyhow::Result;
use tokio::process::Command;

/// Text-to-Speech — shells out to `piper` CLI or `espeak-ng`.
pub async fn speak(text: &str) -> Result<()> {
    if text.trim().is_empty() {
        return Ok(());
    }

    let wav_path = "/tmp/lai-tts.wav";

    // Try piper first
    let piper_result = Command::new("piper")
        .args([
            "--model",
            "en_US-lessac-medium.onnx",
            "--output_file",
            wav_path,
        ])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .output()
        .await;

    match piper_result {
        Ok(o) if o.status.success() => {
            super::audio::play_file(wav_path).await?;
            let _ = tokio::fs::remove_file(wav_path).await;
            return Ok(());
        }
        _ => {}
    }

    // Fallback: espeak-ng
    let espeak_result = Command::new("espeak-ng")
        .args(["-w", wav_path, text])
        .output()
        .await;

    match espeak_result {
        Ok(o) if o.status.success() => {
            super::audio::play_file(wav_path).await?;
            let _ = tokio::fs::remove_file(wav_path).await;
            return Ok(());
        }
        _ => {}
    }

    // Final fallback: espeak (no file output, just plays)
    let _ = Command::new("espeak").args([text]).output().await;

    Ok(())
}
