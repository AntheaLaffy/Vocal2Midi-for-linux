//! Web filesystem picker path and listing behavior.
//!
//! This module mirrors the local path picker helpers in `web_server.py` for
//! fixture-backed path resolution, root entries, extension parsing, entry
//! filtering, and list response shapes while legacy Python remains the runtime
//! owner.

use std::collections::{BTreeSet, HashSet};

/// Filesystem root entry returned to the Web picker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PickerRoot {
    pub label: String,
    pub path: String,
    pub input_path: String,
}

/// Fake filesystem entry used by fixture-backed listing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PickerEntrySpec {
    pub name: String,
    pub entry_type: String,
}

/// Filesystem entry returned to the Web picker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PickerEntry {
    pub name: String,
    pub entry_type: String,
    pub path: String,
    pub input_path: String,
}

/// Result shape modeled from `GET /api/filesystem/list`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PickerListResponse {
    pub status_code: u16,
    pub success: bool,
    pub error: Option<String>,
    pub mode: Option<String>,
    pub path: Option<String>,
    pub input_path: Option<String>,
    pub parent: Option<String>,
    pub parent_input_path: Option<String>,
    pub entries: Vec<PickerEntry>,
    pub roots: Vec<PickerRoot>,
}

/// Inputs needed to model `GET /api/filesystem/list`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PickerListInput<'a> {
    pub project_root: &'a str,
    pub home_dir: &'a str,
    pub path_text: &'a str,
    pub mode: &'a str,
    pub extensions: &'a str,
    pub path_state: &'a str,
    pub scandir_error: Option<&'a str>,
    pub children: Vec<PickerEntrySpec>,
}

/// Resolves a picker path against the project root.
pub fn resolve_picker_path(project_root: &str, home_dir: &str, path_text: &str) -> String {
    let text = path_text.trim();
    if text.is_empty() {
        return normalize_posix_path(project_root);
    }

    let expanded = expand_home(text, home_dir);
    if is_absolute_posix(&expanded) {
        normalize_posix_path(&expanded)
    } else {
        normalize_posix_path(&join_posix(project_root, &expanded))
    }
}

/// Returns the value written back to the UI for a path.
pub fn input_value_for_path(project_root: &str, path: &str) -> String {
    let project_root = normalize_posix_path(project_root);
    let path = normalize_posix_path(path);
    if path == project_root {
        return ".".to_string();
    }
    let prefix = format!("{project_root}/");
    if let Some(relative) = path.strip_prefix(&prefix) {
        return relative.to_string();
    }
    path
}

