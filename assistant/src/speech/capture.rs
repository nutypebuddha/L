use anyhow::Result;
use tokio::process::Command;

/// Capture audio from the microphone using `rec` (sox) or `arecord` (alsa).
/// Saves to a temp WAV file and returns the path.
pub async fn capture_to_file(duration_secs: u32) -> Result<String> {
    let path = "/tmp/lai-capture.wav";

    // Try sox `rec` first, then arecord
    let result = Command::new("rec")
        .args([
            "-r",
            "16000",
            "-c",
            "1",
            "-b",
            "16",
            path,
            "trim",
            "0",
            &duration_secs.to_string(),
        ])
        .output()
        .await;

    match result {
        Ok(o) if o.status.success() => Ok(path.to_string()),
        _ => {
            // Fallback: arecord
            let result = Command::new("arecord")
                .args([
                    "-f",
                    "S16_LE",
                    "-r",
                    "16000",
                    "-c",
                    "1",
                    "-d",
                    &duration_secs.to_string(),
                    "-t",
                    "wav",
                    path,
                ])
                .output()
                .await?;

            if result.status.success() {
                Ok(path.to_string())
            } else {
                anyhow::bail!("no audio capture tool found (install sox or alsa-utils)")
            }
        }
    }
}

/// Capture audio until silence is detected (energy-based VAD).
/// Records in chunks, checks energy, stops after silence_threshold consecutive silent chunks.
pub async fn capture_until_silence(
    silence_threshold: usize,
    chunk_duration_ms: u32,
) -> Result<String> {
    let path = "/tmp/lai-capture.wav";
    let tmp_raw = "/tmp/lai-capture-raw.pcm";

    // Record raw PCM in a background process
    let mut child = Command::new("arecord")
        .args([
            "-f", "S16_LE", "-r", "16000", "-c", "1", "-t", "raw", tmp_raw,
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()?;

    // Wait a bit for initial speech, then start checking energy
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    let mut silent_chunks = 0usize;
    let chunk_samples = (16000 * chunk_duration_ms / 1000) as usize;

    loop {
        tokio::time::sleep(std::time::Duration::from_millis(chunk_duration_ms as u64)).await;

        // Read the raw file and check energy of the last chunk
        match std::fs::read(tmp_raw) {
            Ok(data) => {
                let samples = data.len() / 2; // i16 = 2 bytes
                if samples < chunk_samples {
                    continue;
                }

                // Check energy of the last chunk_samples
                let start = (samples - chunk_samples) * 2;
                let chunk = &data[start..];
                let energy = compute_rms_energy(chunk);

                if energy < 500.0 {
                    // Silence
                    silent_chunks += 1;
                    if silent_chunks >= silence_threshold {
                        break;
                    }
                } else {
                    silent_chunks = 0;
                }
            }
            Err(_) => continue,
        }
    }

    child.kill().await.ok();

    // Convert raw PCM to WAV using sox
    let _ = Command::new("sox")
        .args([
            "-t",
            "raw",
            "-r",
            "16000",
            "-c",
            "1",
            "-b",
            "16",
            "-e",
            "signed-integer",
            tmp_raw,
            path,
        ])
        .output()
        .await;

    let _ = tokio::fs::remove_file(tmp_raw).await;

    Ok(path.to_string())
}

/// Compute RMS energy of 16-bit PCM audio samples.
fn compute_rms_energy(data: &[u8]) -> f64 {
    let samples: Vec<i16> = data
        .chunks_exact(2)
        .map(|c| i16::from_le_bytes([c[0], c[1]]))
        .collect();

    if samples.is_empty() {
        return 0.0;
    }

    let sum: f64 = samples.iter().map(|&s| (s as f64).powi(2)).sum();
    (sum / samples.len() as f64).sqrt()
}
