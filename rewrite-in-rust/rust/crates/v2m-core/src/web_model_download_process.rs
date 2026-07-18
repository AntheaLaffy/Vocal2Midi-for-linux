//! Web model-download subprocess planning and output parsing.
//!
//! This module mirrors deterministic helpers in `web_model_download_manager.py`
//! without spawning `download_models.py`, owning task lifecycle, or replacing
//! SocketIO transport.

use serde_json::{Map, Value, json};

const PROXY_ENV_KEYS: &[&str] = &[
    "HTTP_PROXY",
    "HTTPS_PROXY",
    "ALL_PROXY",
    "NO_PROXY",
    "http_proxy",
    "https_proxy",
    "all_proxy",
    "no_proxy",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ProcessModelSpec {
    id: &'static str,
    label: &'static str,
    asset: Option<&'static str>,
    target_name: &'static str,
}

const PROCESS_MODEL_SPECS: &[ProcessModelSpec] = &[
    ProcessModelSpec {
        id: "game",
        label: "GAME",
        asset: Some("GAME-1.0.3-medium-onnx.zip"),
        target_name: "GAME-1.0.3-medium-onnx",
    },
    ProcessModelSpec {
        id: "hfa",
        label: "HubertFA",
        asset: Some("1218_hfa_model_new_dict.zip"),
        target_name: "1218_hfa_model_new_dict",
    },
    ProcessModelSpec {
        id: "rmvpe",
        label: "RMVPE",
        asset: Some("RMVPE.zip"),
        target_name: "RMVPE",
    },
    ProcessModelSpec {
        id: "romaji",
        label: "romajiASR",
        asset: Some("romajiASR.zip"),
        target_name: "romajiASR",
    },
    ProcessModelSpec {
        id: "qwen",
        label: "Qwen3-ASR-1.7B",
        asset: None,
        target_name: "Qwen3-ASR-1.7B",
    },
];

/// Mutable task state used by process-planning and output-parser fixtures.
#[derive(Debug, Clone, PartialEq)]
pub struct ModelDownloadProcessTask {
    /// The task identifier.
    pub task_id: String,
    /// The ordered selected model identifiers.
    pub selected_models: Vec<String>,
    /// The selected Qwen model source.
    pub qwen_source: String,
    /// Whether forced replacement is enabled.
    pub force: bool,
    /// The proxy mode.
    pub proxy_mode: String,
    /// The proxy URL.
    pub proxy_url: String,
    /// The status.
    pub status: String,
    /// The progress.
    pub progress: i64,
    /// The stage.
    pub stage: String,
    /// The creation timestamp.
    pub created_at: String,
    /// The optional start timestamp.
    pub started_at: Option<String>,
    /// The optional completion timestamp.
    pub completed_at: Option<String>,
    /// The optional error message.
    pub error: Option<String>,
    /// The optional subprocess return code.
    pub returncode: Option<i64>,
    /// The ordered logs.
    pub logs: Vec<Value>,
    /// The ordered completed model identifiers.
    pub completed_models: Vec<String>,
    /// The optional active model identifier.
    pub active_model: Option<String>,
}

impl ModelDownloadProcessTask {
    #[cfg(test)]
    fn from_fixture(value: &Value) -> Self {
        Self {
            task_id: string_field(value, "task_id"),
            selected_models: string_array_field(value, "selected_models"),
            qwen_source: string_field(value, "qwen_source"),
            force: value["force"].as_bool().unwrap(),
            proxy_mode: string_field(value, "proxy_mode"),
            proxy_url: string_field(value, "proxy_url"),
            status: string_field(value, "status"),
            progress: value["progress"].as_i64().unwrap(),
            stage: string_field(value, "stage"),
            created_at: string_field(value, "created_at"),
            started_at: optional_string_field(value, "started_at"),
            completed_at: optional_string_field(value, "completed_at"),
            error: optional_string_field(value, "error"),
            returncode: value.get("returncode").and_then(Value::as_i64),
            logs: value["logs"].as_array().unwrap().clone(),
            completed_models: string_array_field(value, "completed_models"),
            active_model: optional_string_field(value, "active_model"),
        }
    }

    #[cfg(test)]
    fn state_value(&self) -> Value {
        let logs = normalize_log_timestamps(&self.logs);
        let mut state = json!({
            "task_id": self.task_id,
            "selected_models": self.selected_models,
            "qwen_source": self.qwen_source,
            "force": self.force,
            "proxy_mode": self.proxy_mode,
            "proxy_url": self.proxy_url,
            "status": self.status,
            "progress": self.progress,
            "stage": self.stage,
            "created_at": self.created_at,
            "started_at": self.started_at,
            "completed_at": self.completed_at,
            "error": self.error,
            "returncode": self.returncode,
            "logs": logs,
            "log_count": self.logs.len(),
            "completed_models": self.completed_models,
            "active_model": self.active_model,
        });
        if let Value::Object(object) = &mut state
            && !self.logs.is_empty()
        {
            object.insert(
                "first_log_message".to_string(),
                self.logs[0]["message"].clone(),
            );
            object.insert(
                "last_log_message".to_string(),
                self.logs[self.logs.len() - 1]["message"].clone(),
            );
        }
        state
    }
}

/// Builds the `download_models.py` command vector.
pub fn build_command(
    task: &ModelDownloadProcessTask,
    python_executable: &str,
    root_dir: &str,
) -> Vec<String> {
    let script = format!("{}/download_models.py", root_dir.trim_end_matches('/'));
    let mut command = vec![python_executable.to_string(), script];
    for model_id in &task.selected_models {
        command.push("--only".to_string());
        command.push(model_id.clone());
    }
    if task
        .selected_models
        .iter()
        .any(|model_id| model_id == "qwen")
    {
        command.push("--qwen-source".to_string());
        command.push(task.qwen_source.clone());
    }
    if task.force {
        command.push("--force".to_string());
    }
    command
}

/// Applies proxy policy to a child-process environment map.
pub fn build_process_env(
    task: &ModelDownloadProcessTask,
    base_env: &Map<String, Value>,
) -> Map<String, Value> {
    let mut env = base_env.clone();
    env.insert(
        "PYTHONUNBUFFERED".to_string(),
        Value::String("1".to_string()),
    );
    if task.proxy_mode == "system" {
        return env;
    }

    for key in PROXY_ENV_KEYS {
        env.remove(*key);
    }

    if task.proxy_mode == "manual" {
        let proxy_url = task.proxy_url.trim().to_string();
        for key in [
            "HTTP_PROXY",
            "HTTPS_PROXY",
            "ALL_PROXY",
            "http_proxy",
            "https_proxy",
            "all_proxy",
        ] {
            env.insert(key.to_string(), Value::String(proxy_url.clone()));
        }
    }
    env
}

/// Models process-group spawn kwargs for POSIX and Windows.
pub fn popen_process_group_kwargs(os_name: &str, create_new_process_group: i64) -> Value {
    if os_name == "nt" {
        json!({"creationflags": create_new_process_group})
    } else {
        json!({"start_new_session": true})
    }
}

/// Handles a fake process output stream using legacy line splitting.
pub fn read_process_output(task: &mut ModelDownloadProcessTask, output: &str) -> Vec<Value> {
    let mut emits = Vec::new();
    let mut buffer = String::new();
    for ch in output.chars() {
        if ch == '\n' || ch == '\r' {
            let line = buffer.trim().to_string();
            buffer.clear();
            if !line.is_empty() {
                handle_output_line(task, &line, &mut emits);
            }
        } else {
            buffer.push(ch);
        }
    }
    let line = buffer.trim().to_string();
    if !line.is_empty() {
        handle_output_line(task, &line, &mut emits);
    }
    emits
}

/// Handles optional fake stdout; `None` mirrors a process with `stdout is None`.
pub fn read_process_output_optional(
    task: &mut ModelDownloadProcessTask,
    output: Option<&str>,
) -> Vec<Value> {
    output
        .map(|output| read_process_output(task, output))
        .unwrap_or_default()
}

/// Applies legacy output-line classification, model guessing, and progress updates.
pub fn handle_output_lines(task: &mut ModelDownloadProcessTask, lines: &[String]) -> Vec<Value> {
    let mut emits = Vec::new();
    for line in lines {
        handle_output_line(task, line, &mut emits);
    }
    emits
}

/// Emits one log entry and applies the 500-entry cap.
pub fn emit_log(task: &mut ModelDownloadProcessTask, message: &str, level: &str) -> Value {
    let entry = json!({
        "task_id": task.task_id,
        "task_type": "model_download",
        "message": message,
        "level": level,
        "timestamp": "__timestamp__",
    });
    task.logs.push(entry.clone());
    if task.logs.len() > 500 {
        let start = task.logs.len() - 500;
        task.logs = task.logs[start..].to_vec();
    }
    emit("log", &task.task_id, entry)
}

fn handle_output_line(task: &mut ModelDownloadProcessTask, line: &str, emits: &mut Vec<Value>) {
    let lowered = line.to_lowercase();
    let mut level = if lowered.contains("failed") || lowered.contains("error") {
        "error"
    } else {
        "info"
    };
    if lowered.contains("ready") || lowered.contains("already") {
        level = "success";
    }
    emits.push(emit_log(task, line, level));

    let active_model = guess_model_from_line(&task.selected_models, line);
    if let Some(active_model) = active_model.as_deref() {
        task.active_model = Some(active_model.to_string());
    }

    if let (Some(percent), Some(active_model)) = (legacy_percent(line), task.active_model.clone()) {
        emit_progress_for_model(task, &active_model, percent, emits);
    }

    if lowered.contains("ready") || lowered.contains("already present") {
        let completed = active_model.or_else(|| task.active_model.clone());
        if let Some(completed) = completed {
            add_completed_model(task, &completed);
            emit_progress_for_model(task, &completed, 100, emits);
        }
    }
}

fn emit_progress_for_model(
    task: &mut ModelDownloadProcessTask,
    model_id: &str,
    model_pct: i64,
    emits: &mut Vec<Value>,
) {
    let total = task.selected_models.len().max(1) as f64;
    let index = task
        .selected_models
        .iter()
        .position(|id| id == model_id)
        .unwrap_or(task.completed_models.len()) as f64;
    let progress = (((index + model_pct as f64 / 100.0) / total) * 100.0) as i64;
    task.progress = 99.min(task.progress.max(progress));
    task.stage = model_label(model_id).unwrap_or("downloading").to_string();
    emits.push(emit(
        "progress",
        &task.task_id,
        json!({
            "task_id": task.task_id,
            "task_type": "model_download",
            "progress": task.progress,
            "stage": task.stage,
        }),
    ));
}

fn guess_model_from_line(selected_models: &[String], line: &str) -> Option<String> {
    let lowered = line.to_lowercase();
    for model_id in selected_models {
        if model_id == "qwen" {
            if lowered.contains("qwen") || lowered.contains("qwen3-asr-1.7b") {
                return Some("qwen".to_string());
            }
            continue;
        }
        let Some(model) = PROCESS_MODEL_SPECS
            .iter()
            .find(|spec| spec.id == model_id.as_str())
        else {
            continue;
        };
        let mut needles = vec![model.id, model.target_name];
        if let Some(asset) = model.asset {
            needles.push(asset);
        }
        if needles
            .iter()
            .any(|needle| lowered.contains(&needle.to_lowercase()))
        {
            return Some(model.id.to_string());
        }
    }
    None
}

fn legacy_percent(line: &str) -> Option<i64> {
    let chars: Vec<char> = line.chars().collect();
    let mut start = 0;
    while start < chars.len() {
        if decimal_digit_value(chars[start]).is_none() {
            start += 1;
            continue;
        }
        let end = (start + 3).min(chars.len());
        let mut digit_end = start;
        let mut value = 0_i64;
        while digit_end < end {
            let Some(digit) = decimal_digit_value(chars[digit_end]) else {
                break;
            };
            value = value * 10 + i64::from(digit);
            digit_end += 1;
        }
        if digit_end < chars.len()
            && chars[digit_end] == '%'
            && is_word_boundary(&chars, start)
            && is_word_boundary(&chars, digit_end + 1)
        {
            return Some(value.clamp(0, 100));
        }
        start = digit_end.max(start + 1);
    }
    None
}

fn is_word_boundary(chars: &[char], index: usize) -> bool {
    let before = index.checked_sub(1).and_then(|idx| chars.get(idx)).copied();
    let after = chars.get(index).copied();
    is_word_char(before) != is_word_char(after)
}

fn is_word_char(ch: Option<char>) -> bool {
    ch.is_some_and(|ch| ch == '_' || ch.is_alphanumeric())
}

fn decimal_digit_value(ch: char) -> Option<u32> {
    ch.to_digit(10).or_else(|| {
        let code = ch as u32;
        DECIMAL_ZERO_CODEPOINTS.iter().find_map(|zero| {
            let offset = code.checked_sub(*zero)?;
            (offset <= 9).then_some(offset)
        })
    })
}

const DECIMAL_ZERO_CODEPOINTS: &[u32] = &[
    0x0660, 0x06F0, 0x07C0, 0x0966, 0x09E6, 0x0A66, 0x0AE6, 0x0B66, 0x0BE6, 0x0C66, 0x0CE6, 0x0D66,
    0x0DE6, 0x0E50, 0x0ED0, 0x0F20, 0x1040, 0x1090, 0x17E0, 0x1810, 0x1946, 0x19D0, 0x1A80, 0x1A90,
    0x1B50, 0x1BB0, 0x1C40, 0x1C50, 0xA620, 0xA8D0, 0xA900, 0xA9D0, 0xA9F0, 0xAA50, 0xABF0, 0xFF10,
    0x104A0, 0x10D30, 0x11066, 0x110F0, 0x11136, 0x111D0, 0x112F0, 0x11450, 0x114D0, 0x11650,
    0x116C0, 0x11730, 0x118E0, 0x11C50, 0x11D50, 0x11DA0, 0x16A60, 0x16AC0, 0x16B50, 0x1D7CE,
    0x1D7D8, 0x1D7E2, 0x1D7EC, 0x1D7F6, 0x1E140, 0x1E2F0, 0x1E950, 0x1FBF0,
];

fn add_completed_model(task: &mut ModelDownloadProcessTask, model_id: &str) {
    if !task
        .completed_models
        .iter()
        .any(|completed| completed == model_id)
    {
        task.completed_models.push(model_id.to_string());
    }
}

fn model_label(model_id: &str) -> Option<&'static str> {
    PROCESS_MODEL_SPECS
        .iter()
        .find(|spec| spec.id == model_id)
        .map(|spec| spec.label)
}