/// Parses comma-separated extension filters like the Python picker.
pub fn parse_extensions(raw_extensions: &str) -> Vec<String> {
    raw_extensions
        .split(',')
        .filter_map(|item| {
            let extension = item.trim().to_lowercase();
            if extension.is_empty() {
                None
            } else if extension.starts_with('.') {
                Some(extension)
            } else {
                Some(format!(".{extension}"))
            }
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

/// Returns fake root entries for the Unix fixture platform.
pub fn filesystem_roots(project_root: &str, home_dir: &str) -> Vec<PickerRoot> {
    let candidates = [
        ("项目目录", normalize_posix_path(project_root)),
        ("用户目录", normalize_posix_path(home_dir)),
        ("系统根目录", "/".to_string()),
    ];
    let mut roots = Vec::new();
    let mut seen = HashSet::new();
    for (label, path) in candidates {
        if seen.insert(path.clone()) {
            roots.push(PickerRoot {
                label: label.to_string(),
                input_path: input_value_for_path(project_root, &path),
                path,
            });
        }
    }
    roots
}

/// Simulates `GET /api/filesystem/list` for fixture-backed entries.
pub fn list_filesystem(input: &PickerListInput<'_>) -> PickerListResponse {
    if input.mode != "directory" && input.mode != "file" {
        return PickerListResponse {
            status_code: 400,
            success: false,
            error: Some("mode must be directory or file".to_string()),
            mode: None,
            path: None,
            input_path: None,
            parent: None,
            parent_input_path: None,
            entries: Vec::new(),
            roots: Vec::new(),
        };
    }

    let requested_path = resolve_picker_path(input.project_root, input.home_dir, input.path_text);
    if input.path_state == "missing" {
        return PickerListResponse {
            status_code: 404,
            success: false,
            error: Some("Path does not exist".to_string()),
            mode: None,
            path: None,
            input_path: None,
            parent: None,
            parent_input_path: None,
            entries: Vec::new(),
            roots: Vec::new(),
        };
    }

    let current_path = if input.path_state == "file" {
        parent_path(&requested_path)
    } else {
        requested_path
    };

    if let Some(error) = input.scandir_error {
        return PickerListResponse {
            status_code: 400,
            success: false,
            error: Some(format!("Cannot read directory: {error}")),
            mode: None,
            path: None,
            input_path: None,
            parent: None,
            parent_input_path: None,
            entries: Vec::new(),
            roots: Vec::new(),
        };
    }

    let extensions = parse_extensions(input.extensions)
        .into_iter()
        .collect::<BTreeSet<_>>();
    let mut entries = input
        .children
        .iter()
        .filter_map(|entry| {
            picker_entry(
                input.project_root,
                &current_path,
                entry,
                input.mode,
                &extensions,
            )
        })
        .collect::<Vec<_>>();
    entries.sort_by_key(|entry| (entry.entry_type != "directory", entry.name.to_lowercase()));

    let parent = parent_path(&current_path);
    let parent = if parent == current_path {
        None
    } else {
        Some(parent)
    };
    let parent_input_path = parent
        .as_ref()
        .map(|parent| input_value_for_path(input.project_root, parent));

    PickerListResponse {
        status_code: 200,
        success: true,
        error: None,
        mode: Some(input.mode.to_string()),
        path: Some(current_path.clone()),
        input_path: Some(input_value_for_path(input.project_root, &current_path)),
        parent,
        parent_input_path,
        entries,
        roots: filesystem_roots(input.project_root, input.home_dir),
    }
}

fn picker_entry(
    project_root: &str,
    current_path: &str,
    entry: &PickerEntrySpec,
    mode: &str,
    extensions: &BTreeSet<String>,
) -> Option<PickerEntry> {
    let is_dir = entry.entry_type == "directory";
    let is_file = entry.entry_type == "file";
    if !(is_dir || mode == "file" && is_file) {
        return None;
    }

    let path = join_posix(current_path, &entry.name);
    if is_file && !extensions.is_empty() && !extensions.contains(&suffix_lower(&path)) {
        return None;
    }

    Some(PickerEntry {
        name: entry.name.clone(),
        entry_type: if is_dir { "directory" } else { "file" }.to_string(),
        input_path: input_value_for_path(project_root, &path),
        path,
    })
}

fn expand_home(path: &str, home_dir: &str) -> String {
    if path == "~" {
        normalize_posix_path(home_dir)
    } else if let Some(rest) = path.strip_prefix("~/") {
        join_posix(home_dir, rest)
    } else {
        path.to_string()
    }
}

fn is_absolute_posix(path: &str) -> bool {
    path.starts_with('/')
}

fn join_posix(base: &str, child: &str) -> String {
    if child.is_empty() {
        return normalize_posix_path(base);
    }
    if is_absolute_posix(child) {
        return normalize_posix_path(child);
    }
    let base = base.trim_end_matches('/');
    if base.is_empty() {
        normalize_posix_path(&format!("/{child}"))
    } else {
        normalize_posix_path(&format!("{base}/{child}"))
    }
}

fn normalize_posix_path(path: &str) -> String {
    let absolute = path.starts_with('/');
    let mut parts = Vec::new();
    for part in path.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                if !parts.is_empty() {
                    parts.pop();
                } else if !absolute {
                    parts.push(part);
                }
            }
            _ => parts.push(part),
        }
    }
    if absolute {
        if parts.is_empty() {
            "/".to_string()
        } else {
            format!("/{}", parts.join("/"))
        }
    } else if parts.is_empty() {
        ".".to_string()
    } else {
        parts.join("/")
    }
}

fn parent_path(path: &str) -> String {
    let path = normalize_posix_path(path);
    if path == "/" {
        return path;
    }
    let Some((parent, _)) = path.rsplit_once('/') else {
        return ".".to_string();
    };
    if parent.is_empty() {
        "/".to_string()
    } else {
        parent.to_string()
    }
}

