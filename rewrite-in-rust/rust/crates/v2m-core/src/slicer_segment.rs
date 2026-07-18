//! Slicer segment merge helpers.
//!
//! This module mirrors deterministic segment waveform manipulation helpers from
//! `inference/API/slicer_api.py`. Python remains the runtime owner for real
//! audio loading, RMS/default slicing, heuristic/grid/pitch slicing policies,
//! RMVPE/ASR execution, multiprocessing, CLI parsing, and filesystem writes.

/// Synthetic waveform payload used by segment-merge parity fixtures.
#[derive(Debug, Clone, PartialEq)]
pub enum Waveform {
    /// Carries the Python-compatible mono value.
    Mono(Vec<f64>),
    /// Carries the Python-compatible stereo value.
    Stereo(Vec<Vec<f64>>),
}

impl Waveform {
    /// Returns the sample length used by `waveform.shape[-1]` in Python.
    pub fn sample_len(&self) -> usize {
        match self {
            Self::Mono(samples) => samples.len(),
            Self::Stereo(channels) => channels.first().map_or(0, Vec::len),
        }
    }

    /// Returns the channel count used by Python `len(waveform)`.
    pub fn outer_len(&self) -> usize {
        match self {
            Self::Mono(samples) => samples.len(),
            Self::Stereo(channels) => channels.len(),
        }
    }

    /// Returns true when this payload follows the multi-dimensional concat path.
    pub const fn is_stereo(&self) -> bool {
        matches!(self, Self::Stereo(_))
    }
}

/// One segment dictionary accepted by legacy merge helpers.
#[derive(Debug, Clone, PartialEq)]
pub struct Segment {
    /// The offset.
    pub offset: f64,
    /// The waveform.
    pub waveform: Waveform,
}

/// Concatenates waveforms with the same axis choice as Python `_concat_waveforms`.
///
/// # Panics
///
/// Panics when one waveform is mono and the other is stereo.
pub fn concat_waveforms(left: &Waveform, right: &Waveform) -> Waveform {
    match (left, right) {
        (Waveform::Mono(left), Waveform::Mono(right)) => {
            let mut output = left.clone();
            output.extend_from_slice(right);
            Waveform::Mono(output)
        }
        (Waveform::Stereo(left), Waveform::Stereo(right)) => {
            let mut output = left.clone();
            for (channel_index, right_channel) in right.iter().enumerate() {
                if let Some(left_channel) = output.get_mut(channel_index) {
                    left_channel.extend_from_slice(right_channel);
                } else {
                    output.push(right_channel.clone());
                }
            }
            Waveform::Stereo(output)
        }
        _ => panic!("waveform dimensionality mismatch"),
    }
}

/// Creates silence with the same shape behavior as Python `_silence_like`.
pub fn silence_like(waveform: &Waveform, samples: i64) -> Waveform {
    if samples <= 0 {
        return match waveform {
            Waveform::Mono(_) => Waveform::Mono(Vec::new()),
            Waveform::Stereo(channels) => Waveform::Stereo(vec![Vec::new(); channels.len()]),
        };
    }

    let samples = samples as usize;
    match waveform {
        Waveform::Mono(_) => Waveform::Mono(vec![0.0; samples]),
        Waveform::Stereo(channels) => Waveform::Stereo(vec![vec![0.0; samples]; channels.len()]),
    }
}

/// Returns segment duration in seconds using Python `shape[-1] / sr` behavior.
pub fn segment_duration_sec(segment: &Segment, sample_rate: f64) -> f64 {
    segment.waveform.sample_len() as f64 / sample_rate
}

/// Returns merged duration in seconds without allowing negative gaps.
pub fn merged_duration_sec(left: &Segment, right: &Segment, sample_rate: f64) -> f64 {
    let left_end = left.offset + segment_duration_sec(left, sample_rate);
    let gap = 0.0_f64.max(right.offset - left_end);
    segment_duration_sec(left, sample_rate) + gap + segment_duration_sec(right, sample_rate)
}

/// Merges two adjacent segments while preserving the left offset and timeline gap.
pub fn merge_segments(left: &Segment, right: &Segment, sample_rate: f64) -> Segment {
    let left_end = left.offset + segment_duration_sec(left, sample_rate);
    let gap_samples = 0_i64.max(round_half_even_i64((right.offset - left_end) * sample_rate));
    let gap = silence_like(&left.waveform, gap_samples);
    Segment {
        offset: left.offset,
        waveform: concat_waveforms(&concat_waveforms(&left.waveform, &gap), &right.waveform),
    }
}

