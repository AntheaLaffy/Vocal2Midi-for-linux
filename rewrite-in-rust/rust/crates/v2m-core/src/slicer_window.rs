//! RMS-dB and sliding-window slicer compatibility helpers.
//!
//! This module mirrors deterministic `get_rms_db` and `_sliding_window_split`
//! behavior from `inference/API/slicer_api.py`. Python remains the runtime
//! owner for heuristic/grid policy orchestration, pitch/RMVPE smart slicing,
//! audio IO, multiprocessing, CLI parsing, and production routing.

use crate::slicer_default::{PadMode, get_rms};
use crate::slicer_segment::{Segment, Waveform};

/// Error produced by fixture-bound RMS/window split helpers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SlicerWindowError {
    InvalidFrame,
    EmptyStereo,
    RaggedStereo,
    MissingCutType,
}

impl SlicerWindowError {
    /// Returns the Python-compatible exception type for fixture comparisons.
    pub const fn python_type(&self) -> &'static str {
        match self {
            Self::MissingCutType => "UnboundLocalError",
            _ => "ValueError",
        }
    }

    /// Returns a compatibility message for boundary errors.
    pub fn message(&self) -> String {
        match self {
            Self::InvalidFrame => "invalid frame_length, hop_length, or sample rate".to_string(),
            Self::EmptyStereo => "stereo waveform must contain at least one channel".to_string(),
            Self::RaggedStereo => "stereo waveform channels must have equal lengths".to_string(),
            Self::MissingCutType => {
                "cannot access local variable 'cut_type' where it is not associated with a value"
                    .to_string()
            }
        }
    }
}

impl std::fmt::Display for SlicerWindowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message())
    }
}

impl std::error::Error for SlicerWindowError {}

/// Calculates RMS energy in decibels like
/// `inference/API/slicer_api.py::get_rms_db`.
///
/// # Errors
///
/// Returns an error when the waveform or frame parameters are outside the
/// fixture-bound compatibility surface.
pub fn rms_db(
    waveform: &Waveform,
    frame_length: usize,
    hop_length: usize,
) -> Result<Vec<f64>, SlicerWindowError> {
    let samples = waveform_mean_samples(waveform)?;
    let rms = get_rms(&samples, frame_length, hop_length, PadMode::Constant)
        .map_err(|_| SlicerWindowError::InvalidFrame)?;
    Ok(rms
        .into_iter()
        .map(|value| 20.0 * value.max(1e-10).log10())
        .collect())
}

/// Splits one long segment using the legacy RMS sliding-window policy.
///
/// # Errors
///
/// Returns an error for invalid frame parameters or for the legacy empty-window
/// path that raises `UnboundLocalError` before returning chunks.
pub fn sliding_window_split(
    waveform: &Waveform,
    sample_rate: f64,
    min_len_sec: f64,
    max_len_sec: f64,
    target_threshold_db: f64,
    frame_length: usize,
    hop_length: usize,
) -> Result<Vec<Segment>, SlicerWindowError> {
    if sample_rate <= 0.0 || hop_length == 0 {
        return Err(SlicerWindowError::InvalidFrame);
    }

    let total_samples = waveform.sample_len();
    let total_sec = total_samples as f64 / sample_rate;
    if total_sec <= max_len_sec {
        return Ok(vec![Segment {
            offset: 0.0,
            waveform: waveform.clone(),
        }]);
    }

    let rms_values = rms_db(waveform, frame_length, hop_length)?;
    let mut chunks = Vec::new();
    let mut current_start_sec = 0.0;

    while current_start_sec < total_sec {
        let window_start_sec = current_start_sec + min_len_sec;
        let window_end_sec = current_start_sec + max_len_sec;

        if window_end_sec >= total_sec {
            let start_sample = seconds_to_sample_index(current_start_sec, sample_rate);
            chunks.push(Segment {
                offset: current_start_sec,
                waveform: slice_waveform(waveform, start_sample, total_samples),
            });
            break;
        }

        let rms_len = rms_values.len() as i64;
        let mut start_frame = time_to_frames(window_start_sec, sample_rate, hop_length)?;
        let mut end_frame = time_to_frames(window_end_sec, sample_rate, hop_length)?;
        start_frame = 0_i64.max(start_frame.min(rms_len - 1));
        end_frame = 0_i64.max(end_frame.min(rms_len));

        if start_frame >= end_frame {
            return Err(SlicerWindowError::MissingCutType);
        }

        let window = &rms_values[start_frame as usize..end_frame as usize];
        let best_idx_in_window = latest_threshold_index(window, target_threshold_db)
            .unwrap_or_else(|| first_argmin(window));
        let cut_frame = start_frame + best_idx_in_window as i64;
        let cut_sec = frames_to_time(cut_frame, sample_rate, hop_length)?;

        let start_sample = seconds_to_sample_index(current_start_sec, sample_rate);
        let end_sample = seconds_to_sample_index(cut_sec, sample_rate);
        chunks.push(Segment {
            offset: current_start_sec,
            waveform: slice_waveform(waveform, start_sample, end_sample),
        });
        current_start_sec = cut_sec;
    }

    Ok(chunks)
}

