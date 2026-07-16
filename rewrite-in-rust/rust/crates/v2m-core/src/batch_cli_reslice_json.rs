//! Batch CLI JSON timestamp and synthetic re-slicing helpers.
//!
//! This module mirrors deterministic behavior from `scripts/slice_asr_cli.py`
//! while Python remains the runtime owner for real audio decoding, resampling,
//! SoundFile/libsndfile encoding, model execution, and CLI routing.

use serde_json::{Map, Value, json};

use crate::batch_cli_planning::safe_stem;

/// Legacy-compatible error returned by the fixture-backed reslice model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResliceError {
    pub error_type: String,
    pub message: String,
}

impl ResliceError {
    fn new(error_type: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            error_type: error_type.into(),
            message: message.into(),
        }
    }
}

/// One fake WAV write captured by fixture checks.
#[derive(Debug, Clone, PartialEq)]
pub struct WriteCall {
    pub path: String,
    pub sr: i64,
    pub data: Vec<f64>,
}

impl WriteCall {
    fn value(&self) -> Value {
        json!({
            "path": self.path,
            "sr": self.sr,
            "len": self.data.len(),
            "data": self.data,
        })
    }
}

/// One fake lab sidecar captured by fixture checks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LabWrite {
    pub path: String,
    pub content: String,
}

impl LabWrite {
    fn value(&self) -> Value {
        json!({
            "path": self.path,
            "content": self.content,
        })
    }
}

/// Extracts text from fixture-modeled ASR-like result values.
pub fn extract_text_from_fixture(result: &Value) -> Result<String, ResliceError> {
    let kind = required_str(result, "kind")?;
    match kind {
        "none" => Ok(String::new()),
        "object" => Ok(required_str(result, "text")?.to_string()),
        "object_text_none" => Ok(required_str(result, "repr")?.to_string()),
        "object_text_error" => Err(ResliceError::new(
            "RuntimeError",
            required_str(result, "error")?,
        )),
        "dict" => {
            let object = result
                .get("value")
                .and_then(Value::as_object)
                .ok_or_else(|| key_error("value"))?;
            if let Some(text) = object.get("text").filter(|value| python_truthy(value)) {
                return Ok(python_str(text));
            }
            if let Some(transcript) = object
                .get("transcript")
                .filter(|value| python_truthy(value))
            {
                return Ok(python_str(transcript));
            }
            Ok(String::new())
        }
        "scalar" => Ok(python_str(result.get("value").unwrap_or(&Value::Null))),
        other => Err(ResliceError::new(
            "AssertionError",
            format!("unknown result kind {other:?}"),
        )),
    }
}

/// Builds the timestamp JSON payload produced by `save_timestamps_json`.
pub fn save_timestamps_json_model(case: &Value) -> Result<Value, ResliceError> {
    let chunks = required_array(case, "chunks")?;
    let results = required_array(case, "results")?;
    let chunk_indices = required_array(case, "chunk_indices")?;
    let sr = required_i64(case, "sr")?;
    let mut records = Vec::new();

    for (result_index, result) in results.iter().enumerate() {
        let chunk_index = chunk_indices
            .get(result_index)
            .ok_or_else(|| index_error("list index out of range"))
            .and_then(required_list_index_value)?;
        let chunk_position = python_list_index(chunk_index, chunks.len())
            .ok_or_else(|| index_error("list index out of range"))?;
        let chunk = chunks
            .get(chunk_position)
            .ok_or_else(|| index_error("list index out of range"))?;
        let offset = optional_f64(chunk, "offset")?.unwrap_or(0.0);
        let waveform = required_array(chunk, "waveform")?;
        if sr == 0 {
            return Err(ResliceError::new("ZeroDivisionError", "division by zero"));
        }
        let duration = waveform.len() as f64 / sr as f64;
        let mut record = Map::new();
        record.insert("index".to_string(), Value::from(chunk_index));
        record.insert(
            "offset".to_string(),
            json_number(round_half_even_to_decimal(offset, 6)),
        );
        record.insert(
            "duration".to_string(),
            json_number(round_half_even_to_decimal(duration, 6)),
        );
        record.insert(
            "text".to_string(),
            Value::String(extract_text_from_fixture(result)?.trim().to_string()),
        );
        records.push(Value::Object(record));
    }

    records.sort_by(|left, right| {
        left["offset"]
            .as_f64()
            .unwrap()
            .partial_cmp(&right["offset"].as_f64().unwrap())
            .unwrap()
    });

    let mut source = Map::new();
    source.insert(
        "path".to_string(),
        case.get("source_audio")
            .and_then(Value::as_str)
            .map(|path| Value::String(format!("__case__/{path}")))
            .unwrap_or(Value::Null),
    );
    source.insert(
        "md5".to_string(),
        case.get("source_md5")
            .and_then(Value::as_str)
            .map(|md5| Value::String(md5.to_string()))
            .unwrap_or(Value::Null),
    );

    let mut payload = Map::new();
    payload.insert("source".to_string(), Value::Object(source));
    payload.insert("chunks".to_string(), Value::Array(records));
    Ok(Value::Object(payload))
}