/// Greedily merges adjacent short segments toward the target duration range.
///
/// # Panics
///
/// Panics only if the internal merged list becomes empty after initialization
/// from a non-empty `chunks` input.
pub fn merge_short_segments(
    chunks: &[Segment],
    sample_rate: f64,
    min_len_sec: f64,
    max_len_sec: f64,
) -> Vec<Segment> {
    if chunks.is_empty() {
        return Vec::new();
    }

    let mut merged = vec![chunks[0].clone()];
    for next in &chunks[1..] {
        let current = merged.last().unwrap();
        let current_duration = segment_duration_sec(current, sample_rate);
        let combined_duration = merged_duration_sec(current, next, sample_rate);

        if current_duration < min_len_sec && combined_duration <= max_len_sec {
            let merged_segment = merge_segments(current, next, sample_rate);
            *merged.last_mut().unwrap() = merged_segment;
        } else {
            merged.push(next.clone());
        }
    }

    if merged.len() >= 2 {
        let last_index = merged.len() - 1;
        let last_duration = segment_duration_sec(&merged[last_index], sample_rate);
        if last_duration < min_len_sec - 2.0 {
            let combined_duration =
                merged_duration_sec(&merged[last_index - 1], &merged[last_index], sample_rate);
            if combined_duration <= max_len_sec {
                let merged_segment =
                    merge_segments(&merged[last_index - 1], &merged[last_index], sample_rate);
                merged[last_index - 1] = merged_segment;
                merged.pop();
            }
        }
    }

    let any_short = merged
        .iter()
        .any(|segment| segment_duration_sec(segment, sample_rate) < min_len_sec);
    if any_short && merged.len() < chunks.len() {
        return merge_short_segments(&merged, sample_rate, min_len_sec, max_len_sec);
    }

    merged
}

/// Merges tiny chunks into neighboring chunks without discarding isolated tails.
pub fn merge_tiny_chunks(chunks: &[Segment], sample_rate: f64, tiny_sec: f64) -> Vec<Segment> {
    if chunks.is_empty() {
        return Vec::new();
    }

    let mut merged = Vec::new();
    let mut pending_head: Option<Segment> = None;

    for segment in chunks {
        let segment_duration = segment.waveform.outer_len() as f64 / sample_rate;
        if segment_duration < tiny_sec {
            if let Some(previous) = merged.last_mut() {
                *previous = merge_segments(previous, segment, sample_rate);
            } else if let Some(pending) = pending_head.take() {
                pending_head = Some(merge_segments(&pending, segment, sample_rate));
            } else {
                pending_head = Some(segment.clone());
            }
            continue;
        }

        let mut segment = segment.clone();
        if let Some(pending) = pending_head.take() {
            segment = merge_segments(&pending, &segment, sample_rate);
        }
        merged.push(segment);
    }

    if let Some(pending) = pending_head {
        merged.push(pending);
    }

    merged
}

fn round_half_even_i64(value: f64) -> i64 {
    let floor = value.floor();
    let diff = value - floor;
    let rounded = if diff < 0.5 {
        floor
    } else if diff > 0.5 {
        floor + 1.0
    } else if (floor as i128).rem_euclid(2) == 0 {
        floor
    } else {
        floor + 1.0
    };
    rounded as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    const FIXTURES: &str = include_str!("../../../../fixtures/slicer_segment_merge_core.jsonl");

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
            "waveform": encode_waveform(&segment.waveform)
        })
    }

    fn encode_segments(segments: &[Segment]) -> Value {
        Value::Array(segments.iter().map(encode_segment).collect())
    }

    #[test]
    fn slicer_segment_merge_follows_parity_fixture_table() {
        for (line_index, line) in FIXTURES.lines().enumerate() {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let case: Value = serde_json::from_str(line).unwrap();
            let case_id = case["case_id"].as_str().unwrap();
            let kind = case["kind"].as_str().unwrap();
            let actual = match kind {
                "concat" => encode_waveform(&concat_waveforms(
                    &parse_waveform(&case["a"]),
                    &parse_waveform(&case["b"]),
                )),
                "silence_like" => encode_waveform(&silence_like(
                    &parse_waveform(&case["waveform"]),
                    case["samples"].as_i64().unwrap(),
                )),
                "merged_duration" => json!(merged_duration_sec(
                    &parse_segment(&case["left"]),
                    &parse_segment(&case["right"]),
                    case["sr"].as_f64().unwrap(),
                )),
                "merge_segments" => encode_segment(&merge_segments(
                    &parse_segment(&case["left"]),
                    &parse_segment(&case["right"]),
                    case["sr"].as_f64().unwrap(),
                )),
                "merge_short_segments" => encode_segments(&merge_short_segments(
                    &parse_segments(&case["chunks"]),
                    case["sr"].as_f64().unwrap(),
                    case["min_len_sec"].as_f64().unwrap(),
                    case["max_len_sec"].as_f64().unwrap(),
                )),
                "merge_tiny_chunks" => encode_segments(&merge_tiny_chunks(
                    &parse_segments(&case["chunks"]),
                    case["sr"].as_f64().unwrap(),
                    case["tiny_sec"].as_f64().unwrap(),
                )),
                _ => panic!("{case_id} fixture line {} unknown kind", line_index + 1),
            };

            let expected = match kind {
                "concat" | "silence_like" | "merged_duration" => &case["expect"],
                "merge_segments" | "merge_short_segments" | "merge_tiny_chunks" => &case["expect"],
                _ => unreachable!(),
            };
            assert_eq!(
                actual,
                *expected,
                "{case_id} fixture line {}",
                line_index + 1
            );
        }
    }

    #[test]
    fn merge_tiny_chunks_preserves_legacy_stereo_len_behavior() {
        let chunks = [
            Segment {
                offset: 0.0,
                waveform: Waveform::Stereo(vec![vec![1.0; 5], vec![10.0; 5]]),
            },
            Segment {
                offset: 0.5,
                waveform: Waveform::Stereo(vec![vec![2.0; 5], vec![20.0; 5]]),
            },
        ];

        let merged = merge_tiny_chunks(&chunks, 10.0, 0.35);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].waveform.sample_len(), 10);
    }
}
