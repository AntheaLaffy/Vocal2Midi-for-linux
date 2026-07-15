//! Download-model catalog metadata and dry-run display helpers.
//!
//! This module mirrors deterministic catalog behavior from `download_models.py`
//! while Python remains the runtime owner for real GitHub API calls, downloads,
//! archive extraction, external CLIs, and model assets.

use std::collections::{BTreeMap, BTreeSet};

#[cfg(test)]
use serde_json::{Value, json};

pub const GITHUB_REPO: &str = "AntheaLaffy/Vocal2Midi-for-linux";
pub const RELEASE_TAG: &str = "v0.1.0";
pub const QWEN_MODEL_ID: &str = "Qwen/Qwen3-ASR-1.7B";
pub const QWEN_LOCAL_DIR: &str = "experiments/Qwen3-ASR-1.7B";

/// One GitHub release model row from the legacy catalog.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogModel {
    pub name: String,
    pub repo: String,
    pub tag: String,
    pub asset: String,
    pub target: String,
    pub marker: String,
    pub label: String,
}

/// Inputs needed to render the legacy `list_planned` dry-run output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedDisplayInput {
    pub models: Vec<CatalogModel>,
    pub asset_sizes: BTreeMap<(String, String), BTreeMap<String, i64>>,
    pub existing_paths: BTreeSet<String>,
    pub qwen_local_dir: String,
    pub qwen_dest_exists: bool,
    pub qwen_entries: Vec<String>,
    pub qwen_source: String,
}

impl CatalogModel {
    pub fn new(
        name: impl Into<String>,
        repo: impl Into<String>,
        tag: impl Into<String>,
        asset: impl Into<String>,
        target: impl Into<String>,
        marker: impl Into<String>,
        label: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            repo: repo.into(),
            tag: tag.into(),
            asset: asset.into(),
            target: target.into(),
            marker: marker.into(),
            label: label.into(),
        }
    }
}

/// Returns the static GitHub model catalog in legacy order.
pub fn github_models() -> Vec<CatalogModel> {
    vec![
        CatalogModel::new(
            "game",
            GITHUB_REPO,
            RELEASE_TAG,
            "GAME-1.0.3-medium-onnx.zip",
            "experiments/GAME-1.0.3-medium-onnx",
            "encoder.onnx",
            format!("{GITHUB_REPO} {RELEASE_TAG}"),
        ),
        CatalogModel::new(
            "hfa",
            GITHUB_REPO,
            RELEASE_TAG,
            "1218_hfa_model_new_dict.zip",
            "experiments/1218_hfa_model_new_dict",
            "model.onnx",
            format!("{GITHUB_REPO} {RELEASE_TAG}"),
        ),
        CatalogModel::new(
            "rmvpe",
            GITHUB_REPO,
            RELEASE_TAG,
            "RMVPE.zip",
            "experiments/RMVPE",
            "rmvpe.onnx",
            format!("{GITHUB_REPO} {RELEASE_TAG}"),
        ),
        CatalogModel::new(
            "romaji",
            GITHUB_REPO,
            RELEASE_TAG,
            "romajiASR.zip",
            "experiments/romajiASR",
            "model.onnx",
            format!("{GITHUB_REPO} {RELEASE_TAG}"),
        ),
    ]
}

/// Returns lookup keys in the same order as the Python dict comprehension.
pub fn github_model_lookup_keys() -> Vec<String> {
    github_models()
        .into_iter()
        .map(|model| model.name)
        .collect()
}

/// Mirrors `download_models.py::human_size`.
pub fn human_size(num_bytes: i64) -> String {
    let mut size = num_bytes as f64;
    for unit in ["B", "KiB", "MiB", "GiB", "TiB"] {
        if size < 1024.0 || unit == "TiB" {
            if unit == "B" {
                return format!("{} {unit}", size as i64);
            }
            return format!("{size:.1} {unit}");
        }
        size /= 1024.0;
    }
    format!("{num_bytes} B")
}

