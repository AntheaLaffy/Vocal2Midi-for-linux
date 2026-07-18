//! Heuristic slicer policy compatibility helpers.
//!
//! This module mirrors the deterministic orchestration layer from
//! `inference/API/slicer_api.py::heuristic_slice`. Python remains the runtime
//! owner for production slicing, pitch/RMVPE smart slicing, audio IO,
//! multiprocessing, CLI parsing, and UI/web callers.

use crate::slicer_default::{Slicer, SlicerDefaultError};
use crate::slicer_segment::{Segment, Waveform, merge_short_segments, merge_tiny_chunks};
use crate::slicer_window::{SlicerWindowError, sliding_window_split};

/// Caller-provided heuristic slicer parameters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HeuristicConfig {
    /// The min len sec.
    pub min_len_sec: f64,
    /// The max len sec.
    pub max_len_sec: f64,
    /// The silence removal threshold db.
    pub silence_removal_threshold_db: f64,
    /// The min silence len ms.
    pub min_silence_len_ms: f64,
    /// The split threshold db.
    pub split_threshold_db: f64,
    /// The ultra short sec.
    pub ultra_short_sec: f64,
}

impl Default for HeuristicConfig {
    fn default() -> Self {
        Self {
            min_len_sec: 8.0,
            max_len_sec: 22.0,
            silence_removal_threshold_db: -40.0,
            min_silence_len_ms: 800.0,
            split_threshold_db: -30.0,
            ultra_short_sec: 0.35,
        }
    }
}

impl HeuristicConfig {
    /// Returns the legacy pre-slicer construction parameters.
    pub fn pre_slicer_params(self, sample_rate: f64) -> PreSlicerParams {
        PreSlicerParams {
            sample_rate,
            threshold_db: self.silence_removal_threshold_db,
            min_length_ms: self.min_silence_len_ms,
            min_interval_ms: 200.0,
            max_sil_kept_ms: 100.0,
        }
    }

    fn split_request(self, sample_rate: f64) -> SplitRequest {
        SplitRequest {
            sample_rate,
            min_len_sec: self.min_len_sec,
            max_len_sec: self.max_len_sec,
            target_threshold_db: self.split_threshold_db,
            frame_length: 2048,
            hop_length: 512,
        }
    }
}

/// Legacy `Slicer` construction parameters used by `heuristic_slice`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PreSlicerParams {
    /// The sample rate in hertz.
    pub sample_rate: f64,
    /// The threshold db.
    pub threshold_db: f64,
    /// The min length ms.
    pub min_length_ms: f64,
    /// The min interval ms.
    pub min_interval_ms: f64,
    /// The max sil kept ms.
    pub max_sil_kept_ms: f64,
}

/// Parameters passed to the sliding-window split dependency.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SplitRequest {
    /// The sample rate in hertz.
    pub sample_rate: f64,
    /// The min len sec.
    pub min_len_sec: f64,
    /// The max len sec.
    pub max_len_sec: f64,
    /// The target threshold db.
    pub target_threshold_db: f64,
    /// The frame length.
    pub frame_length: usize,
    /// The hop length.
    pub hop_length: usize,
}

/// Error produced by fixture-bound heuristic slicer helpers.
#[derive(Debug, Clone, PartialEq)]
pub enum SlicerHeuristicError {
    /// Carries the Python-compatible default value.
    Default(SlicerDefaultError),
    /// Carries the Python-compatible window value.
    Window(SlicerWindowError),
}

impl std::fmt::Display for SlicerHeuristicError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Default(error) => write!(f, "default slicer error: {error}"),
            Self::Window(error) => write!(f, "sliding-window error: {error}"),
        }
    }
}

impl std::error::Error for SlicerHeuristicError {}

impl From<SlicerDefaultError> for SlicerHeuristicError {
    fn from(value: SlicerDefaultError) -> Self {
        Self::Default(value)
    }
}

