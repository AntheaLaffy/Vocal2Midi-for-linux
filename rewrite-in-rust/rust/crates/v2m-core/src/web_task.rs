//! Web task registry state.
//!
//! This module mirrors the registry/start/stop/list portion of
//! `web_task_manager.py::TaskManager` while legacy Python remains the runtime
//! owner. It does not execute pipeline work or emit SocketIO events.

/// Web pipeline task state stored by the registry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebTask {
    pub id: String,
    pub status: String,
    pub progress: i64,
    pub stage: String,
    pub config_repr: String,
    pub audio_file_path: String,
    pub created_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub error: Option<String>,
    pub output_files: Vec<String>,
    pub thread: Option<WebTaskThread>,
    pub stop_event_set: bool,
    pub logs: Vec<String>,
}

/// Minimal thread metadata exposed by `start_task`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebTaskThread {
    pub name: String,
    pub daemon: bool,
    pub started: bool,
}

/// Summary returned by `TaskManager.list_tasks`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebTaskSummary {
    pub id: String,
    pub status: String,
    pub progress: i64,
    pub stage: String,
    pub created_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
}

/// In-memory task registry model.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct WebTaskRegistry {
    tasks: Vec<WebTask>,
}

impl WebTaskRegistry {
    /// Creates an empty task registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a task using injected UUID and clock values.
    pub fn create_task(
        &mut self,
        task_id: &str,
        config_repr: &str,
        audio_file_path: &str,
        created_at: &str,
    ) -> String {
        let task = WebTask {
            id: task_id.to_string(),
            status: "pending".to_string(),
            progress: 0,
            stage: "idle".to_string(),
            config_repr: config_repr.to_string(),
            audio_file_path: audio_file_path.to_string(),
            created_at: created_at.to_string(),
            started_at: None,
            completed_at: None,
            error: None,
            output_files: Vec::new(),
            thread: None,
            stop_event_set: false,
            logs: Vec::new(),
        };
        self.tasks.push(task);
        task_id.to_string()
    }

    /// Starts a pending task and records fake thread metadata.
    pub fn start_task(&mut self, task_id: &str, started_at: &str) -> bool {
        let Some(task) = self.get_task_mut(task_id) else {
            return false;
        };
        if task.status != "pending" {
            return false;
        }

        task.status = "running".to_string();
        task.started_at = Some(started_at.to_string());
        task.thread = Some(WebTaskThread {
            name: format!("Pipeline-{}", first_chars(task_id, 8)),
            daemon: true,
            started: true,
        });
        true
    }

    /// Requests stop for a running task.
    pub fn stop_task(&mut self, task_id: &str) -> bool {
        let Some(task) = self.get_task_mut(task_id) else {
            return false;
        };
        if task.status != "running" {
            return false;
        }
        task.stop_event_set = true;
        true
    }

    /// Returns a task by id.
    pub fn get_task(&self, task_id: &str) -> Option<&WebTask> {
        self.tasks.iter().find(|task| task.id == task_id)
    }

    /// Returns task summaries in creation order.
    pub fn list_tasks(&self) -> Vec<WebTaskSummary> {
        self.tasks
            .iter()
            .map(|task| WebTaskSummary {
                id: task.id.clone(),
                status: task.status.clone(),
                progress: task.progress,
                stage: task.stage.clone(),
                created_at: task.created_at.clone(),
                started_at: task.started_at.clone(),
                completed_at: task.completed_at.clone(),
            })
            .collect()
    }

    fn get_task_mut(&mut self, task_id: &str) -> Option<&mut WebTask> {
        self.tasks.iter_mut().find(|task| task.id == task_id)
    }
}

