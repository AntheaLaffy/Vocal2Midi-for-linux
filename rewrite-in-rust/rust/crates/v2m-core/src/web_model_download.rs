//! Web model-download request and catalog contract.
//!
//! This module mirrors the route-level request/status behavior in
//! `web_server.py` and the pure serialization helpers in
//! `web_model_download_manager.py`. Legacy Python remains the runtime owner for
//! Flask, SocketIO, subprocesses, real marker checks, and downloads.

use serde_json::{Map, Value, json};

const VALID_MODEL_IDS: &[&str] = &["game", "hfa", "rmvpe", "romaji", "qwen"];
const VALID_QWEN_SOURCES: &[&str] = &["auto", "modelscope", "huggingface"];
const VALID_PROXY_MODES: &[&str] = &["system", "manual", "none"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ModelSpec {
    id: &'static str,
    name: &'static str,
    role: &'static str,
    description: &'static str,
    target_path: &'static str,
    marker: &'static str,
    source: &'static str,
}

const MODEL_SPECS: &[ModelSpec] = &[
    ModelSpec {
        id: "game",
        name: "GAME",
        role: "音符与音高提取",
        description: "note and pitch extraction",
        target_path: "experiments/GAME-1.0.3-medium-onnx",
        marker: "encoder.onnx",
        source: "GitHub release v0.1.0",
    },
    ModelSpec {
        id: "hfa",
        name: "HubertFA",
        role: "歌词强制对齐",
        description: "Chinese/Japanese forced alignment",
        target_path: "experiments/1218_hfa_model_new_dict",
        marker: "model.onnx",
        source: "GitHub release v0.1.0",
    },
    ModelSpec {
        id: "rmvpe",
        name: "RMVPE",
        role: "音高曲线与智能切片",
        description: "pitch curve estimation for slicing and USTX export",
        target_path: "experiments/RMVPE",
        marker: "rmvpe.onnx",
        source: "GitHub release v0.1.0",
    },
    ModelSpec {
        id: "romaji",
        name: "romajiASR",
        role: "日文 mora / 罗马音识别",
        description: "Japanese mora ASR",
        target_path: "experiments/romajiASR",
        marker: "model.onnx",
        source: "GitHub release v0.1.0",
    },
    ModelSpec {
        id: "qwen",
        name: "Qwen3-ASR-1.7B",
        role: "中文语音识别",
        description: "Chinese ASR transcription backend",
        target_path: "experiments/Qwen3-ASR-1.7B",
        marker: "*.safetensors / *.bin",
        source: "ModelScope / Hugging Face: Qwen/Qwen3-ASR-1.7B",
    },
];

/// Snapshot of a model-download task as exposed through the Web API.
#[derive(Debug, Clone, PartialEq)]
pub struct ModelDownloadTaskSnapshot {
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
}

impl ModelDownloadTaskSnapshot {
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
        }
    }
}

/// Outcome supplied by the fake task manager for start-route modeling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StartTaskOutcome {
    /// The task manager accepted and started a download task.
    Success {
        /// The new task identifier.
        task_id: String,
        /// The initial task status.
        status: String,
    },
    /// Another download task prevents the request from starting.
    Conflict {
        /// The Python-compatible conflict message.
        error: String,
    },
}

impl StartTaskOutcome {
    #[cfg(test)]
    fn from_fixture(value: Option<&Value>) -> Self {
        let Some(value) = value else {
            return Self::Success {
                task_id: "fake-download-task".to_string(),
                status: "running".to_string(),
            };
        };
        match value["kind"].as_str().unwrap() {
            "success" => Self::Success {
                task_id: string_field(value, "task_id"),
                status: string_field(value, "status"),
            },
            "conflict" => Self::Conflict {
                error: string_field(value, "error"),
            },
            other => panic!("unknown start outcome {other:?}"),
        }
    }
}