fn suffix_lower(path: &str) -> String {
    let name = path.rsplit('/').next().unwrap_or(path);
    let Some(index) = name.rfind('.') else {
        return String::new();
    };
    name[index..].to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    const FIXTURES: &str =
        include_str!("../../../../fixtures/web_filesystem_picker_contract.jsonl");

    fn replace_case_placeholder(value: &str, case_dir: &str) -> String {
        value.replace("__case__", case_dir)
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

    fn restore_case_placeholder(value: Value, case_dir: &str) -> Value {
        match value {
            Value::String(value) => Value::String(value.replace(case_dir, "__case__")),
            Value::Array(values) => Value::Array(
                values
                    .into_iter()
                    .map(|value| restore_case_placeholder(value, case_dir))
                    .collect(),
            ),
            Value::Object(values) => Value::Object(
                values
                    .into_iter()
                    .map(|(key, value)| (key, restore_case_placeholder(value, case_dir)))
                    .collect(),
            ),
            value => value,
        }
    }

    fn root_to_value(root: &PickerRoot) -> Value {
        json!({
            "label": root.label,
            "path": root.path,
            "input_path": root.input_path,
        })
    }

    fn entry_to_value(entry: &PickerEntry) -> Value {
        json!({
            "name": entry.name,
            "type": entry.entry_type,
            "path": entry.path,
            "input_path": entry.input_path,
        })
    }

    fn list_response_to_value(response: &PickerListResponse) -> Value {
        let mut value = json!({
            "status_code": response.status_code,
            "success": response.success,
        });
        let object = value.as_object_mut().unwrap();
        if let Some(error) = &response.error {
            object.insert("error".to_string(), json!(error));
        }
        if !response.success {
            return value;
        }
        object.insert(
            "entries".to_string(),
            Value::Array(response.entries.iter().map(entry_to_value).collect()),
        );
        object.insert(
            "roots".to_string(),
            Value::Array(response.roots.iter().map(root_to_value).collect()),
        );
        if let Some(mode) = &response.mode {
            object.insert("mode".to_string(), json!(mode));
        }
        if let Some(path) = &response.path {
            object.insert("path".to_string(), json!(path));
        }
        if let Some(input_path) = &response.input_path {
            object.insert("input_path".to_string(), json!(input_path));
        }
        if let Some(parent) = &response.parent {
            object.insert("parent".to_string(), json!(parent));
        }
        if let Some(parent_input_path) = &response.parent_input_path {
            object.insert("parent_input_path".to_string(), json!(parent_input_path));
        }
        value
    }

    fn assert_subset(actual: &Value, expected: &Value, path: &str) {
        match expected {
            Value::Object(expected) => {
                let actual = actual
                    .as_object()
                    .unwrap_or_else(|| panic!("{path}: actual value is not an object: {actual:?}"));
                for (key, expected_value) in expected {
                    if key == "error_contains" {
                        let actual_error = actual
                            .get("error")
                            .and_then(Value::as_str)
                            .unwrap_or_default();
                        let expected_error = expected_value.as_str().unwrap();
                        assert!(
                            actual_error.contains(expected_error),
                            "{path}.error {actual_error:?} should contain {expected_error:?}"
                        );
                        continue;
                    }
                    assert!(
                        actual.contains_key(key),
                        "{path}: missing key {key} in {actual:?}"
                    );
                    assert_subset(&actual[key], expected_value, &format!("{path}.{key}"));
                }
            }
            _ => assert_eq!(actual, expected, "{path}: value mismatch"),
        }
    }

    fn case_project_root(case_dir: &str) -> String {
        format!("{case_dir}/project")
    }

    fn case_home_dir(case_dir: &str) -> String {
        format!("{case_dir}/home")
    }

    fn children(case: &Value) -> Vec<PickerEntrySpec> {
        case["children"]
            .as_array()
            .unwrap_or(&Vec::new())
            .iter()
            .map(|entry| PickerEntrySpec {
                name: entry["name"].as_str().unwrap().to_string(),
                entry_type: entry["type"].as_str().unwrap().to_string(),
            })
            .collect()
    }

    #[test]
    fn web_filesystem_picker_follows_parity_fixture_table() {
        for line in FIXTURES.lines() {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let case: Value = serde_json::from_str(line).unwrap();
            let case_id = case["case_id"].as_str().unwrap();
            let case_dir = format!("/tmp/{case_id}");
            let project_root = case_project_root(&case_dir);
            let home_dir = case_home_dir(&case_dir);
            let expected = replace_placeholders_in_value(&case["expect"], &case_dir);

            match case["operation"].as_str().unwrap() {
                "resolve" => {
                    let path_text =
                        replace_case_placeholder(case["path_text"].as_str().unwrap(), &case_dir);
                    let resolved = resolve_picker_path(&project_root, &home_dir, &path_text);
                    let actual = restore_case_placeholder(
                        json!({
                            "resolved": resolved,
                            "input_path": input_value_for_path(&project_root, &resolved),
                        }),
                        &case_dir,
                    );
                    assert_subset(&actual, &case["expect"], case_id);
                }
                "extensions" => {
                    let actual = json!({
                        "extensions": parse_extensions(case["raw"].as_str().unwrap()),
                    });
                    assert_subset(&actual, &expected, case_id);
                }
                "roots" => {
                    let actual = restore_case_placeholder(
                        json!({
                            "separator": "/",
                            "roots": filesystem_roots(&project_root, &home_dir)
                                .iter()
                                .map(root_to_value)
                                .collect::<Vec<_>>(),
                        }),
                        &case_dir,
                    );
                    assert_subset(&actual, &case["expect"], case_id);
                }
                "list" => {
                    let path_text =
                        replace_case_placeholder(case["path_text"].as_str().unwrap(), &case_dir);
                    let scandir_error = case.get("scandir_error").and_then(Value::as_str);
                    let input = PickerListInput {
                        project_root: &project_root,
                        home_dir: &home_dir,
                        path_text: &path_text,
                        mode: case["mode"].as_str().unwrap(),
                        extensions: case["extensions"].as_str().unwrap(),
                        path_state: case["path_state"].as_str().unwrap(),
                        scandir_error,
                        children: children(&case),
                    };
                    let actual = restore_case_placeholder(
                        list_response_to_value(&list_filesystem(&input)),
                        &case_dir,
                    );
                    assert_subset(&actual, &case["expect"], case_id);
                }
                operation => panic!("unknown operation {operation}"),
            }
        }
    }
}
