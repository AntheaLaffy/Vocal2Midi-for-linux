//! Download-model effectful fetch control-flow model.
//!
//! This module mirrors the deterministic state machines around network,
//! subprocess, package-install, and cleanup behavior from `download_models.py`.
//! All live effects remain owned by Python; this Rust module consumes injected
//! fixture outcomes only.

use std::collections::BTreeMap;

use serde_json::Value;

use crate::download_models_catalog::{
    CatalogModel, QWEN_LOCAL_DIR, QWEN_MODEL_ID, asset_url, human_size,
};

#[cfg(test)]
use serde_json::json;

/// Result for a mocked GitHub release-asset API request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetApiResult {
    /// The first.
    pub first: BTreeMap<String, i64>,
    /// The optional second.
    pub second: Option<BTreeMap<String, i64>>,
    /// The urlopen calls.
    pub urlopen_calls: usize,
    /// The request url.
    pub request_url: String,
}

/// Parse GitHub release JSON into asset-name -> byte-size data.
pub fn parse_asset_sizes(payload: &Value) -> BTreeMap<String, i64> {
    payload
        .get("assets")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|asset| {
            let name = asset.get("name").and_then(Value::as_str)?;
            if name.is_empty() {
                return None;
            }
            let size = asset.get("size").and_then(Value::as_i64)?;
            Some((name.to_string(), size))
        })
        .collect()
}

/// Simulate `github_api_asset_sizes` with a one-entry cache.
pub fn github_api_asset_sizes(
    repo: &str,
    tag: &str,
    mode: &str,
    payload: Option<&Value>,
    repeat: bool,
) -> AssetApiResult {
    let mut cache: BTreeMap<(String, String), BTreeMap<String, i64>> = BTreeMap::new();
    let request_url = format!("https://api.github.com/repos/{repo}/releases/tags/{tag}");
    let key = (repo.to_string(), tag.to_string());
    let mut urlopen_calls = 0;

    let mut fetch = |cache: &mut BTreeMap<(String, String), BTreeMap<String, i64>>| {
        if let Some(cached) = cache.get(&key) {
            return cached.clone();
        }
        urlopen_calls += 1;
        let sizes = if mode == "success" {
            payload.map(parse_asset_sizes).unwrap_or_default()
        } else {
            BTreeMap::new()
        };
        cache.insert(key.clone(), sizes.clone());
        sizes
    };

    let first = fetch(&mut cache);
    let second = repeat.then(|| fetch(&mut cache));

    AssetApiResult {
        first,
        second,
        urlopen_calls,
        request_url,
    }
}

/// Result for a mocked stream download.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamDownloadResult {
    /// The downloaded.
    pub downloaded: i64,
    /// The file content.
    pub file_content: String,
    /// The stdout.
    pub stdout: String,
    /// The request url.
    pub request_url: String,
    /// The user agent.
    pub user_agent: String,
}

/// Simulate `stream_download` byte counting and progress formatting.
pub fn stream_download(
    url: &str,
    content_length: Option<&str>,
    chunks: &[String],
) -> StreamDownloadResult {
    let total_int = content_length
        .filter(|value| !value.is_empty() && value.chars().all(|ch| ch.is_ascii_digit()))
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(0);
    let mut downloaded = 0_i64;
    let mut file_content = String::new();
    let mut stdout = String::new();
    let mut last_pct = -1_i64;

    for chunk in chunks {
        file_content.push_str(chunk);
        downloaded += chunk.len() as i64;
        if total_int != 0 {
            let pct = downloaded * 100 / total_int;
            if pct != last_pct {
                last_pct = pct;
                stdout.push_str(&format!(
                    "\r  {pct:3}%  {} / {}",
                    human_size(downloaded),
                    human_size(total_int)
                ));
            }
        } else {
            stdout.push_str(&format!("\r  {}", human_size(downloaded)));
        }
    }
    stdout.push('\n');

    StreamDownloadResult {
        downloaded,
        file_content,
        stdout,
        request_url: url.to_string(),
        user_agent: "vocal2midi-model-fetch/1.0".to_string(),
    }
}