/// Builds the Web API model catalog from injected install states.
pub fn model_statuses(installed_ids: &[String]) -> Vec<Value> {
    MODEL_SPECS
        .iter()
        .map(|spec| {
            json!({
                "id": spec.id,
                "name": spec.name,
                "role": spec.role,
                "description": spec.description,
                "target_path": spec.target_path,
                "marker": spec.marker,
                "source": spec.source,
                "installed": installed_ids.iter().any(|id| id == spec.id),
                "required": true,
            })
        })
        .collect()
}

/// Serializes a task like `ModelDownloadManager.serialize_task`.
pub fn serialize_task(task: &ModelDownloadTaskSnapshot) -> Value {
    json!({
        "task_id": task.task_id,
        "task_type": "model_download",
        "status": task.status,
        "progress": task.progress,
        "stage": task.stage,
        "selected_models": task.selected_models,
        "qwen_source": task.qwen_source,
        "force": task.force,
        "proxy_mode": task.proxy_mode,
        "proxy_url": redact_proxy_url(&task.proxy_url),
        "created_at": task.created_at,
        "started_at": task.started_at,
        "completed_at": task.completed_at,
        "error": task.error,
        "returncode": task.returncode,
        "logs": task.logs,
    })
}

/// Redacts credentials from a proxy URL using the legacy string split rules.
pub fn redact_proxy_url(proxy_url: &str) -> String {
    if proxy_url.is_empty() {
        return String::new();
    }
    if !proxy_url.contains('@') {
        return proxy_url.to_string();
    }
    if let Some((scheme, rest)) = proxy_url.split_once("://") {
        let host = rest.rsplit('@').next().unwrap_or(rest);
        format!("{scheme}://***@{host}")
    } else {
        let host = proxy_url.rsplit('@').next().unwrap_or(proxy_url);
        format!("***@{host}")
    }
}

/// Validates selected model ids and download-source/proxy options.
pub fn validate_model_request(
    model_ids: &[String],
    qwen_source: Option<&str>,
    proxy_mode: Option<&str>,
    proxy_url: &str,
) -> Option<String> {
    let unknown: Vec<&str> = model_ids
        .iter()
        .map(String::as_str)
        .filter(|model_id| !VALID_MODEL_IDS.contains(model_id))
        .collect();
    if !unknown.is_empty() {
        return Some(format!("Unknown model id(s): {}", unknown.join(", ")));
    }
    if qwen_source.is_none_or(|value| !VALID_QWEN_SOURCES.contains(&value)) {
        return Some(
            "Invalid qwen_source. Expected one of: auto, modelscope, huggingface.".to_string(),
        );
    }
    if proxy_mode.is_none_or(|value| !VALID_PROXY_MODES.contains(&value)) {
        return Some("Invalid proxy_mode. Expected one of: system, manual, none.".to_string());
    }
    if proxy_mode == Some("manual") {
        let proxy_url = proxy_url.trim();
        if proxy_url.is_empty() {
            return Some("proxy_url is required when proxy_mode is manual.".to_string());
        }
        if !proxy_url.contains("://") {
            return Some(
                "proxy_url must include a scheme, for example http://127.0.0.1:7890.".to_string(),
            );
        }
    }
    None
}

/// Models `GET /api/models/status`.
pub fn status_route_response(
    installed_ids: &[String],
    active_task: Option<&ModelDownloadTaskSnapshot>,
) -> Value {
    let models = model_statuses(installed_ids);
    let installed_count = models
        .iter()
        .filter(|model| model["installed"].as_bool().unwrap_or(false))
        .count();
    json!({
        "status_code": 200,
        "success": true,
        "models": models,
        "installed_count": installed_count,
        "missing_count": MODEL_SPECS.len() - installed_count,
        "active_task": active_task.map(serialize_task),
    })
}

