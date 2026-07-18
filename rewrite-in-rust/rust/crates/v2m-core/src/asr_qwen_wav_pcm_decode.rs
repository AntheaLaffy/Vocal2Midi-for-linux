//! Qwen ASR WAV PCM fallback decode contract.
//!
//! This module mirrors the fixture-backed same-rate fallback behavior from
//! `inference/qwen3asr_dml/utils.py`. It is not wired into Python runtime
//! loading and does not own resampling.

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
/// Python-compatible failure returned by the WAV fallback decoder.
pub struct WavPcmDecodeError {
    /// The Python-compatible error type.
    pub error_type: &'static str,
    /// The message text.
    pub message: String,
}

impl WavPcmDecodeError {
    fn value_error(message: impl Into<String>) -> Self {
        Self {
            error_type: "ValueError",
            message: message.into(),
        }
    }

    fn overflow_error(message: impl Into<String>) -> Self {
        Self {
            error_type: "OverflowError",
            message: message.into(),
        }
    }
}

impl fmt::Display for WavPcmDecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.error_type, self.message)
    }
}

impl std::error::Error for WavPcmDecodeError {}

/// Decodes mono or multichannel integer PCM WAV bytes into mono `f32` samples.
///
/// The source sample rate must equal `target_sample_rate`; resampling belongs to
/// [`crate::asr_resample_poly`].
///
/// # Errors
///
/// Returns [`WavPcmDecodeError`] for malformed or truncated RIFF/WAVE data,
/// unsupported encoding or sample widths, invalid channel metadata, or a sample
/// rate mismatch.
pub fn load_wav_audio_fallback_bytes(
    wav_bytes: &[u8],
    target_sample_rate: u32,
) -> Result<Vec<f32>, WavPcmDecodeError> {
    let wav = parse_wave_pcm(wav_bytes)?;

    if !matches!(wav.sample_width, 1..=4) {
        return Err(WavPcmDecodeError::value_error(format!(
            "Unsupported WAV sample width: {}",
            wav.sample_width
        )));
    }

    if wav.sample_rate != target_sample_rate {
        return Err(WavPcmDecodeError::value_error(format!(
            "WAV resampling is owned by asr_resample_poly_contract: source_rate={}, target_rate={}",
            wav.sample_rate, target_sample_rate
        )));
    }

    let audio = match wav.sample_width {
        1 => wav
            .data
            .iter()
            .copied()
            .map(|value| (value as f32 - 128.0) / 128.0)
            .collect(),
        2 => wav
            .data
            .chunks_exact(2)
            .map(|sample| i16::from_le_bytes([sample[0], sample[1]]) as f32 / 32768.0)
            .collect(),
        3 => wav
            .data
            .chunks_exact(3)
            .map(|sample| {
                let value =
                    (sample[0] as i32) | ((sample[1] as i32) << 8) | ((sample[2] as i32) << 16);
                let signed = if value & 0x80_0000 == 0 {
                    value
                } else {
                    value - 0x100_0000
                };
                signed as f32 / 8_388_608.0
            })
            .collect(),
        4 => wav
            .data
            .chunks_exact(4)
            .map(|sample| {
                i32::from_le_bytes([sample[0], sample[1], sample[2], sample[3]]) as f32
                    / 2_147_483_648.0
            })
            .collect(),
        _ => unreachable!("sample_width was validated above"),
    };

    Ok(mean_channels(audio, wav.channels))
}

/// Decodes WAV bytes and applies Python-compatible start/duration slicing.
///
/// # Errors
///
/// Returns [`WavPcmDecodeError`] when WAV decoding fails or a slice argument is
/// NaN or infinite and cannot be projected to a Python integer index.
pub fn load_audio_forced_fallback_bytes(
    wav_bytes: &[u8],
    target_sample_rate: u32,
    start_second: Option<f64>,
    duration: Option<f64>,
) -> Result<Vec<f32>, WavPcmDecodeError> {
    let audio = load_wav_audio_fallback_bytes(wav_bytes, target_sample_rate)?;
    let start = python_trunc_to_isize((start_second.unwrap_or(0.0)) * target_sample_rate as f64)?;
    let end = duration
        .filter(|duration| *duration != 0.0)
        .map(|duration| python_trunc_to_isize(duration * target_sample_rate as f64))
        .transpose()?
        .map(|duration| start + duration);
    let clamped_start = normalize_slice_index(start, audio.len());
    let clamped_end = end
        .map(|value| normalize_slice_index(value, audio.len()))
        .unwrap_or(audio.len());

    if clamped_start > clamped_end {
        return Ok(Vec::new());
    }
    Ok(audio[clamped_start..clamped_end].to_vec())
}

