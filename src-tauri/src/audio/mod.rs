mod ffmpeg;
mod native;

use base64::{engine::general_purpose::STANDARD, Engine};
use serde::Serialize;
use std::path::Path;

pub const TARGET_SAMPLE_RATE: u32 = 16_000;
pub const MAX_AUDIO_DURATION_SECS: f64 = 30.0;

#[derive(Debug, Clone, Serialize)]
pub struct AudioPrepareResult {
    pub base64: String,
    /// `ffmpeg` or `rust`
    pub method: String,
    pub duration_secs: f64,
    pub trimmed: bool,
}

pub fn extension_from_filename(filename: &str) -> String {
    Path::new(filename)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("bin")
        .to_ascii_lowercase()
}

pub fn convert_to_ollama_wav(data: &[u8], filename: &str) -> Result<AudioPrepareResult, String> {
    if data.is_empty() {
        return Err("Audio file is empty".into());
    }

    let (wav_bytes, method) = if ffmpeg::available() {
        match ffmpeg::convert(data, filename) {
            Ok(bytes) => (bytes, "ffmpeg".to_string()),
            Err(ffmpeg_error) => {
                let bytes = native::convert(data, filename).map_err(|native_error| {
                    format!(
                        "ffmpeg failed ({ffmpeg_error}); Rust fallback also failed ({native_error})"
                    )
                })?;
                (bytes, "rust".to_string())
            }
        }
    } else {
        let bytes = native::convert(data, filename)?;
        (bytes, "rust".to_string())
    };

    validate_wav(&wav_bytes)?;

    let mut wav_bytes = wav_bytes;
    let (duration_secs, trimmed) = trim_wav_if_needed(&mut wav_bytes)?;
    let base64 = STANDARD.encode(&wav_bytes);

    Ok(AudioPrepareResult {
        base64,
        method,
        duration_secs,
        trimmed,
    })
}

fn validate_wav(bytes: &[u8]) -> Result<(), String> {
    if bytes.len() < 12 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return Err("Converted audio is not a valid WAV file".into());
    }
    Ok(())
}

fn wav_duration_secs(bytes: &[u8]) -> Result<f64, String> {
    let reader = hound::WavReader::new(std::io::Cursor::new(bytes))
        .map_err(|e| format!("Failed to read WAV metadata: {e}"))?;
    let spec = reader.spec();
    let frames = reader.len();
    Ok(frames as f64 / spec.sample_rate as f64)
}

fn trim_wav_if_needed(bytes: &mut Vec<u8>) -> Result<(f64, bool), String> {
    let duration = wav_duration_secs(bytes)?;
    if duration <= MAX_AUDIO_DURATION_SECS {
        return Ok((duration, false));
    }

    let reader = hound::WavReader::new(std::io::Cursor::new(bytes.as_slice()))
        .map_err(|e| format!("Failed to read WAV for trimming: {e}"))?;
    let spec = reader.spec();
    let max_frames = (MAX_AUDIO_DURATION_SECS * spec.sample_rate as f64) as u32;

    let mut cursor = std::io::Cursor::new(Vec::new());
    {
        let mut writer = hound::WavWriter::new(&mut cursor, spec)
            .map_err(|e| format!("Failed to create trimmed WAV: {e}"))?;

        for (index, sample) in reader.into_samples::<i16>().enumerate() {
            if index as u32 >= max_frames {
                break;
            }
            writer
                .write_sample(sample.map_err(|e| format!("Sample read error: {e}"))?)
                .map_err(|e| format!("Failed to write trimmed sample: {e}"))?;
        }

        writer
            .finalize()
            .map_err(|e| format!("Failed to finalize trimmed WAV: {e}"))?;
    }

    *bytes = cursor.into_inner();
    Ok((MAX_AUDIO_DURATION_SECS, true))
}
