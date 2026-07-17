//! RMS/default silence slicer compatibility helpers.
//!
//! This module mirrors the deterministic `get_rms` and default `Slicer`
//! behavior from `inference/slicer/slicer2.py`. Python remains the runtime
//! owner for audio loading, heuristic/grid slicing, pitch/RMVPE smart slicing,
//! model execution, multiprocessing, CLI parsing, and filesystem effects.

use crate::slicer_segment::Waveform;

/// Error produced by the fixture-bound default slicer helpers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SlicerDefaultError {
    MinLengthIntervalOrder,
    MaxSilKeptOrder,
    InvalidHopSize,
    InvalidFrame,
    UnsupportedPadMode(String),
    EmptyStereo,
    RaggedStereo,
}

impl SlicerDefaultError {
    /// Returns the Python-compatible message where the legacy boundary has one.
    pub fn message(&self) -> String {
        match self {
            Self::MinLengthIntervalOrder => {
                "The following condition must be satisfied: min_length >= min_interval >= hop_size"
                    .to_string()
            }
            Self::MaxSilKeptOrder => {
                "The following condition must be satisfied: max_sil_kept >= hop_size".to_string()
            }
            Self::InvalidHopSize => "computed hop_size must be greater than 0".to_string(),
            Self::InvalidFrame => "frame_length and hop_length must be greater than 0".to_string(),
            Self::UnsupportedPadMode(mode) => format!("unsupported pad_mode: {mode}"),
            Self::EmptyStereo => "stereo waveform must contain at least one channel".to_string(),
            Self::RaggedStereo => "stereo waveform channels must have equal lengths".to_string(),
        }
    }
}

impl std::fmt::Display for SlicerDefaultError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message())
    }
}

impl std::error::Error for SlicerDefaultError {}

/// Padding behavior accepted by `get_rms`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PadMode {
    Constant,
    Reflect,
}

impl PadMode {
    /// Parses the Python fixture `pad_mode` string.
    ///
    /// # Errors
    ///
    /// Returns `UnsupportedPadMode` for modes outside the confirmed fixture
    /// boundary.
    pub fn parse(value: &str) -> Result<Self, SlicerDefaultError> {
        match value {
            "constant" => Ok(Self::Constant),
            "reflect" => Ok(Self::Reflect),
            other => Err(SlicerDefaultError::UnsupportedPadMode(other.to_string())),
        }
    }
}

/// Converted `Slicer` parameters after Python millisecond-to-frame conversion.
#[derive(Debug, Clone, PartialEq)]
pub struct Slicer {
    pub sample_rate: f64,
    pub threshold: f64,
    pub hop_size: usize,
    pub win_size: usize,
    pub min_length: usize,
    pub min_interval: usize,
    pub max_sil_kept: usize,
}

/// One default-slicer output chunk.
#[derive(Debug, Clone, PartialEq)]
pub struct SliceChunk {
    pub offset: f64,
    pub waveform: Waveform,
}

/// Calculates RMS energy with the same center padding and frame layout as
/// `inference/slicer/slicer2.py::get_rms`.
///
/// # Errors
///
/// Returns an error when the frame or hop size is outside the supported
/// fixture boundary.
pub fn get_rms(
    samples: &[f64],
    frame_length: usize,
    hop_length: usize,
    pad_mode: PadMode,
) -> Result<Vec<f64>, SlicerDefaultError> {
    if frame_length == 0 || hop_length == 0 {
        return Err(SlicerDefaultError::InvalidFrame);
    }

    let padding = frame_length / 2;
    let padded = pad_samples(samples, padding, pad_mode)?;
    if padded.len() < frame_length {
        return Ok(Vec::new());
    }

    let frame_count = padded.len() - frame_length + 1;
    let mut rms = Vec::new();
    for frame_start in (0..frame_count).step_by(hop_length) {
        let power = padded[frame_start..frame_start + frame_length]
            .iter()
            .map(|sample| sample.abs().powi(2))
            .sum::<f64>()
            / frame_length as f64;
        rms.push(power.sqrt());
    }

    Ok(rms)
}

