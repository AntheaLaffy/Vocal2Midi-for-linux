//! Web model-download execution result handling.
//!
//! This module mirrors the task-visible `_execute_download` result state
//! machine with fake process/socket inputs. Legacy Python remains the runtime
//! owner for subprocess execution, SocketIO delivery, and OS process killing.

use serde_json::{Map, Value, json};

use crate::web_model_download::redact_proxy_url;
use crate::web_model_download_process::{
    ModelDownloadProcessTask, build_command, build_process_env, emit_log,
    read_process_output_optional,
};

/// Fake wait behavior for cancellable process fixtures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FakeWaitOutcome {
    Return(i64),
    Timeout,
}

/// Fake child process behavior used by execution fixtures.
#[derive(Debug, Clone, PartialEq)]
pub struct FakeExecutionProcess {
    pub stdout: Option<String>,
    pub returncode: i64,
    pub poll_after_read: Option<i64>,
    pub wait_plan: Vec<FakeWaitOutcome>,
    pub popen_error: Option<String>,
}

/// Task state plus execution-only runtime handles.
#[derive(Debug, Clone, PartialEq)]
pub struct ModelDownloadExecutionTask {
    pub task: ModelDownloadProcessTask,
    pub stop_event_present: bool,
    pub stop_event_set: bool,
    pub process_assigned: bool,
}

/// Inputs injected into the fake execution seam.
#[derive(Debug, Clone, Copy)]
pub struct ExecuteDownloadInput<'a> {
    pub python_executable: &'a str,
    pub root_dir: &'a str,
    pub base_env: &'a Map<String, Value>,
    pub popen_kwargs: &'a Value,
    pub active_task_id: Option<&'a str>,
}

/// Captured fake Popen call.
#[derive(Debug, Clone, PartialEq)]
pub struct FakePopenCall {
    pub command: Vec<String>,
    pub cwd: String,
    pub env: Map<String, Value>,
    pub kwargs: Value,
}

/// Captured fake wait call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FakeWaitCall {
    pub timeout: Option<i64>,
    pub result: FakeWaitCallResult,
}

/// Fake wait result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FakeWaitCallResult {
    Return(i64),
    Timeout,
}

/// Captured process termination handoff.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FakeTerminationCall {
    pub force: bool,
}

/// Output from the fake execution seam.
#[derive(Debug, Clone, PartialEq)]
pub struct ExecutionResult {
    pub active_task_id: Option<String>,
    pub emits: Vec<Value>,
    pub popen_calls: Vec<FakePopenCall>,
    pub wait_calls: Vec<FakeWaitCall>,
    pub termination_calls: Vec<FakeTerminationCall>,
    pub output_reader_called: bool,
}

impl ExecutionResult {
    fn new(active_task_id: Option<&str>) -> Self {
        Self {
            active_task_id: active_task_id.map(str::to_string),
            emits: Vec::new(),
            popen_calls: Vec::new(),
            wait_calls: Vec::new(),
            termination_calls: Vec::new(),
            output_reader_called: false,
        }
    }
}

