use anyhow::Result;
use tokio::process::Command;

/// Speech-to-Text — shells out to `whisper` CLI (whisper.cpp).
pub async fn transcribe(wav_path: &str) -> Result<String> {
    // Try whisper CLI first (whisper.cpp)
    let result = Command::new("whisper")
        .args([
            wav_path,
            "--model",
            "tiny.en",
            "--language",
            "en",
            "--output_format",
            "txt",
            "--output_dir",
            "/tmp",
            "--no_timestamps",
        ])
        .output()
        .await;

    match result {
        Ok(o) if o.status.success() => {
            // whisper writes to /tmp/<filename>.txt
            let txt_path = wav_path.replace(".wav", ".txt");
            let text = tokio::fs::read_to_string(&txt_path)
                .await
                .unwrap_or_default();
            let _ = tokio::fs::remove_file(&txt_path).await;
            Ok(text.trim().to_string())
        }
        _ => {
            // Fallback: try whisper-tiny binary in PATH
            let result = Command::new("whisper-tiny").args([wav_path]).output().await;

            match result {
                Ok(o) if o.status.success() => {
                    let text = String::from_utf8_lossy(&o.stdout).trim().to_string();
                    Ok(text)
                }
                _ => {
                    eprintln!("[stt] whisper not found — returning raw input");
                    Ok(String::new())
                }
            }
        }
    }
}