/// Models `POST /api/models/download`.
///
/// # Panics
///
/// Panics only if the internally normalized Qwen source is absent after request
/// validation has supplied its `auto` default.
pub fn start_route_response(
    request: &Value,
    model_statuses: &[Value],
    start_outcome: StartTaskOutcome,
) -> Value {
    let empty_data = Map::new();
    let data = match request.as_object() {
        Some(data) => data,
        None if !py_truthy(request) => &empty_data,
        None => return start_error(400, "models must be a list of model ids", Value::Null),
    };

    let force = data.get("force").is_some_and(py_truthy);
    let qwen_source = request_string_field(data, "qwen_source", "auto");
    let proxy_mode = request_string_field(data, "proxy_mode", "system");
    let proxy_url = request_proxy_url(data);

    let model_ids = match selected_model_ids(data, model_statuses) {
        Ok(model_ids) => model_ids,
        Err(error) => return start_error(400, error, Value::Null),
    };

    let selected_models = dedupe_preserving_order(model_ids);
    if selected_models.is_empty() {
        return start_error(400, "No models selected for download", Value::Null);
    }

    if let Some(error) = validate_model_request(
        &selected_models,
        qwen_source.as_deref(),
        proxy_mode.as_deref(),
        &proxy_url,
    ) {
        return start_error(400, error, Value::Null);
    }

    let captured = json!({
        "selected_models": selected_models,
        "qwen_source": qwen_source.as_deref().unwrap(),
        "force": force,
        "proxy_mode": proxy_mode.as_deref().unwrap(),
        "proxy_url": proxy_url,
    });

    match start_outcome {
        StartTaskOutcome::Success { task_id, status } => json!({
            "status_code": 200,
            "json": {
                "success": true,
                "task_id": task_id,
                "status": status,
                "message": "Model download task started",
            },
            "captured": captured,
        }),
        StartTaskOutcome::Conflict { error } => json!({
            "status_code": 409,
            "json": {
                "success": false,
                "error": error,
            },
            "captured": captured,
        }),
    }
}

/// Models `GET /api/models/download/status/<task_id>`.
pub fn status_lookup_route_response(task: Option<&ModelDownloadTaskSnapshot>) -> Value {
    let Some(task) = task else {
        return json!({
            "status_code": 404,
            "json": {
                "success": false,
                "error": "Download task not found",
            },
        });
    };

    let mut response = match serialize_task(task) {
        Value::Object(map) => map,
        _ => unreachable!("serialized task is an object"),
    };
    response.insert("success".to_string(), Value::Bool(true));
    json!({
        "status_code": 200,
        "json": Value::Object(response),
    })
}

/// Models `POST /api/models/download/stop`.
pub fn stop_route_response(
    request: &Value,
    task: Option<&ModelDownloadTaskSnapshot>,
    stop_success: bool,
) -> Value {
    let task_id = request
        .as_object()
        .and_then(|data| data.get("task_id"))
        .filter(|value| py_truthy(value));
    if task_id.is_none() {
        return json!({
            "status_code": 400,
            "json": {
                "success": false,
                "error": "Missing task_id parameter",
            },
        });
    }

    let Some(task) = task else {
        return json!({
            "status_code": 404,
            "json": {
                "success": false,
                "error": "Download task not found",
            },
        });
    };

    if !stop_success {
        return json!({
            "status_code": 400,
            "json": {
                "success": false,
                "error": format!("Download task cannot be stopped (current status: {})", task.status),
            },
        });
    }

    json!({
        "status_code": 200,
        "json": {
            "success": true,
            "status": "stopping",
            "message": "Stop request sent",
        },
    })
}

fn start_error(status_code: u16, error: impl Into<String>, captured: Value) -> Value {
    json!({
        "status_code": status_code,
        "json": {
            "success": false,
            "error": error.into(),
        },
        "captured": captured,
    })
}

fn selected_model_ids(
    data: &Map<String, Value>,
    model_statuses: &[Value],
) -> Result<Vec<String>, &'static str> {
    match data.get("models") {
        None | Some(Value::Null) => Ok(model_statuses
            .iter()
            .filter(|model| !model["installed"].as_bool().unwrap_or(false))
            .map(|model| model["id"].as_str().unwrap().to_string())
            .collect()),
        Some(Value::Array(values)) => values
            .iter()
            .map(|value| {
                value
                    .as_str()
                    .map(str::to_string)
                    .ok_or("models must be a list of model ids")
            })
            .collect(),
        Some(_) => Err("models must be a list of model ids"),
    }
}