/// Result of a mocked GitHub model download.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GithubDownloadResult {
    /// Whether the modeled operation returned.
    pub returned: bool,
    /// The ordered stream calls.
    pub stream_calls: Vec<String>,
    /// The ordered extract calls.
    pub extract_calls: Vec<BTreeMap<String, String>>,
    /// The ordered tmp zip leftovers.
    pub tmp_zip_leftovers: Vec<String>,
    /// The ordered captured standard-output lines.
    pub stdout_lines: Vec<String>,
    /// The ordered captured standard-error lines.
    pub stderr_lines: Vec<String>,
}

/// Simulate `download_github_model` with injected stream/extract/marker states.
pub fn download_github_model(
    model: &CatalogModel,
    force: bool,
    expected_size: i64,
    stream_result: &Value,
    extract_result: Option<&str>,
    target_present_sequence: &[bool],
) -> GithubDownloadResult {
    let mut target_checks = target_present_sequence.iter();
    let mut stdout_lines = Vec::new();
    let mut stderr_lines = Vec::new();
    let mut stream_calls = Vec::new();
    let mut extract_calls = Vec::new();

    if !force && *target_checks.next().unwrap_or(&false) {
        stdout_lines.push(format!(
            "✓ {} already present, skipping (use --force to re-download)",
            model.target
        ));
        return GithubDownloadResult {
            returned: true,
            stream_calls,
            extract_calls,
            tmp_zip_leftovers: Vec::new(),
            stdout_lines,
            stderr_lines,
        };
    }

    let url = asset_url(model);
    let size_hint = if expected_size != 0 {
        format!("  ({})", human_size(expected_size))
    } else {
        String::new()
    };
    stdout_lines.push(format!(
        "• Downloading {} from {}{}",
        model.asset, model.label, size_hint
    ));
    stdout_lines.push(format!("  {url}"));
    stream_calls.push(url);

    if let Some(code) = stream_result.get("http_error").and_then(Value::as_i64) {
        stderr_lines.push(format!("✗ HTTP {code} downloading {}", model.asset));
        if code == 404 {
            stdout_lines.push(format!(
                "  note: asset not found on {}. Check https://github.com/{}/releases/tag/{}",
                model.label, model.repo, model.tag
            ));
        }
        return GithubDownloadResult {
            returned: false,
            stream_calls,
            extract_calls,
            tmp_zip_leftovers: Vec::new(),
            stdout_lines,
            stderr_lines,
        };
    }
    if stream_result.get("url_error").is_some() || stream_result.get("timeout_error").is_some() {
        return GithubDownloadResult {
            returned: false,
            stream_calls,
            extract_calls,
            tmp_zip_leftovers: Vec::new(),
            stdout_lines,
            stderr_lines,
        };
    }

    let downloaded = stream_result
        .get("bytes")
        .and_then(Value::as_i64)
        .unwrap_or(0);
    if expected_size != 0 && downloaded != expected_size {
        stderr_lines.push(format!(
            "✗ Size mismatch for {}: got {}, expected {} (download may be truncated)",
            model.asset,
            human_size(downloaded),
            human_size(expected_size)
        ));
        return GithubDownloadResult {
            returned: false,
            stream_calls,
            extract_calls,
            tmp_zip_leftovers: Vec::new(),
            stdout_lines,
            stderr_lines,
        };
    }
    if expected_size != 0 {
        stdout_lines.push(format!("✓ Size verified: {}", human_size(downloaded)));
    }

    stdout_lines.push(format!("• Extracting {} -> {}", model.asset, model.target));
    extract_calls.push(BTreeMap::from([(
        "target".to_string(),
        model.target.clone(),
    )]));

    match extract_result.unwrap_or("ok") {
        "bad_zip" => {
            stderr_lines.push(format!(
                "✗ Corrupted zip archive: {} (re-run with --force to retry)",
                model.asset
            ));
            return GithubDownloadResult {
                returned: false,
                stream_calls,
                extract_calls,
                tmp_zip_leftovers: Vec::new(),
                stdout_lines,
                stderr_lines,
            };
        }
        "unsafe_layout" => {
            stderr_lines.push(format!(
                "✗ Unsafe zip archive layout in {}: Unsafe zip member path: '../evil'",
                model.asset
            ));
            return GithubDownloadResult {
                returned: false,
                stream_calls,
                extract_calls,
                tmp_zip_leftovers: Vec::new(),
                stdout_lines,
                stderr_lines,
            };
        }
        _ => {}
    }

    if !*target_checks.next().unwrap_or(&false) {
        stderr_lines.push(format!(
            "✗ Extraction finished but marker '{}' not found in {} — the zip may have an unexpected layout",
            model.marker, model.target
        ));
        return GithubDownloadResult {
            returned: false,
            stream_calls,
            extract_calls,
            tmp_zip_leftovers: Vec::new(),
            stdout_lines,
            stderr_lines,
        };
    }

    stdout_lines.push(format!("✓ {} ready", model.target));
    GithubDownloadResult {
        returned: true,
        stream_calls,
        extract_calls,
        tmp_zip_leftovers: Vec::new(),
        stdout_lines,
        stderr_lines,
    }
}