impl From<SlicerWindowError> for SlicerHeuristicError {
    fn from(value: SlicerWindowError) -> Self {
        Self::Window(value)
    }
}

/// Runs heuristic slicing by composing the verified default slicer,
/// sliding-window splitter, and merge helpers.
///
/// # Errors
///
/// Returns dependency errors from the default slicer or sliding-window splitter.
pub fn heuristic_slice(
    waveform: &Waveform,
    sample_rate: f64,
    config: HeuristicConfig,
) -> Result<Vec<Segment>, SlicerHeuristicError> {
    let pre = config.pre_slicer_params(sample_rate);
    let slicer = Slicer::new(
        pre.sample_rate,
        pre.threshold_db,
        pre.min_length_ms,
        pre.min_interval_ms,
        20.0,
        pre.max_sil_kept_ms,
    )?;
    let vocal_segments: Vec<Segment> = slicer
        .slice(waveform)?
        .into_iter()
        .map(|chunk| Segment {
            offset: chunk.offset,
            waveform: chunk.waveform,
        })
        .collect();
    apply_heuristic_policy(&vocal_segments, sample_rate, config, |segment, request| {
        Ok(sliding_window_split(
            &segment.waveform,
            request.sample_rate,
            request.min_len_sec,
            request.max_len_sec,
            request.target_threshold_db,
            request.frame_length,
            request.hop_length,
        )?)
    })
}

/// Applies the heuristic policy over dependency-provided vocal segments.
///
/// # Errors
///
/// Returns an error if the supplied sliding-window dependency fails.
pub fn apply_heuristic_policy<F>(
    vocal_segments: &[Segment],
    sample_rate: f64,
    config: HeuristicConfig,
    mut split_long_segment: F,
) -> Result<Vec<Segment>, SlicerHeuristicError>
where
    F: FnMut(&Segment, &SplitRequest) -> Result<Vec<Segment>, SlicerHeuristicError>,
{
    let split_request = config.split_request(sample_rate);
    let mut final_chunks = Vec::new();

    for segment in vocal_segments {
        let segment_duration = legacy_len_duration_sec(segment, sample_rate);
        if segment_duration > config.max_len_sec {
            for mut sub_chunk in split_long_segment(segment, &split_request)? {
                sub_chunk.offset += segment.offset;
                final_chunks.push(sub_chunk);
            }
        } else {
            final_chunks.push(segment.clone());
        }
    }

    final_chunks.sort_by(|left, right| left.offset.total_cmp(&right.offset));
    final_chunks = merge_tiny_chunks(&final_chunks, sample_rate, config.ultra_short_sec);

    if !final_chunks.is_empty() {
        final_chunks = merge_short_segments(
            &final_chunks,
            sample_rate,
            config.min_len_sec,
            config.max_len_sec,
        );
    }

    Ok(final_chunks)
}