/// Models `slice_audio_from_json` with fixture payloads and synthetic waveforms.
pub fn slice_audio_from_json_model(case: &Value) -> Result<Value, ResliceError> {
    let json_path = required_str(case, "json_path")?;
    let source_audio = required_str(case, "source_audio")?;
    if case.get("json_exists").and_then(Value::as_bool) == Some(false) {
        return Err(ResliceError::new(
            "FileNotFoundError",
            format!("JSON file does not exist: __case__/{json_path}"),
        ));
    }
    if case.get("source_exists").and_then(Value::as_bool) == Some(false) {
        return Err(ResliceError::new(
            "FileNotFoundError",
            format!("Source audio file does not exist: __case__/{source_audio}"),
        ));
    }
    if case.get("raw_json").is_some() {
        return Err(ResliceError::new(
            "JSONDecodeError",
            "Expecting property name enclosed in double quotes: line 1 column 2 (char 1)",
        ));
    }

    let empty_records = Value::Array(Vec::new());
    let payload = case.get("payload").unwrap_or(&Value::Null);
    let records = records_from_payload(payload, &empty_records);
    if !python_truthy(records) {
        return Ok(json!({
            "status": "ok",
            "written": 0,
            "loaded_audio": false,
            "output_dir_exists": false,
            "stdout_lines": [format!("[SKIP] Empty JSON chunk list: __case__/{json_path}")],
            "writes": [],
            "labs": [],
        }));
    }

    let waveform = required_array(case, "waveform")?;
    if waveform.is_empty() {
        return Ok(json!({
            "status": "ok",
            "written": 0,
            "loaded_audio": true,
            "output_dir_exists": false,
            "stdout_lines": [format!("[SKIP] Empty audio: __case__/{source_audio}")],
            "writes": [],
            "labs": [],
        }));
    }

    let records = records
        .as_array()
        .ok_or_else(|| ResliceError::new("TypeError", "records is not iterable"))?;
    let actual_sr = required_i64(case, "actual_sr")?;
    let output_dir = required_str(case, "output_dir")?;
    let stem = safe_stem(source_audio);
    let mut stdout_lines = Vec::new();
    let mut writes = Vec::new();
    let mut labs = Vec::new();
    let mut written = 0;

    for record in records {
        let index = required_python_int(record, "index")?;
        let offset = required_f64(record, "offset")?;
        let duration = required_f64(record, "duration")?;
        let start_sample = 0.max(round_half_even(offset * actual_sr as f64) as i64);
        let end_sample = (waveform.len() as i64)
            .min(round_half_even((offset + duration) * actual_sr as f64) as i64);
        if start_sample >= end_sample {
            stdout_lines.push(format!(
                "  [SKIP] chunk {index}: invalid range [{offset:.4}s - {:.4}s]",
                offset + duration
            ));
            continue;
        }

        let wav_name = chunk_wav_name(&stem, index, offset, duration);
        let wav_path = path_join(output_dir, &wav_name);
        let data = waveform[start_sample as usize..end_sample as usize]
            .iter()
            .map(|value| value.as_f64().unwrap())
            .collect::<Vec<_>>();
        writes.push(WriteCall {
            path: wav_path,
            sr: actual_sr,
            data,
        });
        written += 1;

        let text = record.get("text").map(python_str).unwrap_or_default();
        if !text.is_empty() {
            labs.push(LabWrite {
                path: path_join(output_dir, &chunk_lab_name(&stem, index, offset)),
                content: text,
            });
        }
    }
    stdout_lines.push(format!(
        "  Sliced {written}/{} chunks from JSON timestamps -> __case__/{output_dir}",
        records.len()
    ));

    Ok(json!({
        "status": "ok",
        "written": written,
        "loaded_audio": true,
        "output_dir_exists": true,
        "stdout_lines": stdout_lines,
        "writes": writes.iter().map(WriteCall::value).collect::<Vec<_>>(),
        "labs": labs.iter().map(LabWrite::value).collect::<Vec<_>>(),
    }))
}

