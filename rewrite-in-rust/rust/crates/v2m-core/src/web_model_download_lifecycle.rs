//! Web model-download task lifecycle state.
//!
//! This module mirrors `ModelDownloadManager` task registry/start/stop state
//! behavior while legacy Python remains the runtime owner for Flask, SocketIO,
//! subprocess execution, and OS process termination.

use serde_json::Value;
#[cfg(test)]
use serde_json::json;

/// Fake thread metadata exposed by lifecycle fixtures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelDownloadThreadMeta {
    pub name: String,
    pub daemon: bool,
    pub started: bool,
    pub target_called: bool,
}

/// Model-download task state used by lifecycle fixtures.
#[derive(Debug, Clone, PartialEq)]
pub struct ModelDownloadLifecycleTask {
    pub task_id: String,
    pub selected_models: Vec<String>,
    pub qwen_source: String,
    pub force: bool,
    pub proxy_mode: String,
    pub proxy_url: String,
    pub status: String,
    pub progress: i64,
    pub stage: String,
    pub created_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub error: Option<String>,
    pub returncode: Option<i64>,
    pub logs: Vec<Value>,
    pub stop_event_present: bool,
    pub stop_event_set: bool,
    pub thread: Option<ModelDownloadThreadMeta>,
}

/// Input used to create a model-download lifecycle task.
#[derive(Debug, Clone, Copy)]
pub struct CreateLifecycleTaskInput<'a> {
    pub selected_models: &'a [String],
    pub qwen_source: &'a str,
    pub force: bool,
    pub proxy_mode: Option<&'a str>,
    pub proxy_url: Option<&'a str>,
    pub task_id: &'a str,
    pub created_at: &'a str,
}

/// Input used to start a model-download lifecycle task.
#[derive(Debug, Clone, Copy)]
pub struct StartLifecycleTaskInput<'a> {
    pub selected_models: &'a [String],
    pub qwen_source: &'a str,
    pub force: bool,
    pub proxy_mode: &'a str,
    pub proxy_url: &'a str,
    pub task_id: &'a str,
    pub created_at: &'a str,
    pub started_at: &'a str,
}

impl ModelDownloadLifecycleTask {
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
            stop_event_present: value["stop_event_present"].as_bool().unwrap(),
            stop_event_set: value["stop_event_set"].as_bool().unwrap(),
            thread: None,
        }
    }

    #[cfg(test)]
    fn state_value(&self) -> Value {
        json!({
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
            "logs": self.logs,
            "stop_event_present": self.stop_event_present,
            "stop_event_set": self.stop_event_set,
            "thread": self.thread.as_ref().map(|thread| {
                json!({
                    "name": thread.name,
                    "daemon": thread.daemon,
                    "started": thread.started,
                    "target_called": thread.target_called,
                })
            }),
        })
    }
}

/// In-memory manager state for model-download lifecycle fixtures.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ModelDownloadLifecycleManager {
    pub tasks: Vec<ModelDownloadLifecycleTask>,
    pub active_task_id: Option<String>,
}

impl ModelDownloadLifecycleManager {
    /// Creates an empty lifecycle manager.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a task with injected UUID/time values and registers it.
    pub fn create_task(
        &mut self,
        input: CreateLifecycleTaskInput<'_>,
    ) -> ModelDownloadLifecycleTask {
        let task = ModelDownloadLifecycleTask {
            task_id: input.task_id.to_string(),
            selected_models: input.selected_models.to_vec(),
            qwen_source: input.qwen_source.to_string(),
            force: input.force,
            proxy_mode: input.proxy_mode.unwrap_or("system").to_string(),
            proxy_url: input.proxy_url.unwrap_or("").trim().to_string(),
            status: "pending".to_string(),
            progress: 0,
            stage: "queued".to_string(),
            created_at: input.created_at.to_string(),
            started_at: None,
            completed_at: None,
            error: None,
            returncode: None,
            logs: Vec::new(),
            stop_event_present: true,
            stop_event_set: false,
            thread: None,
        };
        self.tasks.push(task.clone());
        task
    }