fn legacy_len_duration_sec(segment: &Segment, sample_rate: f64) -> f64 {
    segment.waveform.outer_len() as f64 / sample_rate
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    const FIXTURES: &str = include_str!("../../../../fixtures/slicer_heuristic_policy_core.jsonl");

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

    fn parse_segment(value: &Value) -> Segment {
        Segment {
            offset: value["offset"].as_f64().unwrap(),
            waveform: parse_waveform(&value["waveform"]),
        }
    }

    fn parse_segments(value: &Value) -> Vec<Segment> {
        value
            .as_array()
            .unwrap()
            .iter()
            .map(parse_segment)
            .collect()
    }

    fn parse_config(value: &Value) -> HeuristicConfig {
        HeuristicConfig {
            min_len_sec: value["min_len_sec"].as_f64().unwrap(),
            max_len_sec: value["max_len_sec"].as_f64().unwrap(),
            silence_removal_threshold_db: value["silence_removal_threshold_db"].as_f64().unwrap(),
            min_silence_len_ms: value["min_silence_len_ms"].as_f64().unwrap(),
            split_threshold_db: value["split_threshold_db"].as_f64().unwrap(),
            ultra_short_sec: value["ultra_short_sec"].as_f64().unwrap(),
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

    fn encode_segment(segment: &Segment) -> Value {
        json!({
            "offset": segment.offset,
            "waveform": encode_waveform(&segment.waveform),
        })
    }

    fn encode_segments(segments: &[Segment]) -> Value {
        Value::Array(segments.iter().map(encode_segment).collect())
    }

    fn encode_pre_slicer_call(params: PreSlicerParams) -> Value {
        json!({
            "sr": params.sample_rate,
            "threshold": params.threshold_db,
            "min_length": params.min_length_ms,
            "min_interval": params.min_interval_ms,
            "max_sil_kept": params.max_sil_kept_ms,
        })
    }

    fn encode_split_call(segment: &Segment, request: &SplitRequest) -> Value {
        json!({
            "waveform": encode_waveform(&segment.waveform),
            "sr": request.sample_rate,
            "min_len_sec": request.min_len_sec,
            "max_len_sec": request.max_len_sec,
            "target_threshold_db": request.target_threshold_db,
            "frame_length": request.frame_length,
            "hop_length": request.hop_length,
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

    #[test]
    fn slicer_heuristic_policy_follows_parity_fixture_table() {
        for (line_index, line) in FIXTURES.lines().enumerate() {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let case: Value = serde_json::from_str(line).unwrap();
            let case_id = case["case_id"].as_str().unwrap();
            let config = parse_config(&case["config"]);
            let sample_rate = case["sr"].as_f64().unwrap();
            let pre_segments = parse_segments(&case["pre_segments"]);
            let split_outputs: Vec<Vec<Segment>> = case["split_outputs"]
                .as_array()
                .unwrap()
                .iter()
                .map(parse_segments)
                .collect();
            let mut split_iter = split_outputs.into_iter();
            let mut split_calls = Vec::new();

            let chunks =
                apply_heuristic_policy(&pre_segments, sample_rate, config, |segment, request| {
                    split_calls.push(encode_split_call(segment, request));
                    Ok(split_iter
                        .next()
                        .unwrap_or_else(|| panic!("{case_id}: unexpected split call")))
                })
                .unwrap();
            assert!(
                split_iter.next().is_none(),
                "{case_id}: unused split outputs"
            );

            let actual = json!({
                "chunks": encode_segments(&chunks),
                "slicer_calls": [encode_pre_slicer_call(config.pre_slicer_params(sample_rate))],
                "split_calls": split_calls,
            });
            assert_json_close(
                &actual,
                &case["expect"],
                &format!("{case_id} fixture line {}", line_index + 1),
            );
        }
    }

    #[test]
    fn heuristic_config_defaults_match_python_signature() {
        let config = HeuristicConfig::default();
        assert_eq!(config.min_len_sec, 8.0);
        assert_eq!(config.max_len_sec, 22.0);
        assert_eq!(config.silence_removal_threshold_db, -40.0);
        assert_eq!(config.min_silence_len_ms, 800.0);
        assert_eq!(config.split_threshold_db, -30.0);
        assert_eq!(config.ultra_short_sec, 0.35);
    }

    #[test]
    fn heuristic_slice_composes_verified_dependencies() {
        let waveform = Waveform::Mono(vec![1.0; 100]);
        let config = HeuristicConfig {
            min_len_sec: 0.2,
            max_len_sec: 1.0,
            silence_removal_threshold_db: -40.0,
            min_silence_len_ms: 200.0,
            split_threshold_db: -30.0,
            ultra_short_sec: 0.0,
        };

        let chunks = heuristic_slice(&waveform, 1000.0, config).unwrap();

        assert_eq!(
            encode_segments(&chunks),
            json!([{"offset": 0.0, "waveform": vec![1.0; 100]}])
        );
    }
}
