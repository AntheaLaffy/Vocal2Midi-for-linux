//! Romaji ASR fake-session metadata and batch preparation contract.
//!
//! This module mirrors the deterministic `prepare_batch` helper behavior from
//! `inference/romaji_asr/common.py` using synthetic waveforms and metadata. It
//! does not own audio IO, ONNX Runtime sessions, providers, or model execution.

use std::collections::BTreeMap;
use std::fmt;

use half::f16;
use ndarray::Array2;

#[derive(Debug, Clone, PartialEq, Eq)]
/// ONNX input dimension projected from Python metadata.
pub enum Dim {
    /// Carries the Python-compatible int value.
    Int(i64),
    /// Carries the Python-compatible bool value.
    Bool(bool),
    /// Represents the Python-compatible dynamic case.
    Dynamic,
}

impl Dim {
    fn python_int_if_instance_int(&self) -> Option<i64> {
        match self {
            Self::Int(value) => Some(*value),
            Self::Bool(value) => Some(i64::from(*value)),
            Self::Dynamic => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Name, element type, and dimensions for one fake ONNX input.
pub struct InputMeta {
    /// The name.
    pub name: String,
    /// The Python-compatible type name.
    pub type_name: String,
    /// The ordered shape.
    pub shape: Vec<Dim>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// NumPy dtypes supported by the legacy feed-construction helper.
pub enum NumpyDType {
    /// Represents the Python-compatible float16 case.
    Float16,
    /// Represents the Python-compatible float32 case.
    Float32,
    /// Represents the Python-compatible int64 case.
    Int64,
    /// Represents the Python-compatible int32 case.
    Int32,
}

impl NumpyDType {
    /// Returns the canonical NumPy dtype name.
    pub const fn as_numpy_name(self) -> &'static str {
        match self {
            Self::Float16 => "float16",
            Self::Float32 => "float32",
            Self::Int64 => "int64",
            Self::Int32 => "int32",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
/// Typed two-dimensional feed array produced for a fake ONNX session.
pub enum FeedArray {
    /// Carries the Python-compatible f16 value.
    F16(Array2<f16>),
    /// Carries the Python-compatible f32 value.
    F32(Array2<f32>),
    /// Carries the Python-compatible i64 value.
    I64(Array2<i64>),
    /// Carries the Python-compatible i32 value.
    I32(Array2<i32>),
}

impl FeedArray {
    /// Returns the NumPy-compatible dtype name for this array.
    pub fn dtype_name(&self) -> &'static str {
        match self {
            Self::F16(_) => "float16",
            Self::F32(_) => "float32",
            Self::I64(_) => "int64",
            Self::I32(_) => "int32",
        }
    }

    /// Returns the array shape as `[batch, samples]`.
    pub fn shape(&self) -> Vec<usize> {
        match self {
            Self::F16(array) => array.shape().to_vec(),
            Self::F32(array) => array.shape().to_vec(),
            Self::I64(array) => array.shape().to_vec(),
            Self::I32(array) => array.shape().to_vec(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Recorded legacy audio-loader invocation.
pub struct LoadAudioCall {
    /// The filesystem path.
    pub path: String,
    /// The sample rate in hertz.
    pub sample_rate: i64,
}

#[derive(Debug, Clone, PartialEq)]
/// Prepared fake-session feeds plus observable loader metadata.
pub struct PreparedBatch {
    /// The feeds.
    pub feeds: BTreeMap<String, FeedArray>,
    /// The ordered used lengths.
    pub used_lengths: Vec<usize>,
    /// The ordered load audio calls.
    pub load_audio_calls: Vec<LoadAudioCall>,
}

#[derive(Debug, Clone, PartialEq)]
/// Python-compatible batch preparation failure and calls made before it.
pub struct BatchMetadataError {
    /// The Python-compatible error type.
    pub error_type: &'static str,
    /// The message text.
    pub message: String,
    /// The ordered load audio calls.
    pub load_audio_calls: Vec<LoadAudioCall>,
}

impl BatchMetadataError {
    fn value_error(message: impl Into<String>, load_audio_calls: Vec<LoadAudioCall>) -> Self {
        Self {
            error_type: "ValueError",
            message: message.into(),
            load_audio_calls,
        }
    }

    fn key_error(message: impl Into<String>, load_audio_calls: Vec<LoadAudioCall>) -> Self {
        Self {
            error_type: "KeyError",
            message: message.into(),
            load_audio_calls,
        }
    }
}

impl fmt::Display for BatchMetadataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.error_type, self.message)
    }
}

impl std::error::Error for BatchMetadataError {}

/// Reads a fixed batch dimension from the first input, when present.
pub fn get_fixed_batch_size(inputs: &[InputMeta]) -> Option<i64> {
    let shape = &inputs.first()?.shape;
    if shape.is_empty() {
        return None;
    }
    shape[0].python_int_if_instance_int()
}

/// Reads a fixed sample dimension from the first input, when present.
pub fn get_fixed_num_samples(inputs: &[InputMeta]) -> Option<i64> {
    let shape = &inputs.first()?.shape;
    if shape.len() < 2 {
        return None;
    }
    shape[1].python_int_if_instance_int()
}

/// Maps an ONNX Runtime tensor type string to the legacy NumPy dtype fallback.
pub fn ort_type_to_numpy_dtype(ort_type: &str) -> NumpyDType {
    if ort_type.contains("float16") {
        NumpyDType::Float16
    } else if ort_type.contains("float") {
        NumpyDType::Float32
    } else if ort_type.contains("int64") {
        NumpyDType::Int64
    } else if ort_type.contains("int32") {
        NumpyDType::Int32
    } else {
        NumpyDType::Float32
    }
}

/// Builds fake-session feeds from injected waveforms and ONNX input metadata.
///
/// # Errors
///
/// Returns [`BatchMetadataError`] for an empty audio list, a fixed batch-size
/// mismatch, missing injected waveform, negative fixed dimension, or missing
/// `input_values` metadata.
pub fn prepare_batch_from_waveforms(
    inputs: &[InputMeta],
    audio_paths: &[String],
    waveforms: &BTreeMap<String, Vec<f32>>,
    sample_rate: i64,
) -> Result<PreparedBatch, BatchMetadataError> {
    let mut load_audio_calls = Vec::new();
    if audio_paths.is_empty() {
        return Err(BatchMetadataError::value_error(
            "audio_paths must not be empty.",
            load_audio_calls,
        ));
    }

    let fixed_batch_size = get_fixed_batch_size(inputs);
    if let Some(fixed_batch_size) = fixed_batch_size {
        if fixed_batch_size != audio_paths.len() as i64 {
            return Err(BatchMetadataError::value_error(
                format!(
                    "Model expects batch_size={}, got {}.",
                    fixed_batch_size,
                    audio_paths.len()
                ),
                load_audio_calls,
            ));
        }
    }

    let mut loaded = Vec::with_capacity(audio_paths.len());
    for path in audio_paths {
        load_audio_calls.push(LoadAudioCall {
            path: path.clone(),
            sample_rate,
        });
        let Some(waveform) = waveforms.get(path) else {
            return Err(BatchMetadataError::key_error(
                python_key_error_message(path),
                load_audio_calls,
            ));
        };
        loaded.push(waveform.clone());
    }

    let lengths: Vec<usize> = loaded.iter().map(Vec::len).collect();
    let max_len = lengths.iter().copied().max().unwrap_or(0);
    let target_num_samples = match get_fixed_num_samples(inputs).unwrap_or(0) {
        fixed_num_samples if fixed_num_samples < 0 => {
            return Err(BatchMetadataError::value_error(
                "negative dimensions are not allowed",
                load_audio_calls,
            ));
        }
        fixed_num_samples if fixed_num_samples != 0 => fixed_num_samples as usize,
        _ => max_len,
    };

    let batch_size = loaded.len();
    let mut input_values = Array2::<f32>::zeros((batch_size, target_num_samples));
    let mut attention_mask = Array2::<i64>::zeros((batch_size, target_num_samples));
    let mut used_lengths = Vec::with_capacity(batch_size);

    for (batch_index, waveform) in loaded.iter().enumerate() {
        let num = waveform.len().min(target_num_samples);
        for sample_index in 0..num {
            input_values[(batch_index, sample_index)] = waveform[sample_index];
            attention_mask[(batch_index, sample_index)] = 1;
        }
        used_lengths.push(num);
    }

    let input_meta: BTreeMap<&str, &InputMeta> = inputs
        .iter()
        .map(|meta| (meta.name.as_str(), meta))
        .collect();
    let input_values_meta = input_meta
        .get("input_values")
        .ok_or_else(|| BatchMetadataError::key_error("'input_values'", load_audio_calls.clone()))?;

    let mut feeds = BTreeMap::new();
    feeds.insert(
        "input_values".to_string(),
        cast_input_values(
            &input_values,
            ort_type_to_numpy_dtype(&input_values_meta.type_name),
        ),
    );

    if let Some(mask_meta) = input_meta.get("attention_mask") {
        feeds.insert(
            "attention_mask".to_string(),
            cast_attention_mask(
                &attention_mask,
                ort_type_to_numpy_dtype(&mask_meta.type_name),
            ),
        );
    }

    Ok(PreparedBatch {
        feeds,
        used_lengths,
        load_audio_calls,
    })
}

fn cast_input_values(values: &Array2<f32>, dtype: NumpyDType) -> FeedArray {
    match dtype {
        NumpyDType::Float16 => FeedArray::F16(values.mapv(f16::from_f32)),
        NumpyDType::Float32 => FeedArray::F32(values.clone()),
        NumpyDType::Int64 => FeedArray::I64(values.mapv(|value| value as i64)),
        NumpyDType::Int32 => FeedArray::I32(values.mapv(|value| value as i32)),
    }
}

fn cast_attention_mask(values: &Array2<i64>, dtype: NumpyDType) -> FeedArray {
    match dtype {
        NumpyDType::Float16 => FeedArray::F16(values.mapv(|value| f16::from_f32(value as f32))),
        NumpyDType::Float32 => FeedArray::F32(values.mapv(|value| value as f32)),
        NumpyDType::Int64 => FeedArray::I64(values.clone()),
        NumpyDType::Int32 => FeedArray::I32(values.mapv(|value| value as i32)),
    }
}

fn python_key_error_message(key: &str) -> String {
    if !key.contains('\'') {
        format!("'{key}'")
    } else if !key.contains('"') {
        format!("\"{key}\"")
    } else {
        format!("'{}'", key.replace('\\', "\\\\").replace('\'', "\\'"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Map, Value};

    const FIXTURES: &str =
        include_str!("../../../../fixtures/asr_romaji_batch_metadata_contract.jsonl");

    #[test]
    fn asr_romaji_batch_metadata_contract_matches_fixtures() {
        for (index, line) in FIXTURES.lines().filter(|line| !line.is_empty()).enumerate() {
            let case: Value = serde_json::from_str(line).unwrap();
            let actual = result_for(&case);
            let expected = case["expected"].clone();
            assert_eq!(actual, expected, "case {index}: {}", case["category"]);
        }
    }

    fn result_for(case: &Value) -> Value {
        match case["call"].as_str().unwrap() {
            "metadata" => {
                let inputs = inputs_from_case(case);
                serde_json::json!({
                    "ok": true,
                    "fixed_batch_size": get_fixed_batch_size(&inputs),
                    "fixed_num_samples": get_fixed_num_samples(&inputs),
                })
            }
            "ort_type_to_numpy_dtype" => serde_json::json!({
                "ok": true,
                "dtype": ort_type_to_numpy_dtype(case["ort_type"].as_str().unwrap()).as_numpy_name(),
            }),
            "prepare_batch" => {
                let inputs = inputs_from_case(case);
                let audio_paths = case["audio_paths"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|value| value.as_str().unwrap().to_string())
                    .collect::<Vec<_>>();
                let waveforms = waveforms_from_case(case);
                let sample_rate = case["sample_rate"].as_i64().unwrap();
                match prepare_batch_from_waveforms(&inputs, &audio_paths, &waveforms, sample_rate) {
                    Ok(batch) => batch_result(batch),
                    Err(err) => error_result(err),
                }
            }
            other => panic!("unknown fixture call {other}"),
        }
    }

    fn inputs_from_case(case: &Value) -> Vec<InputMeta> {
        case["inputs"]
            .as_array()
            .unwrap()
            .iter()
            .map(|item| InputMeta {
                name: item["name"].as_str().unwrap().to_string(),
                type_name: item["type"].as_str().unwrap().to_string(),
                shape: item["shape"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(dim_from_json)
                    .collect(),
            })
            .collect()
    }

    fn dim_from_json(value: &Value) -> Dim {
        match value {
            Value::Bool(value) => Dim::Bool(*value),
            Value::Number(value) => Dim::Int(value.as_i64().unwrap()),
            _ => Dim::Dynamic,
        }
    }

    fn waveforms_from_case(case: &Value) -> BTreeMap<String, Vec<f32>> {
        case["waveforms"]
            .as_object()
            .unwrap()
            .iter()
            .map(|(key, value)| {
                (
                    key.clone(),
                    value
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|value| value.as_f64().unwrap() as f32)
                        .collect(),
                )
            })
            .collect()
    }

    fn batch_result(batch: PreparedBatch) -> Value {
        let feeds = batch
            .feeds
            .into_iter()
            .map(|(name, array)| (name, encode_feed_array(&array)))
            .collect::<Map<_, _>>();
        serde_json::json!({
            "ok": true,
            "feeds": feeds,
            "used_lengths": batch.used_lengths,
            "load_audio_calls": batch.load_audio_calls.into_iter().map(|call| {
                serde_json::json!({"path": call.path, "sample_rate": call.sample_rate})
            }).collect::<Vec<_>>(),
        })
    }

    fn error_result(err: BatchMetadataError) -> Value {
        serde_json::json!({
            "ok": false,
            "error_type": err.error_type,
            "message": err.message,
            "load_audio_calls": err.load_audio_calls.into_iter().map(|call| {
                serde_json::json!({"path": call.path, "sample_rate": call.sample_rate})
            }).collect::<Vec<_>>(),
        })
    }

    fn encode_feed_array(array: &FeedArray) -> Value {
        serde_json::json!({
            "dtype": array.dtype_name(),
            "shape": array.shape(),
            "values": feed_values_json(array),
        })
    }

    fn feed_values_json(array: &FeedArray) -> Value {
        match array {
            FeedArray::F16(values) => serde_json::json!(
                values
                    .rows()
                    .into_iter()
                    .map(|row| row.iter().map(|value| value.to_f32()).collect::<Vec<_>>())
                    .collect::<Vec<_>>()
            ),
            FeedArray::F32(values) => serde_json::json!(
                values
                    .rows()
                    .into_iter()
                    .map(|row| row.iter().copied().collect::<Vec<_>>())
                    .collect::<Vec<_>>()
            ),
            FeedArray::I64(values) => serde_json::json!(
                values
                    .rows()
                    .into_iter()
                    .map(|row| row.iter().copied().collect::<Vec<_>>())
                    .collect::<Vec<_>>()
            ),
            FeedArray::I32(values) => serde_json::json!(
                values
                    .rows()
                    .into_iter()
                    .map(|row| row.iter().copied().collect::<Vec<_>>())
                    .collect::<Vec<_>>()
            ),
        }
    }
}