fn emit(event: &str, room: &str, payload: Value) -> Value {
    json!({
        "event": event,
        "room": room,
        "payload": payload,
    })
}

#[cfg(test)]
fn normalize_log_timestamps(logs: &[Value]) -> Vec<Value> {
    logs.iter()
        .map(|entry| {
            let mut normalized = entry.clone();
            if let Value::Object(object) = &mut normalized
                && object.contains_key("timestamp")
            {
                object.insert(
                    "timestamp".to_string(),
                    Value::String("__timestamp__".to_string()),
                );
            }
            normalized
        })
        .collect()
}

#[cfg(test)]
fn normalize_emits(emits: Vec<Value>) -> Vec<Value> {
    emits
        .into_iter()
        .map(|emit| {
            let mut normalized = emit;
            if let Some(payload) = normalized.get_mut("payload")
                && let Value::Object(object) = payload
                && object.contains_key("timestamp")
            {
                object.insert(
                    "timestamp".to_string(),
                    Value::String("__timestamp__".to_string()),
                );
            }
            normalized
        })
        .collect()
}

#[cfg(test)]
fn string_field(value: &Value, key: &str) -> String {
    value[key].as_str().unwrap().to_string()
}

#[cfg(test)]
fn optional_string_field(value: &Value, key: &str) -> Option<String> {
    value.get(key).and_then(Value::as_str).map(str::to_string)
}