    /// Returns a task by id.
    pub fn get_task(&self, task_id: &str) -> Option<&ModelDownloadLifecycleTask> {
        self.tasks.iter().find(|task| task.task_id == task_id)
    }

    /// Returns the active task when its status is pending, running, or stopping.
    pub fn active_task(&self) -> Option<&ModelDownloadLifecycleTask> {
        let active_task_id = self.active_task_id.as_deref()?;
        let task = self.get_task(active_task_id)?;
        is_active_status(&task.status).then_some(task)
    }

    /// Starts a task with fake thread metadata and injected UUID/time values.
    pub fn start_task(
        &mut self,
        input: StartLifecycleTaskInput<'_>,
    ) -> Result<ModelDownloadLifecycleTask, String> {
        if self.active_task().is_some() {
            return Err("A model download task is already running.".to_string());
        }

        self.create_task(CreateLifecycleTaskInput {
            selected_models: input.selected_models,
            qwen_source: input.qwen_source,
            force: input.force,
            proxy_mode: Some(input.proxy_mode),
            proxy_url: Some(input.proxy_url),
            task_id: input.task_id,
            created_at: input.created_at,
        });
        let task_index = self
            .tasks
            .iter()
            .position(|task| task.task_id == input.task_id)
            .expect("created task must exist");
        let task = &mut self.tasks[task_index];
        task.thread = Some(ModelDownloadThreadMeta {
            name: format!("ModelDownload-{}", first_chars(input.task_id, 8)),
            daemon: true,
            started: true,
            target_called: false,
        });
        self.active_task_id = Some(input.task_id.to_string());
        task.status = "running".to_string();
        task.started_at = Some(input.started_at.to_string());
        Ok(task.clone())
    }

    /// Requests stop for a pending or running task with no live process.
    pub fn stop_task(&mut self, task_id: &str) -> bool {
        let Some(task) = self.tasks.iter_mut().find(|task| task.task_id == task_id) else {
            return false;
        };
        if task.status != "pending" && task.status != "running" {
            return false;
        }
        task.status = "stopping".to_string();
        if task.stop_event_present {
            task.stop_event_set = true;
        }
        true
    }

    #[cfg(test)]
    fn registry_ids(&self) -> Vec<String> {
        self.tasks.iter().map(|task| task.task_id.clone()).collect()
    }
}

fn is_active_status(status: &str) -> bool {
    matches!(status, "pending" | "running" | "stopping")
}