impl Slicer {
    /// Creates a default slicer-compatible state object.
    ///
    /// # Errors
    ///
    /// Returns Python-compatible validation errors for invalid millisecond
    /// ordering, or a Rust boundary error when conversion produces an unusable
    /// hop size.
    pub fn new(
        sample_rate: f64,
        threshold_db: f64,
        min_length_ms: f64,
        min_interval_ms: f64,
        hop_size_ms: f64,
        max_sil_kept_ms: f64,
    ) -> Result<Self, SlicerDefaultError> {
        if !(min_length_ms >= min_interval_ms && min_interval_ms >= hop_size_ms) {
            return Err(SlicerDefaultError::MinLengthIntervalOrder);
        }
        if max_sil_kept_ms < hop_size_ms {
            return Err(SlicerDefaultError::MaxSilKeptOrder);
        }

        let min_interval_samples = sample_rate * min_interval_ms / 1000.0;
        let hop_size = round_half_even_usize(sample_rate * hop_size_ms / 1000.0)?;
        if hop_size == 0 {
            return Err(SlicerDefaultError::InvalidHopSize);
        }

        let threshold = 10.0_f64.powf(threshold_db / 20.0);
        let win_size = round_half_even_usize(min_interval_samples)?.min(4 * hop_size);
        let min_length =
            round_half_even_usize(sample_rate * min_length_ms / 1000.0 / hop_size as f64)?;
        let min_interval = round_half_even_usize(min_interval_samples / hop_size as f64)?;
        let max_sil_kept =
            round_half_even_usize(sample_rate * max_sil_kept_ms / 1000.0 / hop_size as f64)?;

        Ok(Self {
            sample_rate,
            threshold,
            hop_size,
            win_size,
            min_length,
            min_interval,
            max_sil_kept,
        })
    }

    /// Creates the same slicer used by `inference/API/slicer_api.py::default_slice`.
    ///
    /// # Errors
    ///
    /// Propagates constructor conversion errors.
    pub fn default_for_sample_rate(sample_rate: f64) -> Result<Self, SlicerDefaultError> {
        Self::new(sample_rate, -30.0, 5000.0, 300.0, 20.0, 500.0)
    }

    /// Slices a synthetic mono or channel-major stereo waveform with the
    /// legacy default silence state machine.
    ///
    /// # Errors
    ///
    /// Returns an error when a stereo payload is empty or ragged.
    pub fn slice(&self, waveform: &Waveform) -> Result<Vec<SliceChunk>, SlicerDefaultError> {
        let samples = waveform_mean_samples(waveform)?;
        if ceil_div(samples.len(), self.hop_size) <= self.min_length {
            return Ok(vec![SliceChunk {
                offset: 0.0,
                waveform: waveform.clone(),
            }]);
        }

        let rms_list = get_rms(&samples, self.win_size, self.hop_size, PadMode::Constant)?;
        let mut silence_tags = Vec::new();
        let mut silence_start = None;
        let mut clip_start = 0usize;

        for (index, rms) in rms_list.iter().enumerate() {
            if *rms < self.threshold {
                if silence_start.is_none() {
                    silence_start = Some(index);
                }
                continue;
            }

            let Some(start) = silence_start else {
                continue;
            };

            let is_leading_silence = start == 0 && index > self.max_sil_kept;
            let need_slice_middle =
                index - start >= self.min_interval && index - clip_start >= self.min_length;
            if !is_leading_silence && !need_slice_middle {
                silence_start = None;
                continue;
            }

            if index - start <= self.max_sil_kept {
                let pos = argmin(&rms_list[start..=index]) + start;
                if start == 0 {
                    silence_tags.push((0, pos));
                } else {
                    silence_tags.push((pos, pos));
                }
                clip_start = pos;
            } else if index - start <= self.max_sil_kept * 2 {
                let search_start = index - self.max_sil_kept;
                let search_end = start + self.max_sil_kept;
                let pos = argmin(&rms_list[search_start..=search_end]) + search_start;
                let pos_l = argmin(&rms_list[start..=start + self.max_sil_kept]) + start;
                let pos_r = argmin(&rms_list[search_start..=index]) + search_start;
                if start == 0 {
                    silence_tags.push((0, pos_r));
                    clip_start = pos_r;
                } else {
                    silence_tags.push((pos_l.min(pos), pos_r.max(pos)));
                    clip_start = pos_r.max(pos);
                }
            } else {
                let search_start = index - self.max_sil_kept;
                let pos_l = argmin(&rms_list[start..=start + self.max_sil_kept]) + start;
                let pos_r = argmin(&rms_list[search_start..=index]) + search_start;
                if start == 0 {
                    silence_tags.push((0, pos_r));
                } else {
                    silence_tags.push((pos_l, pos_r));
                }
                clip_start = pos_r;
            }
            silence_start = None;
        }

        let total_frames = rms_list.len();
        if let Some(start) = silence_start
            && total_frames - start >= self.min_interval
        {
            let silence_end = total_frames.min(start + self.max_sil_kept);
            let search_end = (silence_end + 1).min(total_frames);
            let pos = argmin(&rms_list[start..search_end]) + start;
            silence_tags.push((pos, total_frames + 1));
        }

        if silence_tags.is_empty() {
            return Ok(vec![SliceChunk {
                offset: 0.0,
                waveform: waveform.clone(),
            }]);
        }

        let mut chunks = Vec::new();
        if silence_tags[0].0 > 0 {
            chunks.push(self.apply_slice(waveform, 0, silence_tags[0].0));
        }
        for index in 0..silence_tags.len() - 1 {
            chunks.push(self.apply_slice(
                waveform,
                silence_tags[index].1,
                silence_tags[index + 1].0,
            ));
        }
        if silence_tags.last().unwrap().1 < total_frames {
            chunks.push(self.apply_slice(waveform, silence_tags.last().unwrap().1, total_frames));
        }
        Ok(chunks)
    }

