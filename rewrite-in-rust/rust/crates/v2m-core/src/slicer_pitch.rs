//! Supplied-voiced-mask pitch slicer compatibility helpers.
//!
//! This module mirrors the deterministic supplied-mask path from
//! `inference/API/slicer_api.py::get_pitch_curve`, `_pitch_based_split`, and
//! `pitch_based_slice`. Python remains the runtime owner for `librosa.pyin`,
//! RMVPE/model execution, audio IO, multiprocessing, CLI parsing, GUI/web
//! callers, and production routing.

use crate::slicer_default::{Slicer, SlicerDefaultError};
use crate::slicer_segment::{Segment, Waveform, merge_short_segments, merge_tiny_chunks};
use crate::slicer_window::{SlicerWindowError, rms_db};

/// Caller-provided smart slicer parameters for the supplied-mask path.
#[derive(Debug, Clone, PartialEq)]
pub struct PitchOverrideConfig {
    pub min_len_sec: f64,
    pub max_len_sec: f64,
    pub silence_removal_threshold_db: f64,
    pub min_silence_len_ms: f64,
    pub ultra_short_sec: f64,
    pub voiced_flag_override: Option<Vec<bool>>,
    pub voiced_flag_override_step_sec: Option<f64>,
}

impl Default for PitchOverrideConfig {
    fn default() -> Self {
        Self {
            min_len_sec: 8.0,
            max_len_sec: 22.0,
            silence_removal_threshold_db: -40.0,
            min_silence_len_ms: 800.0,
            ultra_short_sec: 0.35,
            voiced_flag_override: None,
            voiced_flag_override_step_sec: None,
        }
    }
}

impl PitchOverrideConfig {
    /// Returns the legacy pre-slicer construction parameters.
    pub fn pre_slicer_params(&self, sample_rate: f64) -> PreSlicerParams {
        PreSlicerParams {
            sample_rate,
            threshold_db: self.silence_removal_threshold_db,
            min_length_ms: self.min_silence_len_ms,
            min_interval_ms: 200.0,
            max_sil_kept_ms: 100.0,
        }
    }

    fn split_request(&self, sample_rate: f64, segment_offset_sec: f64) -> PitchSplitRequest {
        PitchSplitRequest {
            sample_rate,
            min_len_sec: self.min_len_sec,
            max_len_sec: self.max_len_sec,
            hop_length: 512,
            voiced_flag_override: self.voiced_flag_override.clone(),
            voiced_flag_override_step_sec: self.voiced_flag_override_step_sec,
            segment_offset_sec,
        }
    }
}

/// Legacy `Slicer` construction parameters used by `pitch_based_slice`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PreSlicerParams {
    pub sample_rate: f64,
    pub threshold_db: f64,
    pub min_length_ms: f64,
    pub min_interval_ms: f64,
    pub max_sil_kept_ms: f64,
}

/// Parameters passed from the outer smart-slicing policy into the split helper.
#[derive(Debug, Clone, PartialEq)]
pub struct PitchSplitRequest {
    pub sample_rate: f64,
    pub min_len_sec: f64,
    pub max_len_sec: f64,
    pub hop_length: usize,
    pub voiced_flag_override: Option<Vec<bool>>,
    pub voiced_flag_override_step_sec: Option<f64>,
    pub segment_offset_sec: f64,
}

/// Error produced by fixture-bound supplied-mask pitch slicer helpers.
#[derive(Debug, Clone, PartialEq)]
pub enum SlicerPitchError {
    InvalidFrame,
    EmptyVoicedMask,
    PyinUnsupported,
    Default(SlicerDefaultError),
    Window(SlicerWindowError),
}

impl std::fmt::Display for SlicerPitchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidFrame => f.write_str("invalid frame, sample rate, or hop length"),
            Self::EmptyVoicedMask => f.write_str("voiced mask must not be empty"),
            Self::PyinUnsupported => f.write_str("librosa.pyin path is legacy-owned"),
            Self::Default(error) => write!(f, "default slicer error: {error}"),
            Self::Window(error) => write!(f, "RMS fallback error: {error}"),
        }
    }
}

impl std::error::Error for SlicerPitchError {}