/// Models `save_chunks` write paths and fake writes.
pub fn save_chunks_model(case: &Value) -> Result<Value, ResliceError> {
    let chunk_dir = required_str(case, "chunk_dir")?;
    let source_stem = required_str(case, "source_stem")?;
    let sr = required_i64(case, "sr")?;
    let chunks = required_array(case, "chunks")?;
    let mut writes = Vec::new();
    let mut saved_paths = Vec::new();

    for (index, chunk) in chunks.iter().enumerate() {
        let offset = optional_f64(chunk, "offset")?.unwrap_or(0.0);
        let waveform = required_array(chunk, "waveform")?;
        if sr == 0 {
            return Err(ResliceError::new("ZeroDivisionError", "division by zero"));
        }
        let duration = waveform.len() as f64 / sr as f64;
        let path = path_join(
            chunk_dir,
            &chunk_wav_name(source_stem, index as i64, offset, duration),
        );
        let data = waveform
            .iter()
            .map(|value| value.as_f64().unwrap())
            .collect::<Vec<_>>();
        writes.push(WriteCall {
            path: path.clone(),
            sr,
            data,
        });
        saved_paths.push(Value::String(path));
    }

    Ok(json!({
        "status": "ok",
        "created_dir": true,
        "saved_paths": saved_paths,
        "writes": writes.iter().map(WriteCall::value).collect::<Vec<_>>(),
    }))
}

fn records_from_payload<'a>(payload: &'a Value, empty_records: &'a Value) -> &'a Value {
    payload
        .as_object()
        .map(|object| object.get("chunks").unwrap_or(empty_records))
        .unwrap_or(payload)
}

fn required_array<'a>(value: &'a Value, key: &str) -> Result<&'a Vec<Value>, ResliceError> {
    value
        .get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| key_error(key))
}

fn required_str<'a>(value: &'a Value, key: &str) -> Result<&'a str, ResliceError> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| key_error(key))
}

fn required_i64(value: &Value, key: &str) -> Result<i64, ResliceError> {
    value
        .get(key)
        .and_then(Value::as_i64)
        .ok_or_else(|| key_error(key))
}

fn required_python_int(value: &Value, key: &str) -> Result<i64, ResliceError> {
    let raw = value.get(key).ok_or_else(|| key_error(key))?;
    python_int(raw).ok_or_else(|| int_value_error(raw))
}

fn required_f64(value: &Value, key: &str) -> Result<f64, ResliceError> {
    let raw = value.get(key).ok_or_else(|| key_error(key))?;
    json_to_f64(raw).ok_or_else(|| value_error(raw))
}

fn optional_f64(value: &Value, key: &str) -> Result<Option<f64>, ResliceError> {
    value
        .get(key)
        .map(|value| json_to_f64(value).ok_or_else(|| value_error(value)))
        .transpose()
}

fn json_to_f64(value: &Value) -> Option<f64> {
    match value {
        Value::Number(number) => number.as_f64(),
        Value::String(value) => value.parse::<f64>().ok(),
        _ => None,
    }
}

fn python_int(value: &Value) -> Option<i64> {
    match value {
        Value::Bool(value) => Some(if *value { 1 } else { 0 }),
        Value::Number(number) => number
            .as_i64()
            .or_else(|| number.as_u64().and_then(|value| i64::try_from(value).ok()))
            .or_else(|| number.as_f64().map(|value| value.trunc() as i64)),
        Value::String(value) => value.parse::<i64>().ok(),
        _ => None,
    }
}

