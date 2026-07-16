//! Web settings JSON merge and persistence helpers.
//!
//! This module mirrors the settings helpers and route-level update/reset
//! behavior in `web_server.py` while legacy Python remains the runtime owner.

use serde_json::{Map, Value, json};

/// Response produced by a settings update operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsUpdateResponse {
    pub status_code: u16,
    pub success: bool,
    pub message: Option<String>,
    pub error: Option<String>,
    pub settings: Option<Value>,
    pub saved_payload: Option<String>,
}

/// Returns the Web default settings object.
pub fn default_settings() -> Value {
    json!({
        "models": {
            "game_model_path": "experiments/GAME-1.0.3-medium-onnx",
            "hfa_model_path": "experiments/1218_hfa_model_new_dict",
            "asr_model_path": "experiments/Qwen3-ASR-1.7B",
            "phoneme_asr_model_path": "experiments/romajiASR",
            "rmvpe_model_path": "experiments/RMVPE/rmvpe.onnx"
        },
        "params": {
            "seg_threshold": 0.2,
            "seg_radius": 0.02,
            "est_threshold": 0.2,
            "t0": 0.0,
            "nsteps": 8,
            "game_batch": 1,
            "asr_batch": 2,
            "slice_min": 8.0,
            "slice_max": 22.0
        },
        "debug": {
            "export_txt": false,
            "export_csv": false,
            "export_chunks": false,
            "pitch_format": "name",
            "round_pitch": true
        },
        "pipeline": {
            "slicing_method": "auto",
            "language": "zh",
            "lyric_output_mode": "pinyin",
            "device": "dml",
            "tempo": 120,
            "save_dir": "./output",
            "quantize_precision": "none",
            "quantize_algorithm": "dev",
            "enable_lyrics_match": false,
            "output_lyrics": true,
            "export_ustx": false,
            "output_pitch_curve": true
        },
        "downloads": {
            "qwen_source": "auto",
            "proxy_mode": "system",
            "proxy_url": "",
            "force": false
        }
    })
}

/// Merges persisted settings over defaults while keeping unknown top-level keys out.
pub fn merge_settings(defaults: &Value, overrides: &Value) -> Value {
    let mut merged = defaults.clone();
    let Some(defaults_object) = defaults.as_object() else {
        return merged;
    };
    let Some(overrides_object) = overrides.as_object() else {
        return merged;
    };

    for (section, default_value) in defaults_object {
        let Some(override_value) = overrides_object.get(section) else {
            continue;
        };
        if default_value.is_object() {
            if let (Some(merged_section), Some(override_section)) = (
                merged.get_mut(section).and_then(Value::as_object_mut),
                override_value.as_object(),
            ) {
                update_object(merged_section, override_section);
            }
        } else if !override_value.is_null() {
            merged[section] = override_value.clone();
        }
    }
    merged
}

/// Parses a persisted settings file payload, falling back to defaults on error.
pub fn load_settings_payload(defaults: &Value, payload: Option<&str>) -> Value {
    let Some(payload) = payload else {
        return defaults.clone();
    };
    match serde_json::from_str::<Value>(payload) {
        Ok(value) => merge_settings(defaults, &value),
        Err(_) => defaults.clone(),
    }
}

/// Serializes settings like Python `json.dumps(..., ensure_ascii=False, indent=2) + "\n"`.
pub fn save_settings_payload(settings: &Value) -> String {
    format!("{}\n", serde_json::to_string_pretty(settings).unwrap())
}

/// Applies a settings update request.
pub fn update_settings(current: &mut Value, data: &Value) -> SettingsUpdateResponse {
    let defaults = default_settings();
    let Some(data_object) = data.as_object() else {
        return update_non_object_settings(current, data, &defaults);
    };

    for section in known_sections(&defaults) {
        let Some(section_value) = data_object.get(section) else {
            continue;
        };
        if !section_value.is_object() {
            return SettingsUpdateResponse {
                status_code: 400,
                success: false,
                message: None,
                error: Some(format!("{section} must be an object")),
                settings: None,
                saved_payload: None,
            };
        }
        if let (Some(current_section), Some(update_section)) = (
            current.get_mut(section).and_then(Value::as_object_mut),
            section_value.as_object(),
        ) {
            update_object(current_section, update_section);
        }
    }

    let saved_payload = save_settings_payload(current);
    SettingsUpdateResponse {
        status_code: 200,
        success: true,
        message: Some("Settings updated successfully".to_string()),
        error: None,
        settings: Some(current.clone()),
        saved_payload: Some(saved_payload),
    }
}