/// Simulate `_run_cli`.
pub fn run_cli(mode: &str, returncode: Option<i64>) -> i64 {
    if mode == "missing" {
        127
    } else {
        returncode.unwrap_or(0)
    }
}

/// Simulate `_pip_install`.
pub fn pip_install(
    have_uv: bool,
    pkgs: &[String],
    run_return: i64,
) -> (i64, Vec<String>, Vec<String>) {
    if have_uv {
        let mut args = vec![
            "uv".to_string(),
            "pip".to_string(),
            "install".to_string(),
            "--python".to_string(),
            "__python__".to_string(),
        ];
        args.extend(pkgs.iter().cloned());
        (
            run_return,
            args,
            vec![format!("• Installing with uv: {}", pkgs.join(" "))],
        )
    } else {
        let mut args = vec![
            "__python__".to_string(),
            "-m".to_string(),
            "pip".to_string(),
            "install".to_string(),
        ];
        args.extend(pkgs.iter().cloned());
        (
            run_return,
            args,
            vec![format!("• Installing with pip: {}", pkgs.join(" "))],
        )
    }
}

/// Simulate `_resolve_cli`.
pub fn resolve_cli(cli: &str, venv_exists: bool, which: Option<&str>) -> Option<String> {
    if venv_exists {
        Some(format!("__venv__/{cli}"))
    } else {
        which.map(str::to_string)
    }
}

/// Simulate Qwen cleanup for immediate files.
pub fn cleanup_qwen_artifacts(entries: &[String]) -> (Vec<String>, Vec<String>) {
    cleanup_qwen_artifacts_with_unlink_errors(entries, &[])
}

/// Simulate Qwen cleanup, including legacy-swallowed unlink failures.
pub fn cleanup_qwen_artifacts_with_unlink_errors(
    entries: &[String],
    unlink_error_paths: &[String],
) -> (Vec<String>, Vec<String>) {
    let mut remaining = Vec::new();
    let mut stdout_lines = Vec::new();

    for entry in entries {
        if entry == ".gitattributes" {
            if unlink_error_paths.iter().any(|path| path == entry) {
                remaining.push(entry.clone());
                continue;
            }
            stdout_lines
                .push("• Removed modelscope .gitattributes (would force LFS filters)".to_string());
            continue;
        }
        if !entry.contains('/') && entry.ends_with(".incomplete") {
            if unlink_error_paths.iter().any(|path| path == entry) {
                remaining.push(entry.clone());
                continue;
            }
            stdout_lines.push(format!("• Removed partial download {entry}"));
            continue;
        }
        remaining.push(entry.clone());
    }
    remaining.sort();
    (remaining, stdout_lines)
}

/// Result of a mocked Qwen provider CLI download.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QwenCliResult {
    /// Whether the modeled operation returned.
    pub returned: bool,
    /// The ordered resolve calls.
    pub resolve_calls: Vec<String>,
    /// The ordered pip calls.
    pub pip_calls: Vec<Vec<String>>,
    /// The ordered run calls.
    pub run_calls: Vec<Vec<String>>,
    /// Whether cleanup was called.
    pub cleanup_called: bool,
    /// The ordered captured standard-output lines.
    pub stdout_lines: Vec<String>,
    /// The ordered captured standard-error lines.
    pub stderr_lines: Vec<String>,
}