/// Mirrors `download_models.py::asset_url`.
pub fn asset_url(model: &CatalogModel) -> String {
    format!(
        "https://github.com/{}/releases/download/{}/{}",
        model.repo, model.tag, model.asset
    )
}

/// Mirrors `download_models.py::target_has_model` with injected paths.
pub fn target_has_model(model: &CatalogModel, existing_paths: &BTreeSet<String>) -> bool {
    existing_paths.contains(&join_posix(&model.target, &model.marker))
}

/// Mirrors `download_models.py::qwen_has_weights` with injected immediate paths.
pub fn qwen_has_weights(dest_exists: bool, entries: &[String]) -> bool {
    if !dest_exists {
        return false;
    }

    let immediate_names: BTreeSet<String> = entries
        .iter()
        .filter_map(|entry| entry.split('/').next())
        .filter(|name| !name.is_empty())
        .map(str::to_lowercase)
        .collect();

    immediate_names
        .iter()
        .any(|name| name.ends_with(".safetensors") || name.ends_with(".bin"))
}

/// Renders the legacy `list_planned` output as lines with color disabled.
pub fn list_planned_lines(input: &PlannedDisplayInput) -> Vec<String> {
    let mut lines = vec!["ONNX models (GitHub releases):".to_string()];

    for model in &input.models {
        let size_str = input
            .asset_sizes
            .get(&(model.repo.clone(), model.tag.clone()))
            .and_then(|sizes| sizes.get(&model.asset))
            .map(|size| human_size(*size))
            .unwrap_or_else(|| "size unknown".to_string());
        let marker = if target_has_model(model, &input.existing_paths) {
            "✓"
        } else {
            "✗"
        };
        lines.push(format!(
            "  {marker} {:<7} {:<34} {:>12}  -> {}",
            model.name, model.asset, size_str, model.target
        ));
        lines.push(format!("          {}", model.label));
    }

    lines.push("Qwen3-ASR-1.7B:".to_string());
    let marker = if qwen_has_weights(input.qwen_dest_exists, &input.qwen_entries) {
        "✓"
    } else {
        "✗"
    };
    let source = if input.qwen_source == "skip" {
        "skipped"
    } else {
        input.qwen_source.as_str()
    };
    lines.push(format!(
        "  {marker} qwen    {:<34} {:>12}  -> {}  (source: {source})",
        QWEN_MODEL_ID, "large", input.qwen_local_dir
    ));

    lines
}

