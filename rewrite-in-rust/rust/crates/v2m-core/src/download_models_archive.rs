//! Download-model archive member validation and merge layout.
//!
//! This module mirrors the deterministic archive path-safety and target-layout
//! behavior from `download_models.py` while Python remains the runtime owner for
//! real zip extraction, network downloads, package installation, and model
//! assets.

use std::collections::{BTreeMap, BTreeSet};

#[cfg(test)]
use serde_json::{Value, json};

/// One archive file member used by archive-layout fixtures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchiveMember {
    pub name: String,
    pub content: String,
}

/// One target file used by archive-layout fixtures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetFile {
    pub path: String,
    pub content: String,
}

/// Error message returned for unsafe archive members.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnsafeArchiveMember {
    pub message: String,
}

/// Validates one zip member name using the legacy path rules.
pub fn validate_member_parts(name: &str) -> Result<Vec<String>, UnsafeArchiveMember> {
    if name.is_empty() || name.contains('\0') || name.contains('\\') {
        return Err(unsafe_member(name));
    }
    if name.starts_with('/') || windows_has_drive_or_absolute(name) {
        return Err(unsafe_member(name));
    }

    let parts: Vec<String> = name.split('/').map(str::to_string).collect();
    if parts.iter().any(|part| part == "..") {
        return Err(unsafe_member(name));
    }
    Ok(parts)
}

/// Models `extract_zip` final target files from explicit archive members.
pub fn archive_target_layout(
    members: &[ArchiveMember],
    preexisting: &[TargetFile],
) -> Result<Vec<TargetFile>, UnsafeArchiveMember> {
    let names: Vec<&str> = members.iter().map(|member| member.name.as_str()).collect();
    let top_levels: BTreeSet<String> = names
        .iter()
        .filter(|name| !name.is_empty())
        .map(|name| {
            name.split_once('/')
                .map_or(*name, |(top, _)| top)
                .to_string()
        })
        .collect();
    let single_top = top_levels.len() == 1
        && top_levels.iter().next().is_some_and(|top| {
            !top.ends_with(".onnx") && !top.ends_with(".zip") && !top.is_empty()
        });
    let single_top_root = top_levels.iter().next().cloned();
    let root_is_dir = single_top_root.as_deref().is_some_and(|root| {
        names
            .iter()
            .any(|name| *name == format!("{root}/") || name.starts_with(&format!("{root}/")))
    });
    let strip_root = single_top && root_is_dir;

    let mut files: BTreeMap<String, String> = preexisting
        .iter()
        .map(|file| (file.path.clone(), file.content.clone()))
        .collect();

    for member in members {
        let parts = validate_member_parts(&member.name)?;
        if member.name.ends_with('/') {
            continue;
        }
        let output_parts = if strip_root {
            parts.into_iter().skip(1).collect::<Vec<_>>()
        } else {
            parts
        };
        if output_parts.is_empty() {
            continue;
        }
        files.insert(output_parts.join("/"), member.content.clone());
    }

    Ok(files
        .into_iter()
        .map(|(path, content)| TargetFile { path, content })
        .collect())
}

fn windows_has_drive_or_absolute(name: &str) -> bool {
    let bytes = name.as_bytes();
    if bytes.len() >= 2 && bytes[1] == b':' && bytes[0].is_ascii_alphabetic() {
        return true;
    }
    name.starts_with("//")
}

fn unsafe_member(name: &str) -> UnsafeArchiveMember {
    UnsafeArchiveMember {
        message: format!("Unsafe zip member path: {}", python_string_repr(name)),
    }
}

fn python_string_repr(value: &str) -> String {
    let quote = if value.contains('\'') && !value.contains('"') {
        '"'
    } else {
        '\''
    };
    let mut repr = String::new();
    repr.push(quote);
    for ch in value.chars() {
        match ch {
            '\\' => repr.push_str(r"\\"),
            '\'' if quote == '\'' => repr.push_str(r"\'"),
            '"' if quote == '"' => repr.push_str("\\\""),
            '\n' => repr.push_str(r"\n"),
            '\r' => repr.push_str(r"\r"),
            '\t' => repr.push_str(r"\t"),
            ch if (ch as u32) < 0x100 && (ch.is_control() || ch == '\u{7f}') => {
                repr.push_str(&format!(r"\x{:02x}", ch as u32));
            }
            ch if ch.is_control() => {
                repr.push_str(&format!(r"\u{:04x}", ch as u32));
            }
            ch => repr.push(ch),
        }
    }
    repr.push(quote);
    repr
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURES: &str =
        include_str!("../../../../fixtures/download_models_archive_layout_contract.jsonl");

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
    fn download_models_archive_layout_fixtures_match() {
        for case in load_cases() {
            let actual = match case["operation"].as_str().unwrap() {
                "extract_zip" => run_extract_zip(&case),
                "validate_member" => run_validate_member(&case),
                other => panic!("unknown operation {other:?}"),
            };
            assert_subset(&actual, &case["expect"]);
        }
    }

    fn run_extract_zip(case: &Value) -> Value {
        let members = files_from_fixture(&case["members"]);
        let preexisting = case
            .get("preexisting")
            .map(target_files_from_fixture)
            .unwrap_or_default();
        match archive_target_layout(&members, &preexisting) {
            Ok(files) => json!({
                "status": "ok",
                "files": files_value(&files),
            }),
            Err(error) => json!({
                "status": "error",
                "error": error.message,
            }),
        }
    }

    fn run_validate_member(case: &Value) -> Value {
        match validate_member_parts(case["member_name"].as_str().unwrap()) {
            Ok(parts) => json!({
                "status": "ok",
                "parts": parts,
            }),
            Err(error) => json!({
                "status": "error",
                "error": error.message,
            }),
        }
    }

    fn files_from_fixture(value: &Value) -> Vec<ArchiveMember> {
        value
            .as_array()
            .unwrap()
            .iter()
            .map(|item| ArchiveMember {
                name: item["name"]
                    .as_str()
                    .or_else(|| item["path"].as_str())
                    .unwrap()
                    .to_string(),
                content: item["content"].as_str().unwrap().to_string(),
            })
            .collect()
    }

    fn target_files_from_fixture(value: &Value) -> Vec<TargetFile> {
        value
            .as_array()
            .unwrap()
            .iter()
            .map(|item| TargetFile {
                path: item["path"].as_str().unwrap().to_string(),
                content: item["content"].as_str().unwrap().to_string(),
            })
            .collect()
    }

    fn files_value(files: &[TargetFile]) -> Value {
        Value::Array(
            files
                .iter()
                .map(|file| {
                    json!({
                        "path": file.path,
                        "content": file.content,
                    })
                })
                .collect(),
        )
    }
}