fn dedupe_preserving_order(model_ids: Vec<String>) -> Vec<String> {
    let mut selected = Vec::new();
    for model_id in model_ids {
        if !selected.contains(&model_id) {
            selected.push(model_id);
        }
    }
    selected
}

fn request_string_field(data: &Map<String, Value>, key: &str, default: &str) -> Option<String> {
    match data.get(key) {
        None => Some(default.to_string()),
        Some(Value::String(value)) => Some(value.clone()),
        Some(_) => None,
    }
}

fn request_proxy_url(data: &Map<String, Value>) -> String {
    let Some(value) = data.get("proxy_url") else {
        return String::new();
    };
    if !py_truthy(value) {
        return String::new();
    }
    match value {
        Value::String(value) => value.clone(),
        Value::Bool(true) => "True".to_string(),
        Value::Number(number) => number.to_string(),
        other => other.to_string(),
    }
}

fn py_truthy(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(value) => *value,
        Value::Number(number) => number.as_f64().is_some_and(|value| value != 0.0),
        Value::String(value) => !value.is_empty(),
        Value::Array(value) => !value.is_empty(),
        Value::Object(value) => !value.is_empty(),
    }
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
        include_str!("../../../../fixtures/web_model_download_request_catalog_contract.jsonl");

    fn load_cases() -> Vec<Value> {
        FIXTURES
            .lines()
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .map(|line| serde_json::from_str(line).unwrap())
            .collect()
    }

    fn string_array(value: Option<&Value>) -> Vec<String> {
        value
            .and_then(Value::as_array)
            .map(|values| {
                values
                    .iter()
                    .map(|value| value.as_str().unwrap().to_string())
                    .collect()
            })
            .unwrap_or_default()
    }

    fn task_from_case(value: Option<&Value>) -> Option<ModelDownloadTaskSnapshot> {
        value
            .filter(|value| !value.is_null())
            .map(ModelDownloadTaskSnapshot::from_fixture)
    }

    fn model_statuses_from_case(case: &Value) -> Vec<Value> {
        if let Some(statuses) = case.get("model_statuses").and_then(Value::as_array) {
            return statuses.clone();
        }
        model_statuses(&[])
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

    #[test]
    fn web_model_download_request_catalog_fixtures_match() {
        for case in load_cases() {
            let actual = match case["operation"].as_str().unwrap() {
                "status_route" => {
                    let installed = string_array(case.get("installed"));
                    let active_task = task_from_case(case.get("active_task"));
                    status_route_response(&installed, active_task.as_ref())
                }
                "start_route" => start_route_response(
                    &case["request"],
                    &model_statuses_from_case(&case),
                    StartTaskOutcome::from_fixture(case.get("start_outcome")),
                ),
                "status_lookup_route" => {
                    let task = task_from_case(case.get("task"));
                    status_lookup_route_response(task.as_ref())
                }
                "stop_route" => {
                    let task = task_from_case(case.get("task"));
                    stop_route_response(
                        &case["request"],
                        task.as_ref(),
                        case.get("stop_success")
                            .and_then(Value::as_bool)
                            .unwrap_or(false),
                    )
                }
                other => panic!("unknown operation {other:?}"),
            };
            assert_subset(&actual, &case["expect"]);
        }
    }

    #[test]
    fn proxy_redaction_preserves_legacy_string_splits() {
        assert_eq!(redact_proxy_url(""), "");
        assert_eq!(redact_proxy_url("http://host:7890"), "http://host:7890");
        assert_eq!(
            redact_proxy_url("http://user:pass@host:7890"),
            "http://***@host:7890"
        );
        assert_eq!(redact_proxy_url("user@host:7890"), "***@host:7890");
        assert_eq!(redact_proxy_url("a@b@host:7890"), "***@host:7890");
    }
}