/// Simulate `download_qwen_modelscope` or `download_qwen_huggingface`.
///
/// # Panics
///
/// Panics when `provider` is neither `modelscope` nor `huggingface`.
pub fn qwen_cli_download(
    provider: &str,
    resolve_sequence: &[Option<String>],
    pip_rc: i64,
    venv_bin: Option<&str>,
    run_rc: i64,
    weights_present: bool,
) -> QwenCliResult {
    let mut resolve_sequence = resolve_sequence.iter();
    let mut resolve_calls = Vec::new();
    let mut pip_calls = Vec::new();
    let mut run_calls = Vec::new();
    let mut stdout_lines = Vec::new();
    let mut stderr_lines = Vec::new();

    let spec = provider_spec(provider);
    let mut cli = {
        resolve_calls.push(spec.cli_name.to_string());
        resolve_sequence.next().cloned().flatten()
    };
    if cli.is_none() {
        stdout_lines.push(spec.warn_missing.to_string());
        pip_calls.push(
            spec.install_pkgs
                .iter()
                .map(|value| value.to_string())
                .collect(),
        );
        if pip_rc != 0 {
            stderr_lines.push(spec.install_fail.to_string());
            return QwenCliResult {
                returned: false,
                resolve_calls,
                pip_calls,
                run_calls,
                cleanup_called: false,
                stdout_lines,
                stderr_lines,
            };
        }
        resolve_calls.push(spec.cli_name.to_string());
        cli = resolve_sequence
            .next()
            .cloned()
            .flatten()
            .or_else(|| Some(venv_bin.unwrap_or(spec.default_venv).to_string()));
    }

    let cli = cli.unwrap();
    stdout_lines.push(format!(
        "• Downloading {QWEN_MODEL_ID} from {} -> {QWEN_LOCAL_DIR}",
        spec.display_source
    ));
    let mut run_call = vec![cli];
    run_call.extend(spec.command_args.iter().map(|value| {
        if *value == "__MODEL__" {
            QWEN_MODEL_ID.to_string()
        } else if *value == "__DEST__" {
            "__dest__".to_string()
        } else {
            value.to_string()
        }
    }));
    run_calls.push(run_call);

    if run_rc != 0 {
        stderr_lines.push(spec.run_fail.to_string());
        return QwenCliResult {
            returned: false,
            resolve_calls,
            pip_calls,
            run_calls,
            cleanup_called: false,
            stdout_lines,
            stderr_lines,
        };
    }

    let cleanup_called = true;
    if !weights_present {
        stderr_lines.push(format!(
            "✗ No model weights (.safetensors/.bin) found in {QWEN_LOCAL_DIR} after download"
        ));
        return QwenCliResult {
            returned: false,
            resolve_calls,
            pip_calls,
            run_calls,
            cleanup_called,
            stdout_lines,
            stderr_lines,
        };
    }

    stdout_lines.push(format!("✓ {QWEN_LOCAL_DIR} ready"));
    QwenCliResult {
        returned: true,
        resolve_calls,
        pip_calls,
        run_calls,
        cleanup_called,
        stdout_lines,
        stderr_lines,
    }
}

struct QwenProviderSpec {
    cli_name: &'static str,
    install_pkgs: &'static [&'static str],
    warn_missing: &'static str,
    install_fail: &'static str,
    display_source: &'static str,
    command_args: &'static [&'static str],
    run_fail: &'static str,
    default_venv: &'static str,
}