fn update_non_object_settings(
    current: &Value,
    data: &Value,
    defaults: &Value,
) -> SettingsUpdateResponse {
    if data.is_null() {
        return SettingsUpdateResponse {
            status_code: 400,
            success: false,
            message: None,
            error: Some("Invalid JSON in request body".to_string()),
            settings: None,
            saved_payload: None,
        };
    }

    if non_object_update_would_raise(data, defaults) {
        return SettingsUpdateResponse {
            status_code: 500,
            success: false,
            message: None,
            error: Some(format!(
                "Failed to update settings: {}",
                legacy_non_object_error(data)
            )),
            settings: None,
            saved_payload: None,
        };
    }

    SettingsUpdateResponse {
        status_code: 200,
        success: true,
        message: Some("Settings updated successfully".to_string()),
        error: None,
        settings: Some(current.clone()),
        saved_payload: Some(save_settings_payload(current)),
    }
}

fn non_object_update_would_raise(data: &Value, defaults: &Value) -> bool {
    let sections = known_sections(defaults);
    match data {
        Value::Array(items) => sections
            .iter()
            .any(|section| items.iter().any(|item| item.as_str() == Some(*section))),
        Value::String(text) => sections.iter().any(|section| text.contains(section)),
        Value::Bool(_) | Value::Number(_) => true,
        Value::Null | Value::Object(_) => false,
    }
}

fn legacy_non_object_error(data: &Value) -> &'static str {
    match data {
        Value::Array(_) => "list indices must be integers or slices, not str",
        Value::String(_) => "string indices must be integers, not 'str'",
        Value::Bool(_) => "argument of type 'bool' is not iterable",
        Value::Number(number) if number.is_i64() || number.is_u64() => {
            "argument of type 'int' is not iterable"
        }
        Value::Number(_) => "argument of type 'float' is not iterable",
        Value::Null | Value::Object(_) => "Invalid JSON in request body",
    }
}

/// Resets settings to defaults and returns the saved payload.
pub fn reset_settings() -> SettingsUpdateResponse {
    let settings = default_settings();
    let saved_payload = save_settings_payload(&settings);
    SettingsUpdateResponse {
        status_code: 200,
        success: true,
        message: Some("Settings reset to defaults".to_string()),
        error: None,
        settings: Some(settings),
        saved_payload: Some(saved_payload),
    }
}

fn update_object(target: &mut Map<String, Value>, updates: &Map<String, Value>) {
    for (key, value) in updates {
        target.insert(key.clone(), value.clone());
    }
}