    fn apply_slice(&self, waveform: &Waveform, begin: usize, end: usize) -> SliceChunk {
        let begin_sample = begin * self.hop_size;
        let end_sample = end * self.hop_size;
        SliceChunk {
            offset: begin_sample as f64 / self.sample_rate,
            waveform: slice_waveform(waveform, begin_sample, end_sample),
        }
    }
}

fn pad_samples(
    samples: &[f64],
    padding: usize,
    pad_mode: PadMode,
) -> Result<Vec<f64>, SlicerDefaultError> {
    match pad_mode {
        PadMode::Constant => {
            let mut output = vec![0.0; padding];
            output.extend_from_slice(samples);
            output.extend(std::iter::repeat_n(0.0, padding));
            Ok(output)
        }
        PadMode::Reflect => {
            if samples.len() < 2 && padding > 0 {
                return Err(SlicerDefaultError::InvalidFrame);
            }
            let mut output = Vec::with_capacity(samples.len() + padding * 2);
            for padded_index in 0..padding {
                let source_index =
                    reflect_index(padded_index as isize - padding as isize, samples.len());
                output.push(samples[source_index]);
            }
            output.extend_from_slice(samples);
            for padded_index in 0..padding {
                let source_index = reflect_index(
                    samples.len() as isize + padded_index as isize,
                    samples.len(),
                );
                output.push(samples[source_index]);
            }
            Ok(output)
        }
    }
}

fn reflect_index(index: isize, len: usize) -> usize {
    if len <= 1 {
        return 0;
    }
    let period = (2 * len - 2) as isize;
    let mut wrapped = index % period;
    if wrapped < 0 {
        wrapped += period;
    }
    if wrapped >= len as isize {
        (period - wrapped) as usize
    } else {
        wrapped as usize
    }
}