fn provider_spec(provider: &str) -> QwenProviderSpec {
    match provider {
        "modelscope" => QwenProviderSpec {
            cli_name: "modelscope",
            install_pkgs: &["-U", "modelscope"],
            warn_missing: "! modelscope CLI not found. Installing modelscope...",
            install_fail: "✗ Failed to install modelscope. Run manually: uv pip install -U modelscope  (or: pip install -U modelscope)",
            display_source: "ModelScope",
            command_args: &[
                "download",
                "--model",
                "__MODEL__",
                "--local_dir",
                "__DEST__",
            ],
            run_fail: "✗ modelscope download failed",
            default_venv: "/venv/modelscope",
        },
        "huggingface" => QwenProviderSpec {
            cli_name: "huggingface-cli",
            install_pkgs: &["-U", "huggingface_hub[cli]"],
            warn_missing: "! huggingface-cli not found. Installing huggingface_hub[cli]...",
            install_fail: "✗ Failed to install huggingface_hub. Run manually: uv pip install -U \"huggingface_hub[cli]\"  (or: pip install -U \"huggingface_hub[cli]\")",
            display_source: "Hugging Face",
            command_args: &["download", "__MODEL__", "--local-dir", "__DEST__"],
            run_fail: "✗ huggingface-cli download failed",
            default_venv: "/venv/huggingface-cli",
        },
        other => panic!("unknown provider {other:?}"),
    }
}

/// Result of `download_qwen` strategy selection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QwenStrategyResult {
    /// Whether the modeled operation returned.
    pub returned: bool,
    /// The ordered modelscope calls.
    pub modelscope_calls: Vec<String>,
    /// The ordered huggingface calls.
    pub huggingface_calls: Vec<String>,
    /// The ordered captured standard-output lines.
    pub stdout_lines: Vec<String>,
    /// The ordered captured standard-error lines.
    pub stderr_lines: Vec<String>,
}