fn python_list_index(index: i64, len: usize) -> Option<usize> {
    if index >= 0 {
        usize::try_from(index).ok().filter(|index| *index < len)
    } else {
        let position = len as i64 + index;
        (position >= 0).then_some(position as usize)
    }
}

fn required_list_index_value(value: &Value) -> Result<i64, ResliceError> {
    value
        .as_i64()
        .ok_or_else(|| ResliceError::new("TypeError", list_index_type_error(value)))
}

fn list_index_type_error(value: &Value) -> String {
    format!(
        "list indices must be integers or slices, not {}",
        python_type_name(value)
    )
}

fn key_error(key: &str) -> ResliceError {
    ResliceError::new("KeyError", format!("'{key}'"))
}

fn index_error(message: &str) -> ResliceError {
    ResliceError::new("IndexError", message)
}

fn value_error(value: &Value) -> ResliceError {
    let message = match value {
        Value::String(value) => format!("could not convert string to float: '{value}'"),
        _ => format!("could not convert value to float: {}", python_str(value)),
    };
    ResliceError::new("ValueError", message)
}

fn int_value_error(value: &Value) -> ResliceError {
    let message = match value {
        Value::String(value) => format!("invalid literal for int() with base 10: '{value}'"),
        Value::Null => {
            "int() argument must be a string, a bytes-like object or a real number, not 'NoneType'"
                .to_string()
        }
        _ => format!(
            "invalid literal for int() with base 10: '{}'",
            python_str(value)
        ),
    };
    ResliceError::new("ValueError", message)
}

fn python_type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "NoneType",
        Value::Bool(_) => "bool",
        Value::Number(number) if number.is_i64() || number.is_u64() => "int",
        Value::Number(_) => "float",
        Value::String(_) => "str",
        Value::Array(_) => "list",
        Value::Object(_) => "dict",
    }
}

fn python_truthy(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(value) => *value,
        Value::Number(value) => value.as_f64().is_some_and(|number| number != 0.0),
        Value::String(value) => !value.is_empty(),
        Value::Array(value) => !value.is_empty(),
        Value::Object(value) => !value.is_empty(),
    }
}

fn python_str(value: &Value) -> String {
    match value {
        Value::Null => "None".to_string(),
        Value::Bool(true) => "True".to_string(),
        Value::Bool(false) => "False".to_string(),
        Value::Number(number) => number.to_string(),
        Value::String(value) => value.clone(),
        Value::Array(values) => {
            let items = values.iter().map(python_repr).collect::<Vec<_>>();
            format!("[{}]", items.join(", "))
        }
        Value::Object(object) => {
            let items = object
                .iter()
                .map(|(key, value)| format!("'{}': {}", key, python_repr(value)))
                .collect::<Vec<_>>();
            format!("{{{}}}", items.join(", "))
        }
    }
}

fn python_repr(value: &Value) -> String {
    match value {
        Value::String(value) => format!("'{}'", value.replace('\'', "\\'")),
        _ => python_str(value),
    }
}

fn round_half_even_to_decimal(value: f64, places: i32) -> f64 {
    let factor = 10_f64.powi(places);
    round_half_even(value * factor) / factor
}

fn round_half_even(value: f64) -> f64 {
    if !value.is_finite() {
        return value;
    }

    let floor = value.floor();
    let diff = value - floor;
    if diff < 0.5 {
        floor
    } else if diff > 0.5 {
        floor + 1.0
    } else if (floor as i128).rem_euclid(2) == 0 {
        floor
    } else {
        floor + 1.0
    }
}

fn json_number(value: f64) -> Value {
    Value::Number(serde_json::Number::from_f64(value).unwrap())
}

fn chunk_wav_name(source_stem: &str, index: i64, offset: f64, duration: f64) -> String {
    format!("{source_stem}_chunk{index:04}_off{offset:08.2}s_dur{duration:07.2}s.wav")
}

fn chunk_lab_name(source_stem: &str, index: i64, offset: f64) -> String {
    format!("{source_stem}_chunk{index:04}_off{offset:08.2}s.lab")
}

