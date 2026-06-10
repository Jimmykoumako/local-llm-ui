use std::process::Command;
use tempfile::tempdir;

use super::{extension_from_filename, MAX_AUDIO_DURATION_SECS, TARGET_SAMPLE_RATE};

pub fn available() -> bool {
    Command::new("ffmpeg")
        .arg("-version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn convert(data: &[u8], filename: &str) -> Result<Vec<u8>, String> {
    let ext = extension_from_filename(filename);
    let dir = tempdir().map_err(|e| format!("Failed to create temp directory: {e}"))?;

    let input_path = dir.path().join(format!("input.{ext}"));
    std::fs::write(&input_path, data).map_err(|e| format!("Failed to write temp audio file: {e}"))?;

    let output_path = dir.path().join("output.wav");
    let duration = MAX_AUDIO_DURATION_SECS.to_string();

    let output = Command::new("ffmpeg")
        .args([
            "-y",
            "-i",
            input_path.to_str().ok_or("Invalid temp input path")?,
            "-t",
            &duration,
            "-ar",
            &TARGET_SAMPLE_RATE.to_string(),
            "-ac",
            "1",
            "-f",
            "wav",
            output_path.to_str().ok_or("Invalid temp output path")?,
        ])
        .output()
        .map_err(|e| format!("Failed to run ffmpeg: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let message = stderr
            .lines()
            .filter(|line| !line.is_empty())
            .last()
            .unwrap_or("unknown error");
        return Err(format!("ffmpeg conversion failed: {message}"));
    }

    std::fs::read(&output_path).map_err(|e| format!("Failed to read converted WAV: {e}"))
}
