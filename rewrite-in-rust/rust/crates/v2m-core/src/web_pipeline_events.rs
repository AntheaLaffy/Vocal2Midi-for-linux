//! Web pipeline execution event state.
//!
//! This module mirrors `web_task_manager.py::TaskManager._execute_pipeline` for
//! fixture-backed task state, log/progress events, status changes, and output
//! collection while legacy Python remains the runtime owner.

/// Log entry emitted during pipeline execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipelineLog {
    pub task_id: String,
    pub message: String,
    pub level: String,
    pub timestamp: String,
}

/// Progress event emitted during pipeline execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipelineProgressEvent {
    pub task_id: String,
    pub progress: i64,
    pub stage: String,
}

/// Terminal status event emitted during pipeline execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipelineStatusChange {
    pub task_id: String,
    pub status: String,
    pub error: Option<String>,
    pub output_dir: Option<String>,
    pub files: Vec<String>,
}

/// Result object embedded in completed `status_change` SocketIO payloads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipelineStatusResult {
    pub output_dir: String,
    pub files: Vec<String>,
}

/// SocketIO payload emitted by pipeline execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PipelineSocketPayload {
    Log {
        task_id: String,
        message: String,
        level: String,
        timestamp: String,
    },
    Progress {
        task_id: String,
        progress: i64,
        stage: String,
    },
    StatusChange {
        task_id: String,
        status: String,
        error: Option<Option<String>>,
        result: Option<PipelineStatusResult>,
    },
}

/// Ordered SocketIO emit trace.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipelineSocketEvent {
    pub event: String,
    pub room: String,
    pub payload: PipelineSocketPayload,
}

/// Simplified pipeline execution result modeled from Python behavior.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipelineExecutionOutcome {
    pub status: String,
    pub progress: i64,
    pub stage: String,
    pub error: Option<String>,
    pub completed_at_present: bool,
    pub cancel_checker_set: bool,
    pub stdout_restored: bool,
    pub stderr_restored: bool,
    pub output_files: Vec<String>,
    pub logs: Vec<PipelineLog>,
    pub progress_events: Vec<PipelineProgressEvent>,
    pub status_changes: Vec<PipelineStatusChange>,
    pub socket_events: Vec<PipelineSocketEvent>,
}