fn path_join(left: &str, right: &str) -> String {
    format!(
        "{}/{}",
        left.trim_end_matches('/'),
        right.trim_start_matches('/')
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURES: &str = include_str!("../../../../fixtures/batch_cli_reslice_json_core.jsonl");

    fn load_cases() -> Vec<Value> {
        FIXTURES
            .lines()
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .map(|line| serde_json::from_str(line).unwrap())
            .collect()
    }

    fn assert_subset(case_id: &str, actual: &Value, expected: &Value, path: &str) {
        match expected {
            Value::Object(expected_object) => {
                let actual_object = actual
                    .as_object()
                    .unwrap_or_else(|| panic!("{case_id}: {path} actual is not object"));
                for (key, expected_value) in expected_object {
                    let child_path = format!("{path}.{key}");
                    let actual_value = actual_object
                        .get(key)
                        .unwrap_or_else(|| panic!("{case_id}: missing key {child_path}"));
                    assert_subset(case_id, actual_value, expected_value, &child_path);
                }
            }
            Value::Array(expected_values) => {
                let actual_values = actual
                    .as_array()
                    .unwrap_or_else(|| panic!("{case_id}: {path} actual is not array"));
                assert_eq!(
                    actual_values.len(),
                    expected_values.len(),
                    "{case_id}: {path} list length differs"
                );
                for (index, (actual_value, expected_value)) in
                    actual_values.iter().zip(expected_values).enumerate()
                {
                    assert_subset(
                        case_id,
                        actual_value,
                        expected_value,
                        &format!("{path}[{index}]"),
                    );
                }
            }
            _ => assert_eq!(actual, expected, "{case_id}: {path} differs"),
        }
    }

    #[test]
    fn batch_cli_reslice_json_fixtures_match() {
        for case in load_cases() {
            let case_id = case["case_id"].as_str().unwrap();
            match case["operation"].as_str().unwrap() {
                "extract_text" => run_grouped(case_id, &case, run_extract_text_case),
                "save_timestamps_json" => {
                    let actual = run_save_timestamps_json(&case);
                    assert_subset(case_id, &actual, &case["expect"], "");
                }
                "save_timestamps_json_cases" => {
                    run_grouped(case_id, &case, run_save_timestamps_json)
                }
                "slice_audio_from_json" => {
                    let actual = run_slice_audio_from_json(&case);
                    assert_subset(case_id, &actual, &case["expect"], "");
                }
                "slice_audio_from_json_cases" => {
                    run_grouped(case_id, &case, run_slice_audio_from_json)
                }
                "save_chunks_cases" => run_grouped(case_id, &case, run_save_chunks),
                other => panic!("unknown operation {other:?}"),
            }
        }
    }

    fn run_grouped(case_id: &str, case: &Value, runner: fn(&Value) -> Value) {
        for (index, subcase) in case["cases"].as_array().unwrap().iter().enumerate() {
            let actual = runner(subcase);
            assert_subset(
                &format!("{case_id}[{index}]"),
                &actual,
                &subcase["expect"],
                "",
            );
        }
    }

    fn run_extract_text_case(case: &Value) -> Value {
        match extract_text_from_fixture(&case["result"]) {
            Ok(text) => json!({
                "status": "ok",
                "text": text,
            }),
            Err(error) => error_value(error),
        }
    }

    fn run_save_timestamps_json(case: &Value) -> Value {
        match save_timestamps_json_model(case) {
            Ok(payload) => {
                let json_dir = case["json_dir"].as_str().unwrap();
                let source_stem = case["source_stem"].as_str().unwrap();
                let json_path = format!("{json_dir}/{source_stem}.json");
                json!({
                    "status": "ok",
                    "json_path": json_path,
                    "stdout_lines": [format!("  Timestamps saved: __case__/{json_path}")],
                    "payload_json": serde_json::to_string_pretty(&payload).unwrap(),
                })
            }
            Err(error) => error_value(error),
        }
    }

    fn run_slice_audio_from_json(case: &Value) -> Value {
        slice_audio_from_json_model(case).unwrap_or_else(error_value)
    }

    fn run_save_chunks(case: &Value) -> Value {
        save_chunks_model(case).unwrap_or_else(error_value)
    }

    fn error_value(error: ResliceError) -> Value {
        json!({
            "status": "error",
            "error_type": error.error_type,
            "error": error.message,
        })
    }
}