/// Simulate `download_qwen` source dispatch and fallback behavior.
pub fn download_qwen_strategy(
    source: &str,
    force: bool,
    already_has_weights: bool,
    modelscope_result: bool,
    huggingface_result: bool,
) -> QwenStrategyResult {
    let mut modelscope_calls = Vec::new();
    let mut huggingface_calls = Vec::new();
    let mut stdout_lines = Vec::new();
    let mut stderr_lines = Vec::new();

    if !force && already_has_weights {
        stdout_lines.push(format!(
            "✓ {QWEN_LOCAL_DIR} already has weights, skipping (use --force to re-download)"
        ));
        return QwenStrategyResult {
            returned: true,
            modelscope_calls,
            huggingface_calls,
            stdout_lines,
            stderr_lines,
        };
    }

    match source {
        "modelscope" => {
            modelscope_calls.push(QWEN_LOCAL_DIR.to_string());
            QwenStrategyResult {
                returned: modelscope_result,
                modelscope_calls,
                huggingface_calls,
                stdout_lines,
                stderr_lines,
            }
        }
        "huggingface" => {
            huggingface_calls.push(QWEN_LOCAL_DIR.to_string());
            QwenStrategyResult {
                returned: huggingface_result,
                modelscope_calls,
                huggingface_calls,
                stdout_lines,
                stderr_lines,
            }
        }
        "auto" => {
            stdout_lines
                .push("• Trying ModelScope first (preferred for Mainland China)...".to_string());
            modelscope_calls.push(QWEN_LOCAL_DIR.to_string());
            if modelscope_result {
                QwenStrategyResult {
                    returned: true,
                    modelscope_calls,
                    huggingface_calls,
                    stdout_lines,
                    stderr_lines,
                }
            } else {
                stdout_lines
                    .push("! ModelScope failed, falling back to Hugging Face...".to_string());
                huggingface_calls.push(QWEN_LOCAL_DIR.to_string());
                QwenStrategyResult {
                    returned: huggingface_result,
                    modelscope_calls,
                    huggingface_calls,
                    stdout_lines,
                    stderr_lines,
                }
            }
        }
        _ => {
            stderr_lines.push(format!("✗ Unknown qwen source: {source}"));
            QwenStrategyResult {
                returned: false,
                modelscope_calls,
                huggingface_calls,
                stdout_lines,
                stderr_lines,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURES: &str =
        include_str!("../../../../fixtures/download_models_effectful_fetch_contract.jsonl");

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
    fn download_models_effectful_fetch_fixtures_match() {
        for case in load_cases() {
            let actual = match case["operation"].as_str().unwrap() {
                "github_api_asset_sizes" => run_github_api_asset_sizes(&case),
                "stream_download" => run_stream_download(&case),
                "download_github_model" => run_download_github_model(&case),
                "run_cli" => run_run_cli(&case),
                "pip_install" => run_pip_install(&case),
                "resolve_cli" => run_resolve_cli(&case),
                "cleanup_qwen_artifacts" => run_cleanup_qwen_artifacts(&case),
                "qwen_cli_download" => run_qwen_cli_download(&case),
                "download_qwen_strategy" => run_download_qwen_strategy(&case),
                other => panic!("unknown operation {other:?}"),
            };
            assert_subset(&actual, &case["expect"]);
        }
    }

    fn run_github_api_asset_sizes(case: &Value) -> Value {
        let result = github_api_asset_sizes(
            case["repo"].as_str().unwrap(),
            case["tag"].as_str().unwrap(),
            case["mode"].as_str().unwrap(),
            case.get("payload"),
            case.get("repeat").and_then(Value::as_bool).unwrap_or(false),
        );
        let mut actual = json!({
            "first": result.first,
            "urlopen_calls": result.urlopen_calls,
            "request_url": result.request_url,
        });
        if let Some(second) = result.second {
            actual["second"] = json!(second);
        }
        actual
    }

    fn run_stream_download(case: &Value) -> Value {
        let chunks = string_vec(&case["chunks"]);
        let result = stream_download(
            case["url"].as_str().unwrap(),
            case.get("content_length").and_then(Value::as_str),
            &chunks,
        );
        json!({
            "downloaded": result.downloaded,
            "file_content": result.file_content,
            "stdout": result.stdout,
            "request_url": result.request_url,
            "user_agent": result.user_agent,
        })
    }

    fn run_download_github_model(case: &Value) -> Value {
        let model = model_from_fixture(&case["model"]);
        let target_present_sequence = bool_vec(case.get("target_present_sequence"));
        let result = download_github_model(
            &model,
            case["force"].as_bool().unwrap(),
            case.get("expected_size")
                .and_then(Value::as_i64)
                .unwrap_or(0),
            case.get("stream_result").unwrap_or(&Value::Null),
            case.get("extract_result").and_then(Value::as_str),
            &target_present_sequence,
        );
        let stream_result = case.get("stream_result").unwrap_or(&Value::Null);
        if let Some(error) = stream_result.get("url_error").and_then(Value::as_str) {
            json!({
                "status": "error",
                "error_type": "URLError",
                "error": format!("<urlopen error {error}>"),
                "stream_calls": result.stream_calls,
                "extract_calls": result.extract_calls,
                "tmp_zip_leftovers": result.tmp_zip_leftovers,
                "stdout_lines": result.stdout_lines,
                "stderr_lines": result.stderr_lines,
            })
        } else if let Some(error) = stream_result.get("timeout_error").and_then(Value::as_str) {
            json!({
                "status": "error",
                "error_type": "TimeoutError",
                "error": error,
                "stream_calls": result.stream_calls,
                "extract_calls": result.extract_calls,
                "tmp_zip_leftovers": result.tmp_zip_leftovers,
                "stdout_lines": result.stdout_lines,
                "stderr_lines": result.stderr_lines,
            })
        } else {
            json!({
                "return": result.returned,
                "stream_calls": result.stream_calls,
                "extract_calls": result.extract_calls,
                "tmp_zip_leftovers": result.tmp_zip_leftovers,
                "stdout_lines": result.stdout_lines,
                "stderr_lines": result.stderr_lines,
            })
        }
    }

    fn run_run_cli(case: &Value) -> Value {
        json!({
            "results": case["cases"]
                .as_array()
                .unwrap()
                .iter()
                .map(|item| json!({
                    "name": item["name"],
                    "return": run_cli(
                        item["mode"].as_str().unwrap(),
                        item.get("returncode").and_then(Value::as_i64),
                    ),
                }))
                .collect::<Vec<_>>(),
        })
    }

    fn run_pip_install(case: &Value) -> Value {
        json!({
            "results": case["cases"]
                .as_array()
                .unwrap()
                .iter()
                .map(|item| {
                    let pkgs = string_vec(&item["pkgs"]);
                    let (returned, run_args, stdout_lines) = pip_install(
                        item["have_uv"].as_bool().unwrap(),
                        &pkgs,
                        item["run_return"].as_i64().unwrap(),
                    );
                    json!({
                        "name": item["name"],
                        "return": returned,
                        "run_args": run_args,
                        "stdout_lines": stdout_lines,
                    })
                })
                .collect::<Vec<_>>(),
        })
    }

    fn run_resolve_cli(case: &Value) -> Value {
        json!({
            "results": case["cases"]
                .as_array()
                .unwrap()
                .iter()
                .map(|item| json!({
                    "name": item["name"],
                    "resolved": resolve_cli(
                        item["cli"].as_str().unwrap(),
                        item["venv_exists"].as_bool().unwrap(),
                        item.get("which").and_then(Value::as_str),
                    ),
                }))
                .collect::<Vec<_>>(),
        })
    }

    fn run_cleanup_qwen_artifacts(case: &Value) -> Value {
        let mut unlink_error_paths = Vec::new();
        let entries = case["entries"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|entry| {
                if entry["kind"].as_str() == Some("file") {
                    let path = entry["path"].as_str().unwrap().to_string();
                    if entry
                        .get("unlink_error")
                        .and_then(Value::as_bool)
                        .unwrap_or(false)
                    {
                        unlink_error_paths.push(path.clone());
                    }
                    Some(path)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        let (remaining, stdout_lines) =
            cleanup_qwen_artifacts_with_unlink_errors(&entries, &unlink_error_paths);
        json!({
            "remaining": remaining,
            "stdout_lines": stdout_lines,
        })
    }

    fn run_qwen_cli_download(case: &Value) -> Value {
        let resolve_sequence = case
            .get("resolve_sequence")
            .and_then(Value::as_array)
            .unwrap()
            .iter()
            .map(|value| value.as_str().map(str::to_string))
            .collect::<Vec<_>>();
        let result = qwen_cli_download(
            case["provider"].as_str().unwrap(),
            &resolve_sequence,
            case.get("pip_rc").and_then(Value::as_i64).unwrap_or(0),
            case.get("venv_bin").and_then(Value::as_str),
            case.get("run_rc").and_then(Value::as_i64).unwrap_or(0),
            case.get("weights_present")
                .and_then(Value::as_bool)
                .unwrap_or(false),
        );
        json!({
            "return": result.returned,
            "resolve_calls": result.resolve_calls,
            "pip_calls": result.pip_calls,
            "run_calls": result.run_calls,
            "cleanup_called": result.cleanup_called,
            "stdout_lines": result.stdout_lines,
            "stderr_lines": result.stderr_lines,
        })
    }

    fn run_download_qwen_strategy(case: &Value) -> Value {
        let result = download_qwen_strategy(
            case["source"].as_str().unwrap(),
            case["force"].as_bool().unwrap(),
            case["already_has_weights"].as_bool().unwrap(),
            case.get("modelscope_result")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            case.get("huggingface_result")
                .and_then(Value::as_bool)
                .unwrap_or(false),
        );
        json!({
            "return": result.returned,
            "modelscope_calls": result.modelscope_calls,
            "huggingface_calls": result.huggingface_calls,
            "stdout_lines": result.stdout_lines,
            "stderr_lines": result.stderr_lines,
        })
    }

    fn model_from_fixture(value: &Value) -> CatalogModel {
        CatalogModel::new(
            value["name"].as_str().unwrap(),
            value["repo"].as_str().unwrap(),
            value["tag"].as_str().unwrap(),
            value["asset"].as_str().unwrap(),
            value["target"].as_str().unwrap(),
            value["marker"].as_str().unwrap(),
            value["label"].as_str().unwrap(),
        )
    }

    fn string_vec(value: &Value) -> Vec<String> {
        value
            .as_array()
            .unwrap()
            .iter()
            .map(|item| item.as_str().unwrap().to_string())
            .collect()
    }

    fn bool_vec(value: Option<&Value>) -> Vec<bool> {
        value
            .and_then(Value::as_array)
            .map(|items| items.iter().map(|item| item.as_bool().unwrap()).collect())
            .unwrap_or_default()
    }
}