fn first_chars(value: &str, count: usize) -> String {
    value.chars().take(count).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    const FIXTURES: &str = include_str!("../../../../fixtures/web_task_registry_contract.jsonl");

    fn config_repr(value: &Value) -> String {
        serde_json::to_string(value).unwrap()
    }

    fn create_from_case(registry: &mut WebTaskRegistry, case: &Value) {
        registry.create_task(
            case["task_id"].as_str().unwrap(),
            &config_repr(&case["config"]),
            case["audio_path"].as_str().unwrap(),
            case["times"][0].as_str().unwrap(),
        );
    }

    fn create_multi_task_from_case(registry: &mut WebTaskRegistry, case: &Value) {
        let task_ids = case["task_ids"].as_array().unwrap();
        let times = case["times"].as_array().unwrap();
        let tasks = case["tasks"].as_array().unwrap();

        for ((task_id, time), task) in task_ids.iter().zip(times).zip(tasks) {
            registry.create_task(
                task_id.as_str().unwrap(),
                &config_repr(&task["config"]),
                task["audio_path"].as_str().unwrap(),
                time.as_str().unwrap(),
            );
        }
    }

    fn assert_task_fields(task: &WebTask, expected: &Value) {
        if let Some(value) = expected.get("id") {
            assert_eq!(task.id, value.as_str().unwrap());
        }
        if let Some(value) = expected.get("status") {
            assert_eq!(task.status, value.as_str().unwrap());
        }
        if let Some(value) = expected.get("progress") {
            assert_eq!(task.progress, value.as_i64().unwrap());
        }
        if let Some(value) = expected.get("stage") {
            assert_eq!(task.stage, value.as_str().unwrap());
        }
        if let Some(value) = expected.get("config") {
            assert_eq!(task.config_repr, config_repr(value));
        }
        if let Some(value) = expected.get("audio_file_path") {
            assert_eq!(task.audio_file_path, value.as_str().unwrap());
        }
        if let Some(value) = expected.get("created_at") {
            assert_eq!(task.created_at, value.as_str().unwrap());
        }
        if expected.get("started_at").is_some() {
            assert_eq!(task.started_at.as_deref(), expected["started_at"].as_str());
        }
        if expected.get("completed_at").is_some() {
            assert_eq!(
                task.completed_at.as_deref(),
                expected["completed_at"].as_str()
            );
        }
        if expected.get("error").is_some() {
            assert_eq!(task.error.as_deref(), expected["error"].as_str());
        }
        if let Some(value) = expected.get("output_files") {
            let expected_files: Vec<String> = value
                .as_array()
                .unwrap()
                .iter()
                .map(|item| item.as_str().unwrap().to_string())
                .collect();
            assert_eq!(task.output_files, expected_files);
        }
        if let Some(value) = expected.get("thread") {
            if value.is_null() {
                assert_eq!(task.thread, None);
            }
        }
        if let Some(value) = expected.get("thread_name") {
            assert_eq!(task.thread.as_ref().unwrap().name, value.as_str().unwrap());
        }
        if let Some(value) = expected.get("thread_daemon") {
            assert_eq!(
                task.thread.as_ref().unwrap().daemon,
                value.as_bool().unwrap()
            );
        }
        if let Some(value) = expected.get("thread_started") {
            assert_eq!(
                task.thread.as_ref().unwrap().started,
                value.as_bool().unwrap()
            );
        }
        if let Some(value) = expected.get("stop_event_set") {
            assert_eq!(task.stop_event_set, value.as_bool().unwrap());
        }
        if let Some(value) = expected.get("logs") {
            assert_eq!(task.logs.len(), value.as_array().unwrap().len());
        }
    }

    fn assert_list(registry: &WebTaskRegistry, expected: &Value) {
        let summaries = registry.list_tasks();
        let expected = expected.as_array().unwrap();
        assert_eq!(summaries.len(), expected.len());
        for (summary, expected) in summaries.iter().zip(expected) {
            assert_eq!(summary.id, expected["id"].as_str().unwrap());
            assert_eq!(summary.status, expected["status"].as_str().unwrap());
            assert_eq!(summary.progress, expected["progress"].as_i64().unwrap());
            assert_eq!(summary.stage, expected["stage"].as_str().unwrap());
            assert_eq!(summary.created_at, expected["created_at"].as_str().unwrap());
            assert_eq!(
                summary.started_at.as_deref(),
                expected["started_at"].as_str()
            );
            assert_eq!(
                summary.completed_at.as_deref(),
                expected["completed_at"].as_str()
            );
        }
    }

    #[test]
    fn web_task_registry_follows_parity_fixture_table() {
        for (line_number, line) in FIXTURES.lines().enumerate() {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let case: Value = serde_json::from_str(line).unwrap();
            let case_id = case["case_id"].as_str().unwrap();
            let expected = &case["expect"];
            let mut registry = WebTaskRegistry::new();

            if case.get("tasks").is_some() {
                create_multi_task_from_case(&mut registry, &case);
                if let Some(task_ids) = expected.get("get_task_ids").and_then(Value::as_array) {
                    for task_id in task_ids {
                        let task_id = task_id.as_str().unwrap();
                        assert_eq!(registry.get_task(task_id).unwrap().id, task_id);
                    }
                }
                if let Some(missing_lookup) = expected.get("missing_lookup").and_then(Value::as_str)
                {
                    assert!(registry.get_task(missing_lookup).is_none());
                }
                if let Some(list_expected) = expected.get("list") {
                    assert_list(&registry, list_expected);
                }
                continue;
            }

            let task_id = case["task_id"].as_str().unwrap();

            match case_id {
                "start_missing_task" => {
                    assert_eq!(
                        registry.start_task(task_id, "unused"),
                        expected["start_result"].as_bool().unwrap(),
                        "fixture line {line_number}"
                    );
                    continue;
                }
                "stop_missing_task" => {
                    assert_eq!(
                        registry.stop_task(task_id),
                        expected["stop_result"].as_bool().unwrap(),
                        "fixture line {line_number}"
                    );
                    continue;
                }
                _ => create_from_case(&mut registry, &case),
            }

            if let Some(status) = case.get("initial_status").and_then(Value::as_str) {
                registry.get_task_mut(task_id).unwrap().status = status.to_string();
            }

            if case
                .get("start_before_stop")
                .and_then(Value::as_bool)
                .unwrap_or(false)
            {
                let started_at = case["times"][1].as_str().unwrap();
                assert!(registry.start_task(task_id, started_at));
            }

            if let Some(value) = expected.get("start_result") {
                let started_at = case["times"]
                    .as_array()
                    .and_then(|values| values.get(1))
                    .and_then(Value::as_str)
                    .unwrap_or("unused");
                assert_eq!(
                    registry.start_task(task_id, started_at),
                    value.as_bool().unwrap()
                );
            }
            if let Some(value) = expected.get("stop_result") {
                assert_eq!(registry.stop_task(task_id), value.as_bool().unwrap());
            }
            if let Some(task_expected) = expected.get("task") {
                assert_task_fields(registry.get_task(task_id).unwrap(), task_expected);
            }
            if let Some(list_expected) = expected.get("list") {
                assert_list(&registry, list_expected);
            }
        }
    }

    #[test]
    fn start_and_stop_reject_missing_tasks() {
        let mut registry = WebTaskRegistry::new();
        assert!(!registry.start_task("missing", "2026-07-15T00:00:00"));
        assert!(!registry.stop_task("missing"));
    }
}