fn python_trunc_to_isize(value: f64) -> Result<isize, WavPcmDecodeError> {
    if value.is_nan() {
        return Err(WavPcmDecodeError::value_error(
            "cannot convert float NaN to integer",
        ));
    }
    if value.is_infinite() {
        return Err(WavPcmDecodeError::overflow_error(
            "cannot convert float infinity to integer",
        ));
    }
    Ok(value.trunc() as isize)
}

fn normalize_slice_index(index: isize, len: usize) -> usize {
    let len = len as isize;
    let normalized = if index < 0 { len + index } else { index };
    normalized.clamp(0, len) as usize
}

fn mean_channels(samples: Vec<f32>, channels: u16) -> Vec<f32> {
    let channels = channels as usize;
    if channels <= 1 {
        return samples;
    }

    samples
        .chunks_exact(channels)
        .map(|frame| frame.iter().copied().sum::<f32>() / channels as f32)
        .collect()
}

#[derive(Debug, Clone, Copy)]
struct ParsedWave<'a> {
    channels: u16,
    sample_rate: u32,
    sample_width: u16,
    data: &'a [u8],
}

fn parse_wave_pcm(bytes: &[u8]) -> Result<ParsedWave<'_>, WavPcmDecodeError> {
    if bytes.len() < 12 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return Err(WavPcmDecodeError::value_error(
            "file does not start with RIFF id",
        ));
    }

    let mut offset = 12usize;
    let mut fmt: Option<(u16, u32, u16)> = None;
    let mut data: Option<&[u8]> = None;
    while offset + 8 <= bytes.len() {
        let id = &bytes[offset..offset + 4];
        let len = read_u32(bytes, offset + 4)? as usize;
        let start = offset + 8;
        let end = start
            .checked_add(len)
            .ok_or_else(|| WavPcmDecodeError::value_error("invalid chunk length"))?;
        if end > bytes.len() {
            return Err(WavPcmDecodeError::value_error("truncated chunk"));
        }

        if id == b"fmt " {
            if len < 16 {
                return Err(WavPcmDecodeError::value_error("invalid fmt chunk size"));
            }
            let format_tag = read_u16(bytes, start)?;
            if format_tag != 1 {
                return Err(WavPcmDecodeError::value_error(format!(
                    "unknown format: {format_tag}"
                )));
            }
            let channels = read_u16(bytes, start + 2)?;
            if channels == 0 {
                return Err(WavPcmDecodeError::value_error("bad # of channels"));
            }
            let sample_rate = read_u32(bytes, start + 4)?;
            let bits_per_sample = read_u16(bytes, start + 14)?;
            let sample_width = bits_per_sample.div_ceil(8);
            fmt = Some((channels, sample_rate, sample_width));
        } else if id == b"data" {
            data = Some(&bytes[start..end]);
            if fmt.is_some() {
                break;
            }
        }

        offset = end + (len % 2);
    }

    let (channels, sample_rate, sample_width) =
        fmt.ok_or_else(|| WavPcmDecodeError::value_error("fmt chunk and/or data chunk missing"))?;
    let data =
        data.ok_or_else(|| WavPcmDecodeError::value_error("fmt chunk and/or data chunk missing"))?;
    Ok(ParsedWave {
        channels,
        sample_rate,
        sample_width,
        data,
    })
}