#[cfg(test)]
fn string_array_field(value: &Value, key: &str) -> Vec<String> {
    value[key]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap().to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURES: &str =
        include_str!("../../../../fixtures/web_model_download_process_plan_contract.jsonl");

    fn load_cases() -> Vec<Value> {
        FIXTURES
            .lines()
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .map(|line| serde_json::from_str(line).unwrap())
            .collect()
    }

    fn assert_subset(actual: &Value, expected: &Value) {
        match expected {
            Value::Object(expected_object) => {
                let actual_object = actual.as_object().unwrap();
                for (key, expected_value) in expected_object {
                    assert_subset(actual_object.get(key).unwrap(), expected_value);
                }
            }
            Value::Array(expected_values) => {
                let actual_values = actual.as_array().unwrap();
                assert_eq!(actual_values.len(), expected_values.len());
                for (actual_value, expected_value) in actual_values.iter().zip(expected_values) {
                    assert_subset(actual_value, expected_value);
                }
            }
            _ => assert_eq!(actual, expected),
        }
    }

    fn process_actual(task: &ModelDownloadProcessTask, emits: Vec<Value>) -> Value {
        json!({
            "task": task.state_value(),
            "emits": normalize_emits(emits),
        })
    }

    #[test]
    fn web_model_download_process_plan_fixtures_match() {
        for case in load_cases() {
            let actual = match case["operation"].as_str().unwrap() {
                "command" => {
                    let task = ModelDownloadProcessTask::from_fixture(&case["task"]);
                    json!({
                        "command": build_command(
                            &task,
                            case["python_executable"].as_str().unwrap(),
                            case["root_dir"].as_str().unwrap(),
                        )
                    })
                }
                "env" => {
                    let task = ModelDownloadProcessTask::from_fixture(&case["task"]);
                    let env = build_process_env(&task, case["base_env"].as_object().unwrap());
                    let expected_keys = case["expect"]["env_subset"].as_object().unwrap();
                    let env_subset: Map<String, Value> = expected_keys
                        .keys()
                        .filter_map(|key| env.get(key).map(|value| (key.clone(), value.clone())))
                        .collect();
                    let present_proxy_keys: Vec<&str> = PROXY_ENV_KEYS
                        .iter()
                        .copied()
                        .filter(|key| env.contains_key(*key))
                        .collect();
                    json!({
                        "env_subset": env_subset,
                        "present_proxy_keys": present_proxy_keys,
                    })
                }
                "popen_kwargs" => json!({
                    "kwargs": popen_process_group_kwargs(
                        case["os_name"].as_str().unwrap(),
                        case.get("create_new_process_group")
                            .and_then(Value::as_i64)
                            .unwrap_or(0),
                    )
                }),
                "handle_lines" => {
                    let mut task = ModelDownloadProcessTask::from_fixture(&case["task"]);
                    let lines: Vec<String> = string_array_field(&case, "lines");
                    let emits = handle_output_lines(&mut task, &lines);
                    process_actual(&task, emits)
                }
                "read_output" => {
                    let mut task = ModelDownloadProcessTask::from_fixture(&case["task"]);
                    let emits = read_process_output_optional(&mut task, case["output"].as_str());
                    process_actual(&task, emits)
                }
                "log_cap" => {
                    let mut task = ModelDownloadProcessTask::from_fixture(&case["task"]);
                    let prelog_count = case["prelog_count"].as_u64().unwrap();
                    task.logs = (0..prelog_count)
                        .map(|index| {
                            json!({
                                "task_id": task.task_id,
                                "task_type": "model_download",
                                "message": format!("old-{index}"),
                                "level": "info",
                                "timestamp": format!("old-ts-{index}"),
                            })
                        })
                        .collect();
                    let emit = emit_log(
                        &mut task,
                        case["message"].as_str().unwrap(),
                        case["level"].as_str().unwrap(),
                    );
                    process_actual(&task, vec![emit])
                }
                other => panic!("unknown operation {other:?}"),
            };
            assert_subset(&actual, &case["expect"]);
        }
    }

    #[test]
    fn legacy_percent_boundary_matches_python_regex_edges() {
        assert_eq!(legacy_percent("50%"), None);
        assert_eq!(legacy_percent("progress 50% done"), None);
        assert_eq!(legacy_percent("50%x"), Some(50));
        assert_eq!(legacy_percent("abc50%z"), None);
        assert_eq!(legacy_percent("100%x"), Some(100));
        assert_eq!(legacy_percent("120%x"), Some(100));
        assert_eq!(legacy_percent("GAME 50%完成"), Some(50));
        assert_eq!(legacy_percent("GAME进度50%x"), None);
        assert_eq!(legacy_percent("GAME ٩٠%完成"), Some(90));
        assert_eq!(legacy_percent("GAME ５０%完成"), Some(50));
    }
}