fn waveform_mean_samples(waveform: &Waveform) -> Result<Vec<f64>, SlicerWindowError> {
    match waveform {
        Waveform::Mono(samples) => Ok(samples.clone()),
        Waveform::Stereo(channels) => {
            let Some(first) = channels.first() else {
                return Err(SlicerWindowError::EmptyStereo);
            };
            if channels.iter().any(|channel| channel.len() != first.len()) {
                return Err(SlicerWindowError::RaggedStereo);
            }
            let mut output = vec![0.0; first.len()];
            for channel in channels {
                for (sample_index, sample) in channel.iter().enumerate() {
                    output[sample_index] += *sample;
                }
            }
            for sample in &mut output {
                *sample /= channels.len() as f64;
            }
            Ok(output)
        }
    }
}

fn latest_threshold_index(values: &[f64], threshold: f64) -> Option<usize> {
    values
        .iter()
        .enumerate()
        .filter_map(|(index, value)| (*value < threshold).then_some(index))
        .next_back()
}

fn first_argmin(values: &[f64]) -> usize {
    values
        .iter()
        .enumerate()
        .min_by(|(_, left), (_, right)| left.total_cmp(right))
        .map(|(index, _)| index)
        .unwrap_or(0)
}

fn time_to_frames(
    seconds: f64,
    sample_rate: f64,
    hop_length: usize,
) -> Result<i64, SlicerWindowError> {
    if !seconds.is_finite() || !sample_rate.is_finite() || sample_rate <= 0.0 || hop_length == 0 {
        return Err(SlicerWindowError::InvalidFrame);
    }
    let samples = (seconds * sample_rate).trunc() as i64;
    Ok(samples.div_euclid(hop_length as i64))
}

fn frames_to_time(
    frame: i64,
    sample_rate: f64,
    hop_length: usize,
) -> Result<f64, SlicerWindowError> {
    if !sample_rate.is_finite() || sample_rate <= 0.0 || hop_length == 0 {
        return Err(SlicerWindowError::InvalidFrame);
    }
    Ok(frame as f64 * hop_length as f64 / sample_rate)
}

fn seconds_to_sample_index(seconds: f64, sample_rate: f64) -> usize {
    let samples = seconds * sample_rate;
    if !samples.is_finite() || samples <= 0.0 {
        0
    } else {
        samples.trunc() as usize
    }
}