fn waveform_mean_samples(waveform: &Waveform) -> Result<Vec<f64>, SlicerDefaultError> {
    match waveform {
        Waveform::Mono(samples) => Ok(samples.clone()),
        Waveform::Stereo(channels) => {
            let Some(first) = channels.first() else {
                return Err(SlicerDefaultError::EmptyStereo);
            };
            if channels.iter().any(|channel| channel.len() != first.len()) {
                return Err(SlicerDefaultError::RaggedStereo);
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

fn argmin(values: &[f64]) -> usize {
    values
        .iter()
        .enumerate()
        .min_by(|(_, left), (_, right)| left.total_cmp(right))
        .map(|(index, _)| index)
        .unwrap_or(0)
}

fn ceil_div(value: usize, divisor: usize) -> usize {
    value.div_ceil(divisor)
}

fn round_half_even_usize(value: f64) -> Result<usize, SlicerDefaultError> {
    if !value.is_finite() || value < 0.0 {
        return Err(SlicerDefaultError::InvalidFrame);
    }
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
    Ok(rounded as usize)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    const FIXTURES: &str = include_str!("../../../../fixtures/slicer_rms_and_default_core.jsonl");

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

    fn encode_chunks(chunks: &[SliceChunk]) -> Value {
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

    fn encode_constructor_state(slicer: &Slicer) -> Value {
        json!({
            "sr": slicer.sample_rate,
            "threshold": slicer.threshold,
            "hop_size": slicer.hop_size,
            "win_size": slicer.win_size,
            "min_length": slicer.min_length,
            "min_interval": slicer.min_interval,
            "max_sil_kept": slicer.max_sil_kept,
        })
    }

    fn slicer_from_case(case: &Value) -> Result<Slicer, SlicerDefaultError> {
        let params = &case["params"];
        Slicer::new(
            params["sr"].as_f64().unwrap(),
            params["threshold"].as_f64().unwrap_or(-40.0),
            params["min_length"].as_f64().unwrap_or(5000.0),
            params["min_interval"].as_f64().unwrap_or(300.0),
            params["hop_size"].as_f64().unwrap_or(20.0),
            params["max_sil_kept"].as_f64().unwrap_or(5000.0),
        )
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
    fn slicer_default_follows_parity_fixture_table() {
        for (line_index, line) in FIXTURES.lines().enumerate() {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let case: Value = serde_json::from_str(line).unwrap();
            let case_id = case["case_id"].as_str().unwrap();
            let kind = case["kind"].as_str().unwrap();
            let actual = match kind {
                "get_rms" => {
                    let pad_mode =
                        PadMode::parse(case["pad_mode"].as_str().unwrap_or("constant")).unwrap();
                    let rms = get_rms(
                        parse_waveform(&case["input"]).as_mono().unwrap(),
                        case["frame_length"].as_u64().unwrap() as usize,
                        case["hop_length"].as_u64().unwrap() as usize,
                        pad_mode,
                    )
                    .unwrap();
                    json!([rms])
                }
                "constructor_state" => encode_constructor_state(&slicer_from_case(&case).unwrap()),
                "constructor_error" => {
                    let error = slicer_from_case(&case).unwrap_err();
                    json!({
                        "type": "ValueError",
                        "message": error.message(),
                    })
                }
                "slice" => {
                    let slicer = slicer_from_case(&case).unwrap();
                    let chunks = slicer.slice(&parse_waveform(&case["waveform"])).unwrap();
                    encode_chunks(&chunks)
                }
                "default_slice" => {
                    let slicer =
                        Slicer::default_for_sample_rate(case["sr"].as_f64().unwrap()).unwrap();
                    let chunks = slicer.slice(&parse_waveform(&case["waveform"])).unwrap();
                    encode_chunks(&chunks)
                }
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
    fn default_slicer_uses_legacy_default_parameters() {
        let slicer = Slicer::default_for_sample_rate(1000.0).unwrap();
        assert!((slicer.threshold - 10.0_f64.powf(-30.0 / 20.0)).abs() < 1e-12);
        assert_eq!(slicer.hop_size, 20);
        assert_eq!(slicer.win_size, 80);
        assert_eq!(slicer.max_sil_kept, 25);
    }

    #[test]
    fn slicer_rejects_ragged_stereo_payloads() {
        let slicer = Slicer::new(10.0, -20.0, 400.0, 200.0, 100.0, 300.0).unwrap();
        let error = slicer
            .slice(&Waveform::Stereo(vec![vec![1.0, 2.0], vec![1.0]]))
            .unwrap_err();
        assert_eq!(error, SlicerDefaultError::RaggedStereo);
    }

    trait WaveformTestExt {
        fn as_mono(&self) -> Option<&[f64]>;
    }

    impl WaveformTestExt for Waveform {
        fn as_mono(&self) -> Option<&[f64]> {
            match self {
                Waveform::Mono(samples) => Some(samples),
                Waveform::Stereo(_) => None,
            }
        }
    }
}
