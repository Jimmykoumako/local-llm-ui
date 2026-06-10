use hound::{SampleFormat, WavSpec, WavWriter};
use rubato::{FftFixedIn, Resampler};
use std::io::Cursor;
use symphonia::core::audio::{AudioBufferRef, Signal};
use symphonia::core::codecs::{DecoderOptions, CODEC_TYPE_NULL};
use symphonia::core::errors::Error;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

use super::{extension_from_filename, TARGET_SAMPLE_RATE};

pub fn convert(data: &[u8], filename: &str) -> Result<Vec<u8>, String> {
    let (mut samples, sample_rate) = decode_to_mono_f32(data, filename)?;
    samples = resample_to_target(samples, sample_rate)?;
    write_wav_bytes(&samples)
}

fn decode_to_mono_f32(data: &[u8], filename: &str) -> Result<(Vec<f32>, u32), String> {
    let ext = extension_from_filename(filename);
    let mut hint = Hint::new();
    if !ext.is_empty() && ext != "bin" {
        hint.with_extension(&ext);
    }

    let mss = MediaSourceStream::new(Box::new(Cursor::new(data.to_vec())), Default::default());
    let probed = symphonia::default::get_probe()
        .format(
            &hint,
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .map_err(|e| format!("Unsupported audio format: {e}"))?;

    let mut format = probed.format;
    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
        .ok_or_else(|| "No audio track found".to_string())?;

    let track_id = track.id;
    let sample_rate = track
        .codec_params
        .sample_rate
        .ok_or_else(|| "Unknown sample rate".to_string())?;
    let channels = track.codec_params.channels.map(|c| c.count()).unwrap_or(1);

    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .map_err(|e| format!("Decoder error: {e}"))?;

    let mut samples = Vec::new();

    loop {
        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(Error::ResetRequired) => continue,
            Err(Error::IoError(_)) => break,
            Err(e) => return Err(format!("Failed to read audio: {e}")),
        };

        if packet.track_id() != track_id {
            continue;
        }

        let decoded = decoder
            .decode(&packet)
            .map_err(|e| format!("Failed to decode audio: {e}"))?;

        append_mono_samples(&decoded, channels, &mut samples);
    }

    if samples.is_empty() {
        return Err("Audio file contains no samples".into());
    }

    Ok((samples, sample_rate))
}

fn append_mono_samples(buffer: &AudioBufferRef<'_>, channels: usize, out: &mut Vec<f32>) {
    let frames = buffer.frames();
    if frames == 0 || channels == 0 {
        return;
    }

    match buffer {
        AudioBufferRef::F32(buf) => {
            for frame in 0..frames {
                let mut sum = 0.0f32;
                for ch in 0..channels {
                    sum += buf.chan(ch)[frame];
                }
                out.push(sum / channels as f32);
            }
        }
        AudioBufferRef::S16(buf) => {
            for frame in 0..frames {
                let mut sum = 0.0f32;
                for ch in 0..channels {
                    sum += buf.chan(ch)[frame] as f32 / i16::MAX as f32;
                }
                out.push(sum / channels as f32);
            }
        }
        AudioBufferRef::S32(buf) => {
            for frame in 0..frames {
                let mut sum = 0.0f32;
                for ch in 0..channels {
                    sum += buf.chan(ch)[frame] as f32 / i32::MAX as f32;
                }
                out.push(sum / channels as f32);
            }
        }
        AudioBufferRef::U8(buf) => {
            for frame in 0..frames {
                let mut sum = 0.0f32;
                for ch in 0..channels {
                    sum += (buf.chan(ch)[frame] as f32 - 128.0) / 128.0;
                }
                out.push(sum / channels as f32);
            }
        }
        _ => {}
    }
}

fn resample_to_target(samples: Vec<f32>, sample_rate: u32) -> Result<Vec<f32>, String> {
    if sample_rate == TARGET_SAMPLE_RATE {
        return Ok(samples);
    }

    let mut resampler = FftFixedIn::<f32>::new(
        sample_rate as usize,
        TARGET_SAMPLE_RATE as usize,
        1024,
        1,
        1,
    )
    .map_err(|e| format!("Resampler error: {e}"))?;

    let input = vec![samples];
    let output = resampler
        .process(&input, None)
        .map_err(|e| format!("Resampling failed: {e}"))?;

    Ok(output.into_iter().next().unwrap_or_default())
}

fn write_wav_bytes(samples: &[f32]) -> Result<Vec<u8>, String> {
    let mut cursor = Cursor::new(Vec::new());
    {
        let spec = WavSpec {
            channels: 1,
            sample_rate: TARGET_SAMPLE_RATE,
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
        };

        let mut writer =
            WavWriter::new(&mut cursor, spec).map_err(|e| format!("WAV writer error: {e}"))?;

        for sample in samples {
            let scaled = (sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
            writer
                .write_sample(scaled)
                .map_err(|e| format!("Failed to write sample: {e}"))?;
        }

        writer
            .finalize()
            .map_err(|e| format!("Failed to finalize WAV: {e}"))?;
    }

    Ok(cursor.into_inner())
}