fn slice_waveform(waveform: &Waveform, begin: usize, end: usize) -> Waveform {
    match waveform {
        Waveform::Mono(samples) => {
            let end = end.min(samples.len());
            Waveform::Mono(samples[begin.min(end)..end].to_vec())
        }
        Waveform::Stereo(channels) => Waveform::Stereo(
            channels
                .iter()
                .map(|channel| {
                    let end = end.min(channel.len());
                    channel[begin.min(end)..end].to_vec()
                })
                .collect(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    const FIXTURES: &str =
        include_str!("../../../../fixtures/slicer_rms_db_window_split_core.jsonl");

    fn parse_waveform(value: &Value) -> Waveform {
        let rows = value.as_array().unwrap();
        if rows.first().is_some_and(Value::is_array) {
            Waveform::Stereo(
                rows.iter()
                    .map(|row| {
                        row.as_array()
                            .unwrap()
                            .iter()
                            .map(|sample| sample.as_f64().unwrap())
                            .collect()
                    })
                    .collect(),
            )
        } else {
            Waveform::Mono(rows.iter().map(|sample| sample.as_f64().unwrap()).collect())
        }
    }

    fn encode_waveform(waveform: &Waveform) -> Value {
        match waveform {
            Waveform::Mono(samples) => {
                Value::Array(samples.iter().map(|sample| json!(sample)).collect())
            }
            Waveform::Stereo(channels) => Value::Array(
                channels
                    .iter()
                    .map(|channel| {
                        Value::Array(channel.iter().map(|sample| json!(sample)).collect())
                    })
                    .collect(),
            ),
        }
    }

    fn encode_chunks(chunks: &[Segment]) -> Value {
        Value::Array(
            chunks
                .iter()
                .map(|chunk| {
                    json!({
                        "offset": chunk.offset,
                        "waveform": encode_waveform(&chunk.waveform),
                    })
                })
                .collect(),
        )
    }

    fn encode_error(error: &SlicerWindowError) -> Value {
        json!({
            "type": error.python_type(),
            "message": error.message(),
        })
    }

    fn assert_json_close(actual: &Value, expected: &Value, context: &str) {
        match (actual, expected) {
            (Value::Number(left), Value::Number(right)) => {
                let left = left.as_f64().unwrap();
                let right = right.as_f64().unwrap();
                assert!(
                    (left - right).abs() <= 1e-6,
                    "{context}: {left:?} != {right:?}"
                );
            }
            (Value::Array(left), Value::Array(right)) => {
                assert_eq!(left.len(), right.len(), "{context}: array lengths differ");
                for (index, (left_item, right_item)) in left.iter().zip(right).enumerate() {
                    assert_json_close(left_item, right_item, &format!("{context}[{index}]"));
                }
            }
            (Value::Object(left), Value::Object(right)) => {
                assert_eq!(left.len(), right.len(), "{context}: object lengths differ");
                for (key, right_value) in right {
                    let left_value = left
                        .get(key)
                        .unwrap_or_else(|| panic!("{context}: missing {key}"));
                    assert_json_close(left_value, right_value, &format!("{context}.{key}"));
                }
            }
            _ => assert_eq!(actual, expected, "{context}"),
        }
    }

    fn split_from_case(case: &Value) -> Result<Vec<Segment>, SlicerWindowError> {
        sliding_window_split(
            &parse_waveform(&case["waveform"]),
            case["sr"].as_f64().unwrap(),
            case["min_len_sec"].as_f64().unwrap(),
            case["max_len_sec"].as_f64().unwrap(),
            case["target_threshold_db"].as_f64().unwrap(),
            case["frame_length"].as_u64().unwrap() as usize,
            case["hop_length"].as_u64().unwrap() as usize,
        )
    }

    #[test]
    fn slicer_window_follows_parity_fixture_table() {
        for (line_index, line) in FIXTURES.lines().enumerate() {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let case: Value = serde_json::from_str(line).unwrap();
            let case_id = case["case_id"].as_str().unwrap();
            let kind = case["kind"].as_str().unwrap();
            let actual = match kind {
                "rms_db" => {
                    let values = rms_db(
                        &parse_waveform(&case["waveform"]),
                        case["frame_length"].as_u64().unwrap() as usize,
                        case["hop_length"].as_u64().unwrap() as usize,
                    )
                    .unwrap();
                    Value::Array(values.iter().map(|value| json!(value)).collect())
                }
                "sliding_window_split" => encode_chunks(&split_from_case(&case).unwrap()),
                "sliding_window_error" => encode_error(&split_from_case(&case).unwrap_err()),
                _ => panic!("{case_id} fixture line {} unknown kind", line_index + 1),
            };

            assert_json_close(
                &actual,
                &case["expect"],
                &format!("{case_id} fixture line {}", line_index + 1),
            );
        }
    }

    #[test]
    fn time_frame_conversion_matches_librosa_scalar_path() {
        assert_eq!(time_to_frames(0.1, 10.0, 10).unwrap(), 0);
        assert_eq!(time_to_frames(0.6, 10.0, 1).unwrap(), 6);
        assert!((frames_to_time(4, 10.0, 1).unwrap() - 0.4).abs() < 1e-12);
    }
}