/// Executes the fixture-backed `_execute_download` result state machine.
pub fn execute_download(
    execution_task: &mut ModelDownloadExecutionTask,
    process: &FakeExecutionProcess,
    input: ExecuteDownloadInput<'_>,
) -> ExecutionResult {
    let mut result = ExecutionResult::new(input.active_task_id);
    let command = build_command(
        &execution_task.task,
        input.python_executable,
        input.root_dir,
    );
    result.emits.push(emit_log(
        &mut execution_task.task,
        "准备下载模型...",
        "info",
    ));
    result.emits.push(emit_log(
        &mut execution_task.task,
        &command.join(" "),
        "info",
    ));
    result
        .emits
        .push(emit_progress(&mut execution_task.task, 2, "starting"));

    let env = build_process_env(&execution_task.task, input.base_env);
    result.popen_calls.push(FakePopenCall {
        command,
        cwd: input.root_dir.to_string(),
        env,
        kwargs: input.popen_kwargs.clone(),
    });

    if let Some(error) = process.popen_error.as_deref() {
        fail_with_exception(execution_task, &mut result, error);
        cleanup_active_task(execution_task, &mut result);
        return result;
    }

    execution_task.process_assigned = true;
    result.output_reader_called = true;
    if !execution_task.stop_event_set {
        result.emits.extend(read_process_output_optional(
            &mut execution_task.task,
            process.stdout.as_deref(),
        ));
    }

    if execution_task.stop_event_present && execution_task.stop_event_set {
        if process.poll_after_read.is_none() {
            result
                .termination_calls
                .push(FakeTerminationCall { force: false });
            match wait_with_timeout(process, &mut result, 5, 0) {
                FakeWaitCallResult::Timeout => {
                    result
                        .termination_calls
                        .push(FakeTerminationCall { force: true });
                    wait_with_timeout(process, &mut result, 5, 1);
                }
                FakeWaitCallResult::Return(_) => {}
            }
        }
        execution_task.task.status = "cancelled".to_string();
        execution_task.task.stage = "cancelled".to_string();
        execution_task.task.completed_at = Some("__timestamp__".to_string());
        result.emits.push(emit_log(
            &mut execution_task.task,
            "下载任务已停止",
            "warning",
        ));
        result.emits.push(emit_status(&execution_task.task));
        cleanup_active_task(execution_task, &mut result);
        return result;
    }

    result.wait_calls.push(FakeWaitCall {
        timeout: None,
        result: FakeWaitCallResult::Return(process.returncode),
    });
    execution_task.task.returncode = Some(process.returncode);
    execution_task.task.completed_at = Some("__timestamp__".to_string());
    if process.returncode == 0 {
        execution_task.task.status = "completed".to_string();
        execution_task.task.stage = "done".to_string();
        execution_task.task.progress = 100;
        result
            .emits
            .push(emit_progress(&mut execution_task.task, 100, "done"));
        result.emits.push(emit_log(
            &mut execution_task.task,
            "模型下载完成",
            "success",
        ));
    } else {
        execution_task.task.status = "failed".to_string();
        execution_task.task.stage = "failed".to_string();
        execution_task.task.error = Some(format!(
            "download_models.py exited with code {}",
            process.returncode
        ));
        let error = execution_task.task.error.clone().unwrap();
        result
            .emits
            .push(emit_log(&mut execution_task.task, &error, "error"));
    }

    result.emits.push(emit_status(&execution_task.task));
    cleanup_active_task(execution_task, &mut result);
    result
}

fn wait_with_timeout(
    process: &FakeExecutionProcess,
    result: &mut ExecutionResult,
    timeout: i64,
    index: usize,
) -> FakeWaitCallResult {
    let outcome = process
        .wait_plan
        .get(index)
        .cloned()
        .unwrap_or(FakeWaitOutcome::Return(process.returncode));
    let call_result = match outcome {
        FakeWaitOutcome::Return(returncode) => FakeWaitCallResult::Return(returncode),
        FakeWaitOutcome::Timeout => FakeWaitCallResult::Timeout,
    };
    result.wait_calls.push(FakeWaitCall {
        timeout: Some(timeout),
        result: call_result.clone(),
    });
    call_result
}

fn fail_with_exception(
    execution_task: &mut ModelDownloadExecutionTask,
    result: &mut ExecutionResult,
    error: &str,
) {
    execution_task.task.status = "failed".to_string();
    execution_task.task.stage = "failed".to_string();
    execution_task.task.error = Some(error.to_string());
    execution_task.task.completed_at = Some("__timestamp__".to_string());
    result
        .emits
        .push(emit_log(&mut execution_task.task, error, "error"));
    result
        .emits
        .push(emit_log(&mut execution_task.task, "__traceback__", "error"));
    result.emits.push(emit_status(&execution_task.task));
}

fn cleanup_active_task(execution_task: &ModelDownloadExecutionTask, result: &mut ExecutionResult) {
    if result.active_task_id.as_deref() == Some(execution_task.task.task_id.as_str()) {
        result.active_task_id = None;
    }
}