impl From<SlicerDefaultError> for SlicerPitchError {
    fn from(value: SlicerDefaultError) -> Self {
        Self::Default(value)
    }
}

impl From<SlicerWindowError> for SlicerPitchError {
    fn from(value: SlicerWindowError) -> Self {
        Self::Window(value)
    }
}

/// Resamples a supplied voiced-mask override using the legacy global-time
/// round-and-clip policy.
///
/// # Errors
///
/// Returns an error when frame parameters are unusable or the supplied mask is
/// empty.
pub fn resample_voiced_override(
    waveform: &Waveform,
    sample_rate: f64,
    hop_length: usize,
    voiced_flag_override: &[bool],
    voiced_flag_override_step_sec: f64,
    segment_offset_sec: f64,
) -> Result<Vec<bool>, SlicerPitchError> {
    if !sample_rate.is_finite()
        || sample_rate <= 0.0
        || hop_length == 0
        || !voiced_flag_override_step_sec.is_finite()
        || voiced_flag_override_step_sec <= 0.0
    {
        return Err(SlicerPitchError::InvalidFrame);
    }
    if voiced_flag_override.is_empty() {
        return Err(SlicerPitchError::EmptyVoicedMask);
    }

    let target_frames = (waveform.sample_len() as f64 / hop_length as f64).ceil() as usize + 1;
    let mut output = Vec::with_capacity(target_frames);
    for frame_index in 0..target_frames {
        let time = frame_index as f64 * (hop_length as f64 / sample_rate) + segment_offset_sec;
        let source_index = round_half_even_i64(time / voiced_flag_override_step_sec)
            .clamp(0, voiced_flag_override.len() as i64 - 1) as usize;
        output.push(voiced_flag_override[source_index]);
    }
    Ok(output)
}

/// Splits one long segment using supplied voiced-mask information and an RMS
/// fallback provider.
///
/// # Errors
///
/// Returns dependency or boundary errors from supplied-mask resampling or the
/// RMS fallback provider.
pub fn pitch_based_split_with_override<F>(
    waveform: &Waveform,
    request: &PitchSplitRequest,
    rms_provider: F,
) -> Result<Vec<Segment>, SlicerPitchError>
where
    F: FnMut(&Waveform, usize, usize) -> Result<Vec<f64>, SlicerPitchError>,
{
    let sample_rate = request.sample_rate;
    if !sample_rate.is_finite() || sample_rate <= 0.0 {
        return Err(SlicerPitchError::InvalidFrame);
    }

    let total_sec = waveform.sample_len() as f64 / sample_rate;
    if total_sec <= request.max_len_sec {
        return Ok(vec![Segment {
            offset: 0.0,
            waveform: waveform.clone(),
        }]);
    }

    let hop_length = request.hop_length;
    let voiced_flag = match (
        request.voiced_flag_override.as_deref(),
        request.voiced_flag_override_step_sec,
    ) {
        (Some(flags), Some(step)) if step > 0.0 => resample_voiced_override(
            waveform,
            sample_rate,
            hop_length,
            flags,
            step,
            request.segment_offset_sec,
        )?,
        _ => return Err(SlicerPitchError::PyinUnsupported),
    };
    pitch_based_split_with_voiced_flags(waveform, request, &voiced_flag, rms_provider)
}