fn join_posix(base: &str, child: &str) -> String {
    if base.is_empty() {
        return child.to_string();
    }
    if base.ends_with('/') {
        format!("{base}{child}")
    } else {
        format!("{base}/{child}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURES: &str =
        include_str!("../../../../fixtures/download_models_asset_catalog_contract.jsonl");

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
    fn download_models_catalog_fixtures_match() {
        for case in load_cases() {
            let actual = match case["operation"].as_str().unwrap() {
                "catalog_metadata" => run_catalog_metadata(),
                "human_size" => run_human_size(&case),
                "asset_url" => run_asset_url(&case),
                "target_markers" => run_target_markers(&case),
                "qwen_has_weights" => run_qwen_has_weights(&case),
                "list_planned" => run_list_planned(&case),
                other => panic!("unknown operation {other:?}"),
            };
            assert_subset(&actual, &case["expect"]);
        }
    }

    fn run_catalog_metadata() -> Value {
        json!({
            "models": github_models()
                .iter()
                .map(model_value)
                .collect::<Vec<_>>(),
            "lookup_keys": github_model_lookup_keys(),
            "qwen": {
                "model_id": QWEN_MODEL_ID,
                "local_dir": QWEN_LOCAL_DIR,
            },
        })
    }

    fn run_human_size(case: &Value) -> Value {
        json!({
            "results": case["values"]
                .as_array()
                .unwrap()
                .iter()
                .map(|value| {
                    let input = value.as_i64().unwrap();
                    json!({
                        "input": input,
                        "display": human_size(input),
                    })
                })
                .collect::<Vec<_>>(),
        })
    }

    fn run_asset_url(case: &Value) -> Value {
        let model = model_from_fixture(&case["model"]);
        json!({ "url": asset_url(&model) })
    }

    fn run_target_markers(case: &Value) -> Value {
        let existing_paths = path_set_from_entries(&case["existing"]);
        json!({
            "results": case["models"]
                .as_array()
                .unwrap()
                .iter()
                .map(|item| {
                    let model = model_from_fixture(item);
                    json!({
                        "name": model.name,
                        "present": target_has_model(&model, &existing_paths),
                    })
                })
                .collect::<Vec<_>>(),
        })
    }

    fn run_qwen_has_weights(case: &Value) -> Value {
        json!({
            "results": case["cases"]
                .as_array()
                .unwrap()
                .iter()
                .map(|item| {
                    let entries = entry_paths(&item["entries"]);
                    json!({
                        "name": item["name"].as_str().unwrap(),
                        "present": qwen_has_weights(item["dest_exists"].as_bool().unwrap(), &entries),
                    })
                })
                .collect::<Vec<_>>(),
        })
    }

    fn run_list_planned(case: &Value) -> Value {
        let input = PlannedDisplayInput {
            models: case["models"]
                .as_array()
                .unwrap()
                .iter()
                .map(model_from_fixture)
                .collect(),
            asset_sizes: asset_sizes_from_fixture(&case["asset_sizes"]),
            existing_paths: path_set_from_entries(&case["existing"]),
            qwen_local_dir: QWEN_LOCAL_DIR.to_string(),
            qwen_dest_exists: case["qwen_dest_exists"].as_bool().unwrap(),
            qwen_entries: entry_paths(&case["qwen_entries"]),
            qwen_source: case["qwen_source"].as_str().unwrap().to_string(),
        };
        json!({ "lines": list_planned_lines(&input) })
    }

    fn model_value(model: &CatalogModel) -> Value {
        json!({
            "name": model.name,
            "repo": model.repo,
            "tag": model.tag,
            "asset": model.asset,
            "target": model.target,
            "marker": model.marker,
            "label": model.label,
            "asset_url": asset_url(model),
        })
    }

    fn model_from_fixture(value: &Value) -> CatalogModel {
        let name = string_field(value, "name", "fixture");
        CatalogModel::new(
            name.clone(),
            string_field(value, "repo", "fixture/repo"),
            string_field(value, "tag", "v0"),
            string_field(value, "asset", &format!("{name}.zip")),
            string_field(value, "target", &format!("models/{name}")),
            string_field(value, "marker", "model.onnx"),
            string_field(value, "label", "fixture label"),
        )
    }

    fn string_field(value: &Value, key: &str, default: &str) -> String {
        value
            .get(key)
            .and_then(Value::as_str)
            .unwrap_or(default)
            .to_string()
    }

    fn path_set_from_entries(value: &Value) -> BTreeSet<String> {
        entry_paths(value).into_iter().collect()
    }

    fn entry_paths(value: &Value) -> Vec<String> {
        value
            .as_array()
            .unwrap_or_else(|| panic!("expected array fixture, got {value:?}"))
            .iter()
            .map(|entry| entry["path"].as_str().unwrap().to_string())
            .collect()
    }

    fn asset_sizes_from_fixture(
        value: &Value,
    ) -> BTreeMap<(String, String), BTreeMap<String, i64>> {
        value
            .as_array()
            .unwrap()
            .iter()
            .map(|item| {
                let sizes = item["sizes"]
                    .as_object()
                    .unwrap()
                    .iter()
                    .map(|(asset, size)| (asset.clone(), size.as_i64().unwrap()))
                    .collect::<BTreeMap<_, _>>();
                (
                    (
                        item["repo"].as_str().unwrap().to_string(),
                        item["tag"].as_str().unwrap().to_string(),
                    ),
                    sizes,
                )
            })
            .collect()
    }
}