/// Fake pipeline result selected by a parity fixture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelineRunResult<'a> {
    Completed,
    StopAfterRun,
    KeyboardInterrupt,
    StoppedError(&'a str),
    GenericError(&'a str),
    ConfigError(&'a str),
}

/// Inputs needed to model `_execute_pipeline`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipelineExecutionInput<'a> {
    pub task_id: &'a str,
    pub audio_path: &'a str,
    pub output_dir: &'a str,
    pub language: &'a str,
    pub device: &'a str,
    pub run_result: PipelineRunResult<'a>,
    pub stdout_lines: Vec<&'a str>,
    pub stderr_lines: Vec<&'a str>,
    pub output_files: Vec<&'a str>,
}

/// Simulates the externally visible event/state result of `_execute_pipeline`.
pub fn simulate_pipeline_execution(input: &PipelineExecutionInput<'_>) -> PipelineExecutionOutcome {
    let mut clock = FakeClock::new();
    let mut outcome = PipelineExecutionOutcome {
        status: "running".to_string(),
        progress: 0,
        stage: "idle".to_string(),
        error: None,
        completed_at_present: false,
        cancel_checker_set: false,
        stdout_restored: true,
        stderr_restored: true,
        output_files: Vec::new(),
        logs: Vec::new(),
        progress_events: Vec::new(),
        status_changes: Vec::new(),
        socket_events: Vec::new(),
    };

    push_log(
        &mut outcome,
        input.task_id,
        "正在构建配置参数...",
        "info",
        &mut clock,
    );

    if let PipelineRunResult::ConfigError(message) = input.run_result {
        outcome.status = "failed".to_string();
        outcome.error = Some(format!("Task manager error: {message}"));
        let error = outcome.error.clone();
        push_status(&mut outcome, input.task_id, "failed", Some(error), None);
        return outcome;
    }

    outcome.cancel_checker_set = true;
    push_log(
        &mut outcome,
        input.task_id,
        "=== 开始全自动提取流程 ===",
        "success",
        &mut clock,
    );
    push_log(
        &mut outcome,
        input.task_id,
        &format!("处理文件: {}", file_name(input.audio_path)),
        "info",
        &mut clock,
    );
    push_log(
        &mut outcome,
        input.task_id,
        &format!("目标语言: {}", input.language),
        "info",
        &mut clock,
    );
    push_log(
        &mut outcome,
        input.task_id,
        &format!("计算设备: {}", input.device),
        "info",
        &mut clock,
    );
    push_log(
        &mut outcome,
        input.task_id,
        &format!("保存目录: {}", input.output_dir),
        "info",
        &mut clock,
    );
    push_log(&mut outcome, input.task_id, "", "info", &mut clock);
    outcome.progress = 5;
    outcome.stage = "loading".to_string();
    push_progress(&mut outcome, input.task_id, 5, "loading");
    push_log(
        &mut outcome,
        input.task_id,
        "正在加载模型...",
        "info",
        &mut clock,
    );

    for line in &input.stdout_lines {
        push_log(&mut outcome, input.task_id, line, "info", &mut clock);
    }
    for line in &input.stderr_lines {
        push_log(&mut outcome, input.task_id, line, "info", &mut clock);
    }

    match input.run_result {
        PipelineRunResult::Completed => complete_success(input, &mut outcome, &mut clock),
        PipelineRunResult::StopAfterRun => {
            complete_cancelled_after_stop(input.task_id, &mut outcome, &mut clock);
        }
        PipelineRunResult::KeyboardInterrupt => {
            outcome.status = "cancelled".to_string();
            push_log(
                &mut outcome,
                input.task_id,
                "任务被中断",
                "warning",
                &mut clock,
            );
            push_status(&mut outcome, input.task_id, "cancelled", None, None);
        }
        PipelineRunResult::StoppedError(_) => {
            outcome.status = "cancelled".to_string();
            push_log(
                &mut outcome,
                input.task_id,
                "任务已被停止",
                "warning",
                &mut clock,
            );
            push_status(&mut outcome, input.task_id, "cancelled", Some(None), None);
        }
        PipelineRunResult::GenericError(message) => {
            outcome.status = "failed".to_string();
            outcome.error = Some(message.to_string());
            push_log(
                &mut outcome,
                input.task_id,
                "发生错误:",
                "error",
                &mut clock,
            );
            push_log(&mut outcome, input.task_id, message, "error", &mut clock);
            push_log(&mut outcome, input.task_id, "", "error", &mut clock);
            push_log(
                &mut outcome,
                input.task_id,
                &format!("Traceback (most recent call last): RuntimeError: {message}"),
                "error",
                &mut clock,
            );
            push_status(
                &mut outcome,
                input.task_id,
                "failed",
                Some(Some(message.to_string())),
                None,
            );
        }
        PipelineRunResult::ConfigError(_) => unreachable!(),
    }

    outcome
}

fn complete_success(
    input: &PipelineExecutionInput<'_>,
    outcome: &mut PipelineExecutionOutcome,
    clock: &mut FakeClock,
) {
    outcome.status = "completed".to_string();
    outcome.progress = 100;
    outcome.stage = "done".to_string();
    outcome.completed_at_present = true;
    clock.consume_completed_at();
    outcome.output_files = collect_output_files(input.output_dir, &input.output_files);
    push_log(outcome, input.task_id, "", "info", clock);
    push_log(
        outcome,
        input.task_id,
        "=============================",
        "info",
        clock,
    );
    push_log(
        outcome,
        input.task_id,
        "✓ 全自动提取完成！",
        "success",
        clock,
    );
    push_log(
        outcome,
        input.task_id,
        &format!("输出目录: {}", input.output_dir),
        "success",
        clock,
    );
    push_status(
        outcome,
        input.task_id,
        "completed",
        None,
        Some(PipelineStatusResult {
            output_dir: input.output_dir.to_string(),
            files: outcome.output_files.clone(),
        }),
    );
}

fn complete_cancelled_after_stop(
    task_id: &str,
    outcome: &mut PipelineExecutionOutcome,
    clock: &mut FakeClock,
) {
    outcome.status = "cancelled".to_string();
    push_log(outcome, task_id, "", "warning", clock);
    push_log(outcome, task_id, "⚠ 任务已被用户取消", "warning", clock);
    push_status(outcome, task_id, "cancelled", None, None);
}

fn push_log(
    outcome: &mut PipelineExecutionOutcome,
    task_id: &str,
    message: &str,
    level: &str,
    clock: &mut FakeClock,
) {
    let timestamp = clock.next_timestamp();
    outcome.logs.push(PipelineLog {
        task_id: task_id.to_string(),
        message: message.to_string(),
        level: level.to_string(),
        timestamp: timestamp.clone(),
    });
    outcome.socket_events.push(PipelineSocketEvent {
        event: "log".to_string(),
        room: task_id.to_string(),
        payload: PipelineSocketPayload::Log {
            task_id: task_id.to_string(),
            message: message.to_string(),
            level: level.to_string(),
            timestamp,
        },
    });
}

fn push_progress(
    outcome: &mut PipelineExecutionOutcome,
    task_id: &str,
    progress: i64,
    stage: &str,
) {
    outcome.progress_events.push(PipelineProgressEvent {
        task_id: task_id.to_string(),
        progress,
        stage: stage.to_string(),
    });
    outcome.socket_events.push(PipelineSocketEvent {
        event: "progress".to_string(),
        room: task_id.to_string(),
        payload: PipelineSocketPayload::Progress {
            task_id: task_id.to_string(),
            progress,
            stage: stage.to_string(),
        },
    });
}

fn push_status(
    outcome: &mut PipelineExecutionOutcome,
    task_id: &str,
    status: &str,
    error: Option<Option<String>>,
    result: Option<PipelineStatusResult>,
) {
    outcome.status_changes.push(PipelineStatusChange {
        task_id: task_id.to_string(),
        status: status.to_string(),
        error: error.clone().flatten(),
        output_dir: result.as_ref().map(|value| value.output_dir.clone()),
        files: result
            .as_ref()
            .map(|value| value.files.clone())
            .unwrap_or_default(),
    });
    outcome.socket_events.push(PipelineSocketEvent {
        event: "status_change".to_string(),
        room: task_id.to_string(),
        payload: PipelineSocketPayload::StatusChange {
            task_id: task_id.to_string(),
            status: status.to_string(),
            error,
            result,
        },
    });
}

struct FakeClock {
    counter: u32,
}

impl FakeClock {
    fn new() -> Self {
        Self { counter: 0 }
    }

    fn next_timestamp(&mut self) -> String {
        let timestamp = format!("2026-07-16T12:00:{:02}", self.counter);
        self.counter += 1;
        timestamp
    }

    fn consume_completed_at(&mut self) {
        self.counter += 1;
    }
}

fn collect_output_files(output_dir: &str, files: &[&str]) -> Vec<String> {
    let mut collected = Vec::new();
    for extension in ["mid", "ustx", "txt", "csv"] {
        let mut extension_files = files
            .iter()
            .filter(|file| file.rsplit_once('.').map(|(_, ext)| ext) == Some(extension))
            .collect::<Vec<_>>();
        extension_files.sort();
        for file in extension_files {
            collected.push(format!("{output_dir}/{file}"));
        }
    }
    collected
}

fn file_name(path: &str) -> &str {
    path.rsplit(['/', '\\']).next().unwrap_or(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Map, Value, json};

    const FIXTURES: &str = include_str!("../../../../fixtures/web_pipeline_execution_events.jsonl");

    fn replace_case_placeholder(value: &str, case_dir: &str) -> String {
        value.replace("__case__", case_dir)
    }

    fn string_vec(value: &Value, key: &str, case_dir: &str) -> Vec<String> {
        value[key]
            .as_array()
            .unwrap()
            .iter()
            .map(|item| replace_case_placeholder(item.as_str().unwrap(), case_dir))
            .collect()
    }

    fn str_refs(values: &[String]) -> Vec<&str> {
        values.iter().map(String::as_str).collect()
    }

    fn run_result<'a>(case: &'a Value) -> PipelineRunResult<'a> {
        if let Some(message) = case.get("config_error").and_then(Value::as_str) {
            return PipelineRunResult::ConfigError(message);
        }
        match case["run_result"].as_str().unwrap() {
            "completed" => PipelineRunResult::Completed,
            "stop_after_run" => PipelineRunResult::StopAfterRun,
            "keyboard_interrupt" => PipelineRunResult::KeyboardInterrupt,
            "stopped_error" => {
                PipelineRunResult::StoppedError(case["error_message"].as_str().unwrap())
            }
            "generic_error" => {
                PipelineRunResult::GenericError(case["error_message"].as_str().unwrap())
            }
            value => panic!("unknown run_result {value}"),
        }
    }

    fn expected_logs(value: &Value, case_dir: &str) -> Vec<(String, String)> {
        value["logs"]
            .as_array()
            .unwrap()
            .iter()
            .map(|entry| {
                let entry = entry.as_array().unwrap();
                (
                    replace_case_placeholder(entry[0].as_str().unwrap(), case_dir),
                    entry[1].as_str().unwrap().to_string(),
                )
            })
            .collect()
    }

    fn expected_progress(value: &Value, task_id: &str) -> Vec<PipelineProgressEvent> {
        value["progress_events"]
            .as_array()
            .unwrap()
            .iter()
            .map(|entry| PipelineProgressEvent {
                task_id: task_id.to_string(),
                progress: entry["progress"].as_i64().unwrap(),
                stage: entry["stage"].as_str().unwrap().to_string(),
            })
            .collect()
    }

    fn expected_status_changes(
        value: &Value,
        task_id: &str,
        case_dir: &str,
    ) -> Vec<PipelineStatusChange> {
        value["status_changes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|entry| PipelineStatusChange {
                task_id: task_id.to_string(),
                status: entry["status"].as_str().unwrap().to_string(),
                error: entry
                    .get("error")
                    .and_then(Value::as_str)
                    .map(str::to_string),
                output_dir: entry
                    .get("output_dir")
                    .and_then(Value::as_str)
                    .map(|value| replace_case_placeholder(value, case_dir)),
                files: string_vec(entry, "files", case_dir),
            })
            .collect()
    }

    fn replace_placeholders_in_value(value: &Value, case_dir: &str) -> Value {
        match value {
            Value::String(value) => Value::String(replace_case_placeholder(value, case_dir)),
            Value::Array(values) => Value::Array(
                values
                    .iter()
                    .map(|value| replace_placeholders_in_value(value, case_dir))
                    .collect(),
            ),
            Value::Object(values) => Value::Object(
                values
                    .iter()
                    .map(|(key, value)| {
                        (key.clone(), replace_placeholders_in_value(value, case_dir))
                    })
                    .collect(),
            ),
            value => value.clone(),
        }
    }

    fn socket_events_to_value(events: &[PipelineSocketEvent]) -> Value {
        Value::Array(
            events
                .iter()
                .map(|event| {
                    json!({
                        "event": event.event,
                        "room": event.room,
                        "payload": socket_payload_to_value(&event.payload),
                    })
                })
                .collect(),
        )
    }

    fn socket_payload_to_value(payload: &PipelineSocketPayload) -> Value {
        match payload {
            PipelineSocketPayload::Log {
                task_id,
                message,
                level,
                timestamp,
            } => json!({
                "task_id": task_id,
                "message": message,
                "level": level,
                "timestamp": timestamp,
            }),
            PipelineSocketPayload::Progress {
                task_id,
                progress,
                stage,
            } => json!({
                "task_id": task_id,
                "progress": progress,
                "stage": stage,
            }),
            PipelineSocketPayload::StatusChange {
                task_id,
                status,
                error,
                result,
            } => {
                let mut payload = Map::new();
                payload.insert("task_id".to_string(), json!(task_id));
                payload.insert("status".to_string(), json!(status));
                if let Some(error) = error {
                    payload.insert("error".to_string(), json!(error));
                }
                if let Some(result) = result {
                    payload.insert(
                        "result".to_string(),
                        json!({
                            "output_dir": result.output_dir,
                            "files": result.files,
                        }),
                    );
                }
                Value::Object(payload)
            }
        }
    }

    fn assert_value_matches(actual: &Value, expected: &Value, path: &str) {
        if let Some(expected) = expected.as_str() {
            if let Some(needle) = expected.strip_prefix("__contains__:") {
                let actual = actual.as_str().unwrap_or_default();
                assert!(
                    actual.contains(needle),
                    "{path}: {actual:?} should contain {needle:?}"
                );
                return;
            }
        }

        match (actual, expected) {
            (Value::Array(actual), Value::Array(expected)) => {
                assert_eq!(
                    actual.len(),
                    expected.len(),
                    "{path}: array length mismatch"
                );
                for (index, (actual, expected)) in actual.iter().zip(expected).enumerate() {
                    assert_value_matches(actual, expected, &format!("{path}[{index}]"));
                }
            }
            (Value::Object(actual), Value::Object(expected)) => {
                let mut actual_keys = actual.keys().collect::<Vec<_>>();
                let mut expected_keys = expected.keys().collect::<Vec<_>>();
                actual_keys.sort();
                expected_keys.sort();
                assert_eq!(actual_keys, expected_keys, "{path}: object keys mismatch");
                for (key, expected) in expected {
                    assert_value_matches(&actual[key], expected, &format!("{path}.{key}"));
                }
            }
            _ => assert_eq!(actual, expected, "{path}: value mismatch"),
        }
    }

    fn assert_logs(actual: &[PipelineLog], expected: &[(String, String)]) {
        assert_eq!(actual.len(), expected.len());
        for (actual, (expected_message, expected_level)) in actual.iter().zip(expected) {
            assert_eq!(&actual.level, expected_level);
            if let Some(needle) = expected_message.strip_prefix("__contains__:") {
                assert!(
                    actual.message.contains(needle),
                    "{:?} should contain {:?}",
                    actual.message,
                    needle
                );
            } else {
                assert_eq!(&actual.message, expected_message);
            }
        }
    }

    #[test]
    fn web_pipeline_execution_events_follow_parity_fixture_table() {
        for (line_number, line) in FIXTURES.lines().enumerate() {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let case: Value = serde_json::from_str(line).unwrap();
            let case_id = case["case_id"].as_str().unwrap();
            let case_dir = format!("/tmp/{case_id}");
            let stdout_lines = string_vec(&case, "stdout_lines", &case_dir);
            let stderr_lines = string_vec(&case, "stderr_lines", &case_dir);
            let output_files = string_vec(&case, "output_files", &case_dir);
            let audio_path =
                replace_case_placeholder(case["audio_path"].as_str().unwrap(), &case_dir);
            let output_dir =
                replace_case_placeholder(case["output_dir"].as_str().unwrap(), &case_dir);
            let input = PipelineExecutionInput {
                task_id: case["task_id"].as_str().unwrap(),
                audio_path: &audio_path,
                output_dir: &output_dir,
                language: case["language"].as_str().unwrap(),
                device: case["device"].as_str().unwrap(),
                run_result: run_result(&case),
                stdout_lines: str_refs(&stdout_lines),
                stderr_lines: str_refs(&stderr_lines),
                output_files: str_refs(&output_files),
            };
            let actual = simulate_pipeline_execution(&input);
            let expected = &case["expect"];

            assert_eq!(
                actual.status,
                expected["status"].as_str().unwrap(),
                "line {line_number}"
            );
            assert_eq!(
                actual.progress,
                expected["progress"].as_i64().unwrap(),
                "line {line_number}"
            );
            assert_eq!(
                actual.stage,
                expected["stage"].as_str().unwrap(),
                "line {line_number}"
            );
            assert_eq!(
                actual.error.as_deref(),
                expected["error"].as_str(),
                "line {line_number}"
            );
            assert_eq!(
                actual.completed_at_present,
                expected["completed_at_present"].as_bool().unwrap(),
                "line {line_number}"
            );
            assert_eq!(
                actual.cancel_checker_set,
                expected["cancel_checker_set"].as_bool().unwrap(),
                "line {line_number}"
            );
            assert_eq!(
                actual.stdout_restored,
                expected["stdout_restored"].as_bool().unwrap(),
                "line {line_number}"
            );
            assert_eq!(
                actual.stderr_restored,
                expected["stderr_restored"].as_bool().unwrap(),
                "line {line_number}"
            );
            assert_eq!(
                actual.output_files,
                string_vec(expected, "output_files", &case_dir)
            );
            assert_logs(&actual.logs, &expected_logs(expected, &case_dir));
            assert_eq!(
                actual.progress_events,
                expected_progress(expected, input.task_id)
            );
            assert_eq!(
                actual.status_changes,
                expected_status_changes(expected, input.task_id, &case_dir)
            );
            assert_value_matches(
                &socket_events_to_value(&actual.socket_events),
                &replace_placeholders_in_value(&expected["emit_events"], &case_dir),
                case_id,
            );
        }
    }
}