fn first_chars(value: &str, count: usize) -> String {
    value.chars().take(count).collect()
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
        include_str!("../../../../fixtures/web_model_download_task_lifecycle_contract.jsonl");

    fn load_cases() -> Vec<Value> {
        FIXTURES
            .lines()
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .map(|line| serde_json::from_str(line).unwrap())
            .collect()
    }

    fn manager_from_case(case: &Value) -> ModelDownloadLifecycleManager {
        let tasks = case
            .get("tasks")
            .and_then(Value::as_array)
            .map(|tasks| {
                tasks
                    .iter()
                    .map(ModelDownloadLifecycleTask::from_fixture)
                    .collect()
            })
            .unwrap_or_default();
        ModelDownloadLifecycleManager {
            tasks,
            active_task_id: optional_string_field(case, "active_task_id"),
        }
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
    fn web_model_download_lifecycle_fixtures_match() {
        for case in load_cases() {
            let actual = match case["operation"].as_str().unwrap() {
                "create_task" => run_create_task(&case),
                "get_task" => run_get_task(&case),
                "active_task" => {
                    let manager = manager_from_case(&case);
                    let task = manager.active_task();
                    json!({"active_task_id": task.map(|task| task.task_id.clone())})
                }
                "active_task_matrix" => run_active_task_matrix(&case),
                "start_task" => run_start_task(&case),
                "stop_task_matrix" => run_stop_task_matrix(&case),
                other => panic!("unknown operation {other:?}"),
            };
            assert_subset(&actual, &case["expect"]);
        }
    }

    fn run_create_task(case: &Value) -> Value {
        let mut manager = manager_from_case(case);
        let mut selected_models = string_array_field(case, "selected_models");
        let task = manager.create_task(CreateLifecycleTaskInput {
            selected_models: &selected_models,
            qwen_source: case["qwen_source"].as_str().unwrap(),
            force: case["force"].as_bool().unwrap(),
            proxy_mode: case.get("proxy_mode").and_then(Value::as_str),
            proxy_url: case.get("proxy_url").and_then(Value::as_str),
            task_id: case["uuid"].as_str().unwrap(),
            created_at: case["times"][0].as_str().unwrap(),
        });
        if let Some(mutated) = case.get("mutate_selected_after") {
            selected_models = string_array_field(&json!({"items": mutated}), "items");
            assert_ne!(task.selected_models, selected_models);
        }
        json!({
            "task": task.state_value(),
            "registered": manager.get_task(&task.task_id).is_some(),
            "active_task_id": manager.active_task_id,
            "registry_ids": manager.registry_ids(),
        })
    }

    fn run_get_task(case: &Value) -> Value {
        let manager = manager_from_case(case);
        let results: Vec<Value> = case["queries"]
            .as_array()
            .unwrap()
            .iter()
            .map(|task_id| {
                let task_id = task_id.as_str().unwrap();
                manager
                    .get_task(task_id)
                    .map(|task| json!({"task_id": task.task_id, "status": task.status}))
                    .unwrap_or_else(|| json!({"task_id": null}))
            })
            .collect();
        json!({"results": results})
    }

    fn run_active_task_matrix(case: &Value) -> Value {
        let mut manager = manager_from_case(case);
        let results: Vec<Value> = case["checks"]
            .as_array()
            .unwrap()
            .iter()
            .map(|check| {
                manager.active_task_id = optional_string_field(check, "active_task_id");
                let task = manager.active_task();
                json!({"active_task_id": task.map(|task| task.task_id.clone())})
            })
            .collect();
        json!({"results": results})
    }

    fn run_start_task(case: &Value) -> Value {
        let mut manager = manager_from_case(case);
        let selected_models = string_array_field(case, "selected_models");
        match manager.start_task(StartLifecycleTaskInput {
            selected_models: &selected_models,
            qwen_source: case["qwen_source"].as_str().unwrap(),
            force: case["force"].as_bool().unwrap(),
            proxy_mode: case["proxy_mode"].as_str().unwrap(),
            proxy_url: case["proxy_url"].as_str().unwrap(),
            task_id: case["uuid"].as_str().unwrap(),
            created_at: case["times"][0].as_str().unwrap(),
            started_at: case["times"][1].as_str().unwrap(),
        }) {
            Ok(task) => json!({
                "task": task.state_value(),
                "active_task_id": manager.active_task_id,
                "registry_ids": manager.registry_ids(),
            }),
            Err(error) => json!({
                "error": error,
                "active_task_id": manager.active_task_id,
                "registry_ids": manager.registry_ids(),
            }),
        }
    }

    fn run_stop_task_matrix(case: &Value) -> Value {
        let mut manager = manager_from_case(case);
        let results: Vec<Value> = case["stops"]
            .as_array()
            .unwrap()
            .iter()
            .map(|task_id| {
                let task_id = task_id.as_str().unwrap();
                let success = manager.stop_task(task_id);
                let task = manager.get_task(task_id).map(|task| {
                    json!({
                        "status": task.status,
                        "stop_event_present": task.stop_event_present,
                        "stop_event_set": task.stop_event_set,
                    })
                });
                json!({
                    "task_id": task_id,
                    "success": success,
                    "task": task,
                })
            })
            .collect();
        json!({"results": results})
    }
}
