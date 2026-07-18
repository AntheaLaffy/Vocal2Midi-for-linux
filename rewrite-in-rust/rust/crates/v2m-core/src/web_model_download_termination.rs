//! Web model-download process termination decisions.
//!
//! This module mirrors `_terminate_process_tree` and the live-process
//! `stop_task` branch with fake OS/process inputs. It never sends signals,
//! invokes `taskkill`, or terminates a real process.

use serde_json::{Value, json};

/// Fake child process used by termination fixtures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FakeTerminationProcess {
    /// The pid.
    pub pid: i64,
    /// The optional poll.
    pub poll: Option<i64>,
    /// The ordered calls.
    pub calls: Vec<String>,
}

impl FakeTerminationProcess {
    fn is_live(&self) -> bool {
        self.poll.is_none()
    }
}

/// Fake OS/API outcomes used by termination fixtures.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TerminationEnvironment<'a> {
    /// The os name.
    pub os_name: &'a str,
    /// The optional killpg outcome.
    pub killpg_outcome: Option<&'a str>,
    /// The optional taskkill outcome.
    pub taskkill_outcome: Option<&'a str>,
}

/// Result of a fake `_terminate_process_tree` call.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TerminationTrace {
    /// The ordered killpg calls.
    pub killpg_calls: Vec<Value>,
    /// The ordered taskkill calls.
    pub taskkill_calls: Vec<Value>,
    /// The ordered process calls.
    pub process_calls: Vec<String>,
}

/// Fake task state for the live-process `stop_task` branch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelDownloadTerminationTask {
    /// The task identifier.
    pub task_id: String,
    /// The status.
    pub status: String,
    /// Whether a stop event is present.
    pub stop_event_present: bool,
    /// Whether the stop event is set.
    pub stop_event_set: bool,
    /// The optional process.
    pub process: Option<FakeTerminationProcess>,
}

/// Result of the fake live-process `stop_task` branch.
#[derive(Debug, Clone, PartialEq)]
pub struct StopTaskTrace {
    /// Whether the operation succeeded.
    pub success: bool,
    /// The ordered recorded process-termination calls.
    pub termination_calls: Vec<Value>,
}

/// Models `ModelDownloadManager._terminate_process_tree`.
pub fn terminate_process_tree(
    process: &mut FakeTerminationProcess,
    force: bool,
    env: TerminationEnvironment<'_>,
) -> TerminationTrace {
    let mut trace = TerminationTrace::default();
    if !process.is_live() {
        return trace;
    }

    if env.os_name == "nt" {
        let mut command = vec![
            "taskkill".to_string(),
            "/PID".to_string(),
            process.pid.to_string(),
            "/T".to_string(),
        ];
        if force {
            command.push("/F".to_string());
        }
        trace.taskkill_calls.push(json!({
            "command": command,
            "stdout_devnull": true,
            "stderr_devnull": true,
            "check": false,
        }));
        if env.taskkill_outcome == Some("os_error") {
            if force {
                process.calls.push("kill".to_string());
            } else {
                process.calls.push("terminate".to_string());
            }
        }
        trace.process_calls = process.calls.clone();
        return trace;
    }

    let signal = if force { "SIGKILL" } else { "SIGTERM" };
    trace.killpg_calls.push(json!({
        "pid": process.pid,
        "signal": signal,
    }));
    match env.killpg_outcome {
        Some("process_lookup_error") => {}
        Some("os_error") => {
            if force {
                process.calls.push("kill".to_string());
            } else {
                process.calls.push("terminate".to_string());
            }
        }
        _ => {}
    }
    trace.process_calls = process.calls.clone();
    trace
}

/// Models `ModelDownloadManager.stop_task` for tasks that may have a process.
pub fn stop_task_with_process(
    task: &mut ModelDownloadTerminationTask,
    terminate_raises_oserror: bool,
) -> StopTaskTrace {
    let mut trace = StopTaskTrace {
        success: false,
        termination_calls: Vec::new(),
    };
    if task.status != "pending" && task.status != "running" {
        return trace;
    }

    task.status = "stopping".to_string();
    if task.stop_event_present {
        task.stop_event_set = true;
    }

    if let Some(process) = &task.process
        && process.poll.is_none()
    {
        trace
            .termination_calls
            .push(json!({"pid": process.pid, "force": false}));
        if terminate_raises_oserror {
            return trace;
        }
    }

    trace.success = true;
    trace
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURES: &str =
        include_str!("../../../../fixtures/web_model_download_process_termination_contract.jsonl");

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
    fn web_model_download_termination_fixtures_match() {
        for case in load_cases() {
            let actual = match case["operation"].as_str().unwrap() {
                "terminate" => run_terminate(&case),
                "stop_task" => run_stop_task(&case),
                other => panic!("unknown operation {other:?}"),
            };
            assert_subset(&actual, &case["expect"]);
        }
    }

    fn run_terminate(case: &Value) -> Value {
        let mut process = process_from_fixture(&case["process"]);
        let trace = terminate_process_tree(
            &mut process,
            case["force"].as_bool().unwrap(),
            TerminationEnvironment {
                os_name: case["os_name"].as_str().unwrap(),
                killpg_outcome: case.get("killpg_outcome").and_then(Value::as_str),
                taskkill_outcome: case.get("taskkill_outcome").and_then(Value::as_str),
            },
        );
        json!({
            "killpg_calls": trace.killpg_calls,
            "taskkill_calls": trace.taskkill_calls,
            "process_calls": trace.process_calls,
        })
    }

    fn run_stop_task(case: &Value) -> Value {
        let mut task =
            task_from_fixture(&case["task"], Some(process_from_fixture(&case["process"])));
        let trace = stop_task_with_process(
            &mut task,
            case.get("terminate_raises_oserror")
                .and_then(Value::as_bool)
                .unwrap_or(false),
        );
        json!({
            "success": trace.success,
            "task": {
                "status": task.status,
                "stop_event_present": task.stop_event_present,
                "stop_event_set": task.stop_event_set,
            },
            "termination_calls": trace.termination_calls,
        })
    }

    fn process_from_fixture(value: &Value) -> FakeTerminationProcess {
        FakeTerminationProcess {
            pid: value["pid"].as_i64().unwrap(),
            poll: value.get("poll").and_then(Value::as_i64),
            calls: Vec::new(),
        }
    }

    fn task_from_fixture(
        value: &Value,
        process: Option<FakeTerminationProcess>,
    ) -> ModelDownloadTerminationTask {
        ModelDownloadTerminationTask {
            task_id: value["task_id"].as_str().unwrap().to_string(),
            status: value["status"].as_str().unwrap().to_string(),
            stop_event_present: value
                .get("stop_event_present")
                .and_then(Value::as_bool)
                .unwrap_or(true),
            stop_event_set: value
                .get("stop_event_set")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            process,
        }
    }
}