fn emit_progress(task: &mut ModelDownloadProcessTask, progress: i64, stage: &str) -> Value {
    task.progress = progress.clamp(0, 100);
    task.stage = stage.to_string();
    json!({
        "event": "progress",
        "room": task.task_id,
        "payload": {
            "task_id": task.task_id,
            "task_type": "model_download",
            "progress": task.progress,
            "stage": task.stage,
        },
    })
}

fn emit_status(task: &ModelDownloadProcessTask) -> Value {
    json!({
        "event": "status_change",
        "room": task.task_id,
        "payload": serialize_execution_task(task),
    })
}

fn serialize_execution_task(task: &ModelDownloadProcessTask) -> Value {
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

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURES: &str =
        include_str!("../../../../fixtures/web_model_download_execution_result_contract.jsonl");

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

    #[test]
    fn web_model_download_execution_fixtures_match() {
        for case in load_cases() {
            let mut task = execution_task_from_fixture(&case["task"]);
            let process = process_from_fixture(&case["process"]);
            let actual = run_execute_download(&case, &mut task, &process);
            assert_subset(&actual, &case["expect"]);
        }
    }

    fn run_execute_download(
        case: &Value,
        task: &mut ModelDownloadExecutionTask,
        process: &FakeExecutionProcess,
    ) -> Value {
        let empty_env = Map::new();
        let base_env = case
            .get("base_env")
            .and_then(Value::as_object)
            .unwrap_or(&empty_env);
        let result = execute_download(
            task,
            process,
            ExecuteDownloadInput {
                python_executable: case["python_executable"].as_str().unwrap(),
                root_dir: case["root_dir"].as_str().unwrap(),
                base_env,
                popen_kwargs: case.get("popen_kwargs").unwrap_or(&Value::Null),
                active_task_id: case.get("active_task_id").and_then(Value::as_str),
            },
        );
        json!({
            "task": task_state_value(task),
            "active_task_id": result.active_task_id,
            "process_assigned": task.process_assigned,
            "output_reader_called": result.output_reader_called,
            "popen_calls": popen_calls_value(&result.popen_calls, env_keys(case)),
            "wait_calls": wait_calls_value(&result.wait_calls),
            "termination_calls": termination_calls_value(&result.termination_calls),
            "emits": summarize_emits(&result.emits),
        })
    }

    fn execution_task_from_fixture(value: &Value) -> ModelDownloadExecutionTask {
        ModelDownloadExecutionTask {
            task: ModelDownloadProcessTask {
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
            },
            stop_event_present: value
                .get("stop_event_present")
                .and_then(Value::as_bool)
                .unwrap_or(true),
            stop_event_set: value
                .get("stop_event_set")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            process_assigned: false,
        }
    }

    fn process_from_fixture(value: &Value) -> FakeExecutionProcess {
        FakeExecutionProcess {
            stdout: optional_string_field(value, "stdout"),
            returncode: value.get("returncode").and_then(Value::as_i64).unwrap_or(0),
            poll_after_read: value.get("poll_after_read").and_then(Value::as_i64),
            wait_plan: value
                .get("wait_plan")
                .and_then(Value::as_array)
                .map(|plan| {
                    plan.iter()
                        .map(|outcome| match outcome["kind"].as_str().unwrap() {
                            "timeout" => FakeWaitOutcome::Timeout,
                            "return" => FakeWaitOutcome::Return(
                                outcome
                                    .get("returncode")
                                    .and_then(Value::as_i64)
                                    .unwrap_or(0),
                            ),
                            other => panic!("unknown wait outcome {other:?}"),
                        })
                        .collect()
                })
                .unwrap_or_default(),
            popen_error: optional_string_field(value, "popen_error"),
        }
    }

    fn task_state_value(task: &ModelDownloadExecutionTask) -> Value {
        let mut state = json!({
            "task_id": task.task.task_id,
            "selected_models": task.task.selected_models,
            "qwen_source": task.task.qwen_source,
            "force": task.task.force,
            "proxy_mode": task.task.proxy_mode,
            "proxy_url": task.task.proxy_url,
            "status": task.task.status,
            "progress": task.task.progress,
            "stage": task.task.stage,
            "created_at": task.task.created_at,
            "started_at": task.task.started_at,
            "completed_at": task.task.completed_at,
            "error": task.task.error,
            "returncode": task.task.returncode,
            "logs": task.task.logs,
            "log_count": task.task.logs.len(),
            "completed_models": task.task.completed_models,
            "active_model": task.task.active_model,
            "stop_event_present": task.stop_event_present,
            "stop_event_set": task.stop_event_set,
            "process_assigned": task.process_assigned,
        });
        if let Value::Object(object) = &mut state
            && !task.task.logs.is_empty()
        {
            object.insert(
                "first_log_message".to_string(),
                task.task.logs[0]["message"].clone(),
            );
            object.insert(
                "last_log_message".to_string(),
                task.task.logs[task.task.logs.len() - 1]["message"].clone(),
            );
        }
        state
    }

    fn popen_calls_value(calls: &[FakePopenCall], env_keys: Vec<String>) -> Value {
        Value::Array(
            calls
                .iter()
                .map(|call| {
                    let env_subset: Map<String, Value> = env_keys
                        .iter()
                        .filter_map(|key| {
                            call.env.get(key).map(|value| (key.clone(), value.clone()))
                        })
                        .collect();
                    json!({
                        "command": call.command,
                        "cwd": call.cwd,
                        "stdout_pipe": true,
                        "stderr_stdout": true,
                        "text": true,
                        "bufsize": 0,
                        "env_subset": env_subset,
                        "kwargs": call.kwargs,
                    })
                })
                .collect(),
        )
    }

    fn wait_calls_value(calls: &[FakeWaitCall]) -> Value {
        Value::Array(
            calls
                .iter()
                .map(|call| {
                    let result = match call.result {
                        FakeWaitCallResult::Return(returncode) => json!(returncode),
                        FakeWaitCallResult::Timeout => json!("timeout"),
                    };
                    json!({
                        "timeout": call.timeout,
                        "result": result,
                    })
                })
                .collect(),
        )
    }

    fn termination_calls_value(calls: &[FakeTerminationCall]) -> Value {
        Value::Array(
            calls
                .iter()
                .map(|call| json!({"force": call.force}))
                .collect(),
        )
    }

    fn summarize_emits(emits: &[Value]) -> Value {
        let mut logs = Vec::new();
        let mut progress = Vec::new();
        let mut status_changes = Vec::new();
        for emit in emits {
            let event = emit["event"].as_str().unwrap();
            let payload = &emit["payload"];
            match event {
                "log" => logs.push(json!({
                    "message": payload["message"],
                    "level": payload["level"],
                })),
                "progress" => progress.push(json!({
                    "progress": payload["progress"],
                    "stage": payload["stage"],
                })),
                "status_change" => status_changes.push(json!({
                    "status": payload["status"],
                    "stage": payload["stage"],
                    "progress": payload["progress"],
                    "error": payload["error"],
                    "returncode": payload["returncode"],
                    "completed_at": payload["completed_at"],
                    "proxy_url": payload["proxy_url"],
                    "logs_len": payload["logs"].as_array().unwrap().len(),
                })),
                other => panic!("unexpected event {other:?}"),
            }
        }
        json!({
            "events": emits.iter().map(|emit| emit["event"].clone()).collect::<Vec<_>>(),
            "logs": logs,
            "progress": progress,
            "status_changes": status_changes,
        })
    }

    fn env_keys(case: &Value) -> Vec<String> {
        case.get("env_keys")
            .and_then(Value::as_array)
            .map(|keys| {
                keys.iter()
                    .map(|key| key.as_str().unwrap().to_string())
                    .collect()
            })
            .unwrap_or_default()
    }

    fn string_field(value: &Value, key: &str) -> String {
        value[key].as_str().unwrap().to_string()
    }

    fn optional_string_field(value: &Value, key: &str) -> Option<String> {
        value.get(key).and_then(Value::as_str).map(str::to_string)
    }

    fn string_array_field(value: &Value, key: &str) -> Vec<String> {
        value[key]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_str().unwrap().to_string())
            .collect()
    }
}