fn known_sections(defaults: &Value) -> Vec<&str> {
    defaults
        .as_object()
        .map(|object| object.keys().map(String::as_str).collect())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURES: &str = include_str!("../../../../fixtures/web_settings_contract.jsonl");
    const RESPONSE_EXPECTATION_KEYS: &[&str] = &[
        "status_code",
        "success",
        "message",
        "error_contains",
        "saved_file",
        "saved_trailing_newline",
        "raw_contains",
        "json_semantic",
    ];

    fn expected_settings_subset(expected: &Value) -> Value {
        Value::Object(
            expected
                .as_object()
                .unwrap()
                .iter()
                .filter(|(key, _)| !RESPONSE_EXPECTATION_KEYS.contains(&key.as_str()))
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect(),
        )
    }

    fn assert_subset(actual: &Value, expected: &Value) {
        let Some(expected_object) = expected.as_object() else {
            assert_eq!(actual, expected);
            return;
        };
        let actual_object = actual.as_object().unwrap();
        for (key, expected_value) in expected_object {
            if key == "unknown_top_present" {
                assert_eq!(
                    actual_object.contains_key("unknown_top"),
                    expected_value.as_bool().unwrap()
                );
                continue;
            }
            assert_subset(actual_object.get(key).unwrap(), expected_value);
        }
    }

    fn check_merge(case: &Value) {
        let actual = merge_settings(&default_settings(), &case["overrides"]);
        assert_subset(&actual, &case["expect"]);
    }

    fn check_load(case: &Value) {
        let payload = match case["file_state"].as_str().unwrap() {
            "missing" => None,
            "malformed" => Some(case["file_text"].as_str().unwrap().to_string()),
            "json" => Some(serde_json::to_string(&case["file_json"]).unwrap()),
            state => panic!("unknown file_state {state}"),
        };
        let actual = load_settings_payload(&default_settings(), payload.as_deref());
        assert_subset(&actual, &case["expect"]);
    }

    fn check_update(case: &Value) {
        let mut current = default_settings();
        let response = update_settings(&mut current, &case["request_json"]);
        let expected = &case["expect"];
        assert_eq!(
            response.status_code,
            expected["status_code"].as_u64().unwrap() as u16
        );
        assert_eq!(response.success, expected["success"].as_bool().unwrap());
        if let Some(message) = expected.get("message").and_then(Value::as_str) {
            assert_eq!(response.message.as_deref(), Some(message));
        }
        if let Some(error_contains) = expected.get("error_contains").and_then(Value::as_str) {
            assert!(
                response
                    .error
                    .as_deref()
                    .unwrap_or("")
                    .contains(error_contains)
            );
        }
        if response.success {
            assert_subset(
                response.settings.as_ref().unwrap(),
                &expected_settings_subset(expected),
            );
        }
        assert_eq!(
            response.saved_payload.is_some(),
            expected
                .get("saved_file")
                .and_then(Value::as_bool)
                .unwrap_or(false)
        );
        if expected
            .get("saved_trailing_newline")
            .and_then(Value::as_bool)
            .unwrap_or(false)
        {
            assert!(response.saved_payload.as_ref().unwrap().ends_with('\n'));
        }
    }

    fn check_update_raw(case: &Value) {
        let parsed = serde_json::from_str::<Value>(case["request_raw"].as_str().unwrap());
        let response = match parsed {
            Ok(value) => update_settings(&mut default_settings(), &value),
            Err(_) => SettingsUpdateResponse {
                status_code: 400,
                success: false,
                message: None,
                error: Some("Invalid JSON in request body".to_string()),
                settings: None,
                saved_payload: None,
            },
        };
        let expected = &case["expect"];
        assert_eq!(
            response.status_code,
            expected["status_code"].as_u64().unwrap() as u16
        );
        assert_eq!(response.success, expected["success"].as_bool().unwrap());
        if let Some(message) = expected.get("message").and_then(Value::as_str) {
            assert_eq!(response.message.as_deref(), Some(message));
        }
        if let Some(error_contains) = expected.get("error_contains").and_then(Value::as_str) {
            assert!(
                response
                    .error
                    .as_deref()
                    .unwrap_or("")
                    .contains(error_contains)
            );
        }
        if response.success {
            assert_subset(
                response.settings.as_ref().unwrap(),
                &expected_settings_subset(expected),
            );
        }
        assert_eq!(
            response.saved_payload.is_some(),
            expected["saved_file"].as_bool().unwrap()
        );
        if expected
            .get("saved_trailing_newline")
            .and_then(Value::as_bool)
            .unwrap_or(false)
        {
            assert!(response.saved_payload.as_ref().unwrap().ends_with('\n'));
        }
    }

    fn check_reset(case: &Value) {
        let response = reset_settings();
        let expected = &case["expect"];
        assert_eq!(
            response.status_code,
            expected["status_code"].as_u64().unwrap() as u16
        );
        assert_eq!(response.success, expected["success"].as_bool().unwrap());
        assert_eq!(
            response.message.as_deref(),
            Some(expected["message"].as_str().unwrap())
        );
        assert_subset(
            response.settings.as_ref().unwrap(),
            &expected_settings_subset(expected),
        );
        assert_eq!(
            response.saved_payload.is_some(),
            expected["saved_file"].as_bool().unwrap()
        );
    }

    fn check_save(case: &Value) {
        let raw = save_settings_payload(&case["settings"]);
        let expected = &case["expect"];
        assert_eq!(
            raw.ends_with('\n'),
            expected["saved_trailing_newline"].as_bool().unwrap()
        );
        for needle in expected["raw_contains"].as_array().unwrap() {
            assert!(raw.contains(needle.as_str().unwrap()));
        }
        if expected["json_semantic"].as_bool().unwrap() {
            assert_eq!(
                serde_json::from_str::<Value>(&raw).unwrap(),
                case["settings"]
            );
        }
    }

    #[test]
    fn web_settings_follow_parity_fixture_table() {
        for line in FIXTURES.lines() {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let case: Value = serde_json::from_str(line).unwrap();
            match case["operation"].as_str().unwrap() {
                "merge" => check_merge(&case),
                "load" => check_load(&case),
                "update" => check_update(&case),
                "update_raw" => check_update_raw(&case),
                "reset" => check_reset(&case),
                "save" => check_save(&case),
                operation => panic!("unknown operation {operation}"),
            }
        }
    }
}