fn pitch_based_split_with_voiced_flags<F>(
    waveform: &Waveform,
    request: &PitchSplitRequest,
    voiced_flag: &[bool],
    mut rms_provider: F,
) -> Result<Vec<Segment>, SlicerPitchError>
where
    F: FnMut(&Waveform, usize, usize) -> Result<Vec<f64>, SlicerPitchError>,
{
    let sample_rate = request.sample_rate;
    let hop_length = request.hop_length;
    if !sample_rate.is_finite() || sample_rate <= 0.0 || hop_length == 0 {
        return Err(SlicerPitchError::InvalidFrame);
    }

    let mut chunks = Vec::new();
    let total_samples = waveform.sample_len();
    let total_sec = total_samples as f64 / sample_rate;
    if total_sec <= request.max_len_sec {
        return Ok(vec![Segment {
            offset: 0.0,
            waveform: waveform.clone(),
        }]);
    }
    let mut current_start_sec = 0.0;

    while current_start_sec < total_sec {
        let window_start_sec = current_start_sec + request.min_len_sec;
        let window_end_sec = current_start_sec + request.max_len_sec;

        if window_end_sec >= total_sec {
            let start_sample = seconds_to_sample_index(current_start_sec, sample_rate);
            chunks.push(Segment {
                offset: current_start_sec,
                waveform: slice_waveform(waveform, start_sample, total_samples),
            });
            break;
        }

        let voiced_len = voiced_flag.len() as i64;
        let mut start_frame = time_to_frames(window_start_sec, sample_rate, hop_length)?;
        let mut end_frame = time_to_frames(window_end_sec, sample_rate, hop_length)?;
        start_frame = 0_i64.max(start_frame.min(voiced_len - 1));
        end_frame = 0_i64.max(end_frame.min(voiced_len));

        let mut cut_frame = -1_i64;
        if start_frame < end_frame {
            let window = &voiced_flag[start_frame as usize..end_frame as usize];
            if let Some(cut_idx_in_window) = longest_unvoiced_midpoint(window) {
                cut_frame = start_frame + cut_idx_in_window as i64;
            }
        }

        if cut_frame == -1 {
            let rms_values = rms_provider(waveform, hop_length * 4, hop_length)?;
            let rms_len = rms_values.len() as i64;
            let mut start_frame_rms = time_to_frames(window_start_sec, sample_rate, hop_length)?;
            let mut end_frame_rms = time_to_frames(window_end_sec, sample_rate, hop_length)?;
            start_frame_rms = 0_i64.max(start_frame_rms.min(rms_len - 1));
            end_frame_rms = 0_i64.max(end_frame_rms.min(rms_len));

            if start_frame_rms < end_frame_rms {
                let window = &rms_values[start_frame_rms as usize..end_frame_rms as usize];
                cut_frame = start_frame_rms + first_argmin(window) as i64;
            } else {
                cut_frame = end_frame;
            }
        }

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

/// Runs supplied-mask smart slicing by composing verified default Slicer, RMS
/// fallback, and merge helper dependencies.
///
/// # Errors
///
/// Returns dependency or boundary errors from the composed helpers.
pub fn pitch_based_slice(
    waveform: &Waveform,
    sample_rate: f64,
    config: &PitchOverrideConfig,
) -> Result<Vec<Segment>, SlicerPitchError> {
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

    apply_pitch_override_policy(&vocal_segments, sample_rate, config, |segment, request| {
        pitch_based_split_with_override(
            &segment.waveform,
            request,
            |waveform, frame_length, hop_length| Ok(rms_db(waveform, frame_length, hop_length)?),
        )
    })
}

/// Applies outer supplied-mask smart-slicing policy over dependency-provided
/// vocal segments.
///
/// # Errors
///
/// Returns an error if the supplied long-segment splitter fails.
pub fn apply_pitch_override_policy<F>(
    vocal_segments: &[Segment],
    sample_rate: f64,
    config: &PitchOverrideConfig,
    mut split_long_segment: F,
) -> Result<Vec<Segment>, SlicerPitchError>
where
    F: FnMut(&Segment, &PitchSplitRequest) -> Result<Vec<Segment>, SlicerPitchError>,
{
    let mut short_segments = Vec::new();
    let mut long_segments = Vec::new();

    for segment in vocal_segments {
        let segment_duration = segment.waveform.outer_len() as f64 / sample_rate;
        if segment_duration > config.max_len_sec {
            long_segments.push(segment.clone());
        } else {
            short_segments.push(segment.clone());
        }
    }

    let mut final_chunks = if long_segments.is_empty() {
        short_segments
    } else {
        let mut processed_chunks = Vec::new();
        for segment in &long_segments {
            let request = config.split_request(sample_rate, segment.offset);
            for mut sub_chunk in split_long_segment(segment, &request)? {
                sub_chunk.offset += segment.offset;
                processed_chunks.push(sub_chunk);
            }
        }
        let mut chunks = short_segments;
        chunks.extend(processed_chunks);
        chunks.sort_by(|left, right| left.offset.total_cmp(&right.offset));
        chunks
    };

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

fn longest_unvoiced_midpoint(values: &[bool]) -> Option<usize> {
    let mut best_start = None;
    let mut best_len = 0usize;
    let mut index = 0usize;

    while index < values.len() {
        if values[index] {
            index += 1;
            continue;
        }

        let start = index;
        while index < values.len() && !values[index] {
            index += 1;
        }
        let len = index - start;
        if len > best_len {
            best_len = len;
            best_start = Some(start);
        }
    }

    best_start.map(|start| start + best_len / 2)
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
) -> Result<i64, SlicerPitchError> {
    if !seconds.is_finite() || !sample_rate.is_finite() || sample_rate <= 0.0 || hop_length == 0 {
        return Err(SlicerPitchError::InvalidFrame);
    }
    let samples = (seconds * sample_rate).trunc() as i64;
    Ok(samples.div_euclid(hop_length as i64))
}

fn frames_to_time(
    frame: i64,
    sample_rate: f64,
    hop_length: usize,
) -> Result<f64, SlicerPitchError> {
    if !sample_rate.is_finite() || sample_rate <= 0.0 || hop_length == 0 {
        return Err(SlicerPitchError::InvalidFrame);
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

    const FIXTURES: &str = include_str!("../../../../fixtures/slicer_pitch_override_core.jsonl");

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

    fn parse_bool_vec(value: &Value) -> Vec<bool> {
        value
            .as_array()
            .unwrap()
            .iter()
            .map(|item| item.as_bool().unwrap())
            .collect()
    }

    fn parse_optional_bool_vec(value: &Value) -> Option<Vec<bool>> {
        if value.is_null() {
            None
        } else {
            Some(parse_bool_vec(value))
        }
    }

    fn parse_optional_f64(value: &Value) -> Option<f64> {
        if value.is_null() {
            None
        } else {
            Some(value.as_f64().unwrap())
        }
    }

    fn parse_config(value: &Value) -> PitchOverrideConfig {
        PitchOverrideConfig {
            min_len_sec: value["min_len_sec"].as_f64().unwrap(),
            max_len_sec: value["max_len_sec"].as_f64().unwrap(),
            silence_removal_threshold_db: value["silence_removal_threshold_db"].as_f64().unwrap(),
            min_silence_len_ms: value["min_silence_len_ms"].as_f64().unwrap(),
            ultra_short_sec: value["ultra_short_sec"].as_f64().unwrap(),
            voiced_flag_override: parse_optional_bool_vec(&value["voiced_flag_override"]),
            voiced_flag_override_step_sec: parse_optional_f64(
                &value["voiced_flag_override_step_sec"],
            ),
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

    fn encode_split_call(segment: &Segment, request: &PitchSplitRequest) -> Value {
        json!({
            "waveform": encode_waveform(&segment.waveform),
            "sr": request.sample_rate,
            "min_len_sec": request.min_len_sec,
            "max_len_sec": request.max_len_sec,
            "hop_length": request.hop_length,
            "voiced_flag_override": request.voiced_flag_override,
            "voiced_flag_override_step_sec": request.voiced_flag_override_step_sec,
            "segment_offset_sec": request.segment_offset_sec,
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
    fn slicer_pitch_override_follows_parity_fixture_table() {
        for (line_index, line) in FIXTURES.lines().enumerate() {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let case: Value = serde_json::from_str(line).unwrap();
            let case_id = case["case_id"].as_str().unwrap();
            let actual = match case["kind"].as_str().unwrap() {
                "voiced_override" => {
                    let waveform = parse_waveform(&case["waveform"]);
                    let voiced = resample_voiced_override(
                        &waveform,
                        case["sr"].as_f64().unwrap(),
                        case["hop_length"].as_u64().unwrap() as usize,
                        &parse_bool_vec(&case["voiced_flag_override"]),
                        case["voiced_flag_override_step_sec"].as_f64().unwrap(),
                        case["segment_offset_sec"].as_f64().unwrap(),
                    )
                    .unwrap();
                    json!({
                        "voiced_flag": voiced,
                        "f0": vec![0.0; case["expect"]["f0"].as_array().unwrap().len()],
                    })
                }
                "pitch_split" => {
                    let waveform = parse_waveform(&case["waveform"]);
                    let request = PitchSplitRequest {
                        sample_rate: case["sr"].as_f64().unwrap(),
                        min_len_sec: case["min_len_sec"].as_f64().unwrap(),
                        max_len_sec: case["max_len_sec"].as_f64().unwrap(),
                        hop_length: case["hop_length"].as_u64().unwrap() as usize,
                        voiced_flag_override: parse_optional_bool_vec(
                            &case["voiced_flag_override"],
                        ),
                        voiced_flag_override_step_sec: parse_optional_f64(
                            &case["voiced_flag_override_step_sec"],
                        ),
                        segment_offset_sec: case["segment_offset_sec"].as_f64().unwrap(),
                    };
                    let rms_values = case.get("rms_db").map(|value| {
                        value
                            .as_array()
                            .unwrap()
                            .iter()
                            .map(|item| item.as_f64().unwrap())
                            .collect::<Vec<_>>()
                    });
                    let chunks = if let Some(voiced_flag) = case.get("direct_voiced_flag") {
                        pitch_based_split_with_voiced_flags(
                            &waveform,
                            &request,
                            &parse_bool_vec(voiced_flag),
                            |_, _, _| rms_values.clone().ok_or(SlicerPitchError::InvalidFrame),
                        )
                        .unwrap()
                    } else {
                        pitch_based_split_with_override(&waveform, &request, |_, _, _| {
                            rms_values.clone().ok_or(SlicerPitchError::InvalidFrame)
                        })
                        .unwrap()
                    };
                    json!({ "chunks": encode_segments(&chunks) })
                }
                "pitch_policy" => {
                    let sample_rate = case["sr"].as_f64().unwrap();
                    let config = parse_config(&case["config"]);
                    let pre_segments = parse_segments(&case["pre_segments"]);
                    let split_outputs: Vec<Vec<Segment>> = case["split_outputs"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(parse_segments)
                        .collect();
                    let mut split_iter = split_outputs.into_iter();
                    let mut split_calls = Vec::new();
                    let chunks = apply_pitch_override_policy(
                        &pre_segments,
                        sample_rate,
                        &config,
                        |segment, request| {
                            split_calls.push(encode_split_call(segment, request));
                            Ok(split_iter
                                .next()
                                .unwrap_or_else(|| panic!("{case_id}: unexpected split call")))
                        },
                    )
                    .unwrap();
                    assert!(
                        split_iter.next().is_none(),
                        "{case_id}: unused split outputs"
                    );
                    json!({
                        "chunks": encode_segments(&chunks),
                        "slicer_calls": [encode_pre_slicer_call(config.pre_slicer_params(sample_rate))],
                        "split_calls": split_calls,
                    })
                }
                other => panic!("unknown fixture kind {other}"),
            };

            assert_json_close(
                &actual,
                &case["expect"],
                &format!("{case_id} fixture line {}", line_index + 1),
            );
        }
    }

    #[test]
    fn pitch_override_config_defaults_match_python_signature() {
        let config = PitchOverrideConfig::default();
        assert_eq!(config.min_len_sec, 8.0);
        assert_eq!(config.max_len_sec, 22.0);
        assert_eq!(config.silence_removal_threshold_db, -40.0);
        assert_eq!(config.min_silence_len_ms, 800.0);
        assert_eq!(config.ultra_short_sec, 0.35);
        assert!(config.voiced_flag_override.is_none());
        assert!(config.voiced_flag_override_step_sec.is_none());
    }

    #[test]
    fn pitch_based_slice_composes_verified_dependencies_for_short_input() {
        let waveform = Waveform::Mono(vec![1.0; 100]);
        let config = PitchOverrideConfig {
            min_len_sec: 0.2,
            max_len_sec: 1.0,
            silence_removal_threshold_db: -40.0,
            min_silence_len_ms: 200.0,
            ultra_short_sec: 0.0,
            voiced_flag_override: Some(vec![true; 4]),
            voiced_flag_override_step_sec: Some(0.5),
        };

        let chunks = pitch_based_slice(&waveform, 1000.0, &config).unwrap();

        assert_eq!(
            encode_segments(&chunks),
            json!([{"offset": 0.0, "waveform": vec![1.0; 100]}])
        );
    }
}