fn read_u16(bytes: &[u8], offset: usize) -> Result<u16, WavPcmDecodeError> {
    let end = offset + 2;
    let value = bytes
        .get(offset..end)
        .ok_or_else(|| WavPcmDecodeError::value_error("truncated chunk"))?;
    Ok(u16::from_le_bytes([value[0], value[1]]))
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, WavPcmDecodeError> {
    let end = offset + 4;
    let value = bytes
        .get(offset..end)
        .ok_or_else(|| WavPcmDecodeError::value_error("truncated chunk"))?;
    Ok(u32::from_le_bytes([value[0], value[1], value[2], value[3]]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    const FIXTURES: &str = include_str!("../../../../fixtures/asr_qwen_wav_pcm_decode_core.jsonl");

    #[test]
    fn asr_qwen_wav_pcm_decode_core_matches_fixtures() {
        for (index, line) in FIXTURES.lines().filter(|line| !line.is_empty()).enumerate() {
            let case: Value = serde_json::from_str(line).unwrap();
            let wav_bytes = decode_base64(case["wav_b64"].as_str().unwrap()).unwrap();
            let target_rate = case["target_rate"].as_u64().unwrap() as u32;
            let actual = match case["call"].as_str().unwrap() {
                "_load_wav_audio" | "rust_same_rate_boundary" => {
                    load_wav_audio_fallback_bytes(&wav_bytes, target_rate)
                }
                "load_audio_forced_fallback" => load_audio_forced_fallback_bytes(
                    &wav_bytes,
                    target_rate,
                    optional_float(&case, "start_second"),
                    optional_float(&case, "duration"),
                ),
                other => panic!("unknown fixture call {other}"),
            };

            let expected = &case["expected"];
            if expected["ok"].as_bool().unwrap() {
                let actual = actual.unwrap_or_else(|err| panic!("case {index} failed: {err}"));
                assert_eq!(
                    actual.len(),
                    expected["shape"][0].as_u64().unwrap() as usize
                );
                for (sample_index, (actual, expected)) in actual
                    .iter()
                    .zip(expected["values"].as_array().unwrap().iter())
                    .enumerate()
                {
                    let expected = expected.as_f64().unwrap() as f32;
                    assert!(
                        (*actual - expected).abs() <= 1e-7,
                        "case {index} sample {sample_index}: actual {actual:?}, expected {expected:?}"
                    );
                }
            } else {
                let err = actual.expect_err("expected fixture error");
                assert_eq!(err.error_type, expected["error_type"].as_str().unwrap());
                assert_eq!(err.message, expected["message"].as_str().unwrap());
            }
        }
    }

    fn optional_float(case: &Value, key: &str) -> Option<f64> {
        match case.get(key)? {
            Value::String(value) if value == "nan" => Some(f64::NAN),
            Value::String(value) if value == "inf" => Some(f64::INFINITY),
            Value::String(value) if value == "-inf" => Some(f64::NEG_INFINITY),
            value => value.as_f64(),
        }
    }

    fn decode_base64(input: &str) -> Result<Vec<u8>, String> {
        let mut out = Vec::with_capacity(input.len() * 3 / 4);
        let mut chunk = [0u8; 4];
        let mut len = 0;

        for byte in input.bytes().filter(|byte| !byte.is_ascii_whitespace()) {
            let value = match byte {
                b'A'..=b'Z' => byte - b'A',
                b'a'..=b'z' => byte - b'a' + 26,
                b'0'..=b'9' => byte - b'0' + 52,
                b'+' => 62,
                b'/' => 63,
                b'=' => 64,
                other => return Err(format!("invalid base64 byte {other}")),
            };
            chunk[len] = value;
            len += 1;
            if len == 4 {
                push_base64_chunk(&mut out, chunk)?;
                len = 0;
            }
        }

        if len != 0 {
            return Err("truncated base64 input".to_string());
        }
        Ok(out)
    }

    fn push_base64_chunk(out: &mut Vec<u8>, chunk: [u8; 4]) -> Result<(), String> {
        if chunk[0] == 64 || chunk[1] == 64 {
            return Err("invalid base64 padding".to_string());
        }
        let b0 = (chunk[0] << 2) | (chunk[1] >> 4);
        out.push(b0);
        if chunk[2] != 64 {
            let b1 = ((chunk[1] & 0x0f) << 4) | (chunk[2] >> 2);
            out.push(b1);
        }
        if chunk[3] != 64 {
            let b2 = ((chunk[2] & 0x03) << 6) | chunk[3];
            out.push(b2);
        }
        Ok(())
    }
}
