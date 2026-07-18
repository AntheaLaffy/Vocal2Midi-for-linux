//! Download-model command-line selection planning.
//!
//! This module mirrors deterministic `parse_args` and `main` selection behavior
//! from `download_models.py` while Python remains the runtime owner for real
//! downloads, archive extraction, external CLIs, and model assets.

use std::collections::BTreeMap;

use crate::download_models_catalog::github_models;

#[cfg(test)]
use serde_json::{Value, json};

const QWEN_SOURCE_CHOICES: &[&str] = &["auto", "modelscope", "huggingface", "skip"];

/// Parsed CLI arguments for `download_models.py`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DownloadModelsArgs {
    /// The optional only.
    pub only: Option<Vec<String>>,
    /// Whether forced replacement is enabled.
    pub force: bool,
    /// The selected Qwen model source.
    pub qwen_source: String,
    /// Whether the Qwen download is disabled.
    pub no_qwen: bool,
    /// Whether dry-run listing is selected.
    pub list: bool,
}

/// Parser failure modeled after argparse's exit status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliParseError {
    /// The exit code.
    pub exit_code: i32,
    /// The message text.
    pub message: String,
}

/// One fake GitHub download call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GithubDownloadCall {
    /// The name.
    pub name: String,
    /// Whether forced replacement is enabled.
    pub force: bool,
}

/// One fake Qwen download call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QwenDownloadCall {
    /// The source.
    pub source: String,
    /// Whether forced replacement is enabled.
    pub force: bool,
}

/// Simulated output of `main` with patched effectful collaborators.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MainPlanOutcome {
    /// The exit code.
    pub exit_code: i32,
    /// Whether output-directory creation was called.
    pub mkdir_called: bool,
    /// The ordered github calls.
    pub github_calls: Vec<GithubDownloadCall>,
    /// The ordered qwen calls.
    pub qwen_calls: Vec<QwenDownloadCall>,
    /// The ordered list calls.
    pub list_calls: Vec<String>,
    /// The ordered captured standard-output lines.
    pub stdout_lines: Vec<String>,
    /// The ordered captured standard-error lines.
    pub stderr_lines: Vec<String>,
}

/// Mirrors the accepted flags and defaults of `download_models.py::parse_args`.
///
/// # Errors
///
/// Returns [`CliParseError`] for a missing option value, invalid choice, or
/// unrecognized argument.
pub fn parse_cli_args(argv: &[String]) -> Result<DownloadModelsArgs, CliParseError> {
    let only_choices = only_choices();
    let mut args = DownloadModelsArgs {
        only: None,
        force: false,
        qwen_source: "auto".to_string(),
        no_qwen: false,
        list: false,
    };

    let mut index = 0;
    while index < argv.len() {
        match argv[index].as_str() {
            "--only" => {
                let value = option_value(argv, index, "--only")?;
                if !only_choices.iter().any(|choice| choice == value) {
                    return Err(invalid_choice("--only", value, &only_choices));
                }
                args.only.get_or_insert_with(Vec::new).push(value.clone());
                index += 2;
            }
            "--force" => {
                args.force = true;
                index += 1;
            }
            "--qwen-source" => {
                let value = option_value(argv, index, "--qwen-source")?;
                if !QWEN_SOURCE_CHOICES.contains(&value.as_str()) {
                    return Err(invalid_choice("--qwen-source", value, QWEN_SOURCE_CHOICES));
                }
                args.qwen_source = value.clone();
                index += 2;
            }
            "--no-qwen" => {
                args.no_qwen = true;
                index += 1;
            }
            "--list" => {
                args.list = true;
                index += 1;
            }
            other => {
                return Err(CliParseError {
                    exit_code: 2,
                    message: format!("unrecognized arguments: {other}"),
                });
            }
        }
    }

    Ok(args)
}

/// Simulates `download_models.py::main` with fake download/list collaborators.
///
/// # Errors
///
/// Returns [`CliParseError`] when [`parse_cli_args`] rejects the argument list.
pub fn plan_main(
    argv: &[String],
    github_outcomes: &BTreeMap<String, bool>,
    qwen_outcome: bool,
) -> Result<MainPlanOutcome, CliParseError> {
    let args = parse_cli_args(argv)?;

    if args.list {
        let qwen_source = if args.no_qwen {
            "skip".to_string()
        } else {
            args.qwen_source.clone()
        };
        return Ok(MainPlanOutcome {
            exit_code: 0,
            mkdir_called: false,
            github_calls: Vec::new(),
            qwen_calls: Vec::new(),
            list_calls: vec![qwen_source.clone()],
            stdout_lines: vec![format!("LIST {qwen_source}")],
            stderr_lines: Vec::new(),
        });
    }

    let selected = args.only.as_ref();
    let do_qwen = !args.no_qwen
        && selected.is_none_or(|selected_models| selected_models.iter().any(|name| name == "qwen"));

    let mut github_calls = Vec::new();
    let mut qwen_calls = Vec::new();
    let mut failures = Vec::new();

    for model in github_models() {
        if selected
            .is_some_and(|selected_models| !selected_models.iter().any(|name| name == &model.name))
        {
            continue;
        }
        github_calls.push(GithubDownloadCall {
            name: model.name.clone(),
            force: args.force,
        });
        if !github_outcomes.get(&model.name).copied().unwrap_or(true) {
            failures.push(model.name);
        }
    }

    if do_qwen {
        qwen_calls.push(QwenDownloadCall {
            source: args.qwen_source.clone(),
            force: args.force,
        });
        if !qwen_outcome {
            failures.push("qwen".to_string());
        }
    }

    if failures.is_empty() {
        return Ok(MainPlanOutcome {
            exit_code: 0,
            mkdir_called: true,
            github_calls,
            qwen_calls,
            list_calls: Vec::new(),
            stdout_lines: vec![
                String::new(),
                "✓ All requested models are ready under experiments/".to_string(),
            ],
            stderr_lines: Vec::new(),
        });
    }

    Ok(MainPlanOutcome {
        exit_code: 1,
        mkdir_called: true,
        github_calls,
        qwen_calls,
        list_calls: Vec::new(),
        stdout_lines: vec![
            String::new(),
            String::new(),
            "Tips:".to_string(),
            "  - re-run with --force to retry".to_string(),
            "  - for Qwen, try: --qwen-source huggingface".to_string(),
        ],
        stderr_lines: vec![format!("✗ Failed to fetch: {}", failures.join(", "))],
    })
}

fn only_choices() -> Vec<String> {
    let mut choices = github_models()
        .into_iter()
        .map(|model| model.name)
        .collect::<Vec<_>>();
    choices.push("qwen".to_string());
    choices
}

fn option_value<'a>(
    argv: &'a [String],
    index: usize,
    option: &str,
) -> Result<&'a String, CliParseError> {
    argv.get(index + 1).ok_or_else(|| CliParseError {
        exit_code: 2,
        message: format!("argument {option}: expected one argument"),
    })
}

fn invalid_choice(option: &str, value: &str, choices: &[impl AsRef<str>]) -> CliParseError {
    CliParseError {
        exit_code: 2,
        message: format!(
            "argument {option}: invalid choice: '{value}' (choose from {})",
            choices
                .iter()
                .map(AsRef::as_ref)
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURES: &str =
        include_str!("../../../../fixtures/download_models_cli_selection_contract.jsonl");

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
    fn download_models_cli_selection_fixtures_match() {
        for case in load_cases() {
            let actual = match case["operation"].as_str().unwrap() {
                "parse_args" => run_parse_args(&case),
                "main_plan" => run_main_plan(&case),
                other => panic!("unknown operation {other:?}"),
            };
            assert_subset(&actual, &case["expect"]);
        }
    }

    fn run_parse_args(case: &Value) -> Value {
        let argv = string_vec(&case["argv"]);
        match parse_cli_args(&argv) {
            Ok(args) => json!({
                "status": "ok",
                "only": args.only,
                "force": args.force,
                "qwen_source": args.qwen_source,
                "no_qwen": args.no_qwen,
                "list": args.list,
            }),
            Err(error) => {
                let fragments = string_vec(&case["stderr_must_contain"]);
                json!({
                    "status": "error",
                    "exit_code": error.exit_code,
                    "stderr_contains": fragments
                        .into_iter()
                        .filter(|fragment| error.message.contains(fragment))
                        .collect::<Vec<_>>(),
                })
            }
        }
    }

    fn run_main_plan(case: &Value) -> Value {
        let argv = string_vec(&case["argv"]);
        let github_outcomes = github_outcomes_from_fixture(case);
        let qwen_outcome = case
            .get("qwen_outcome")
            .and_then(Value::as_bool)
            .unwrap_or(true);
        match plan_main(&argv, &github_outcomes, qwen_outcome) {
            Ok(outcome) => json!({
                "exit_code": outcome.exit_code,
                "mkdir_called": outcome.mkdir_called,
                "github_calls": outcome.github_calls
                    .iter()
                    .map(|call| json!({
                        "name": call.name,
                        "force": call.force,
                    }))
                    .collect::<Vec<_>>(),
                "qwen_calls": outcome.qwen_calls
                    .iter()
                    .map(|call| json!({
                        "source": call.source,
                        "force": call.force,
                    }))
                    .collect::<Vec<_>>(),
                "list_calls": outcome.list_calls,
                "stdout_lines": outcome.stdout_lines,
                "stderr_lines": outcome.stderr_lines,
            }),
            Err(error) => json!({
                "status": "error",
                "exit_code": error.exit_code,
            }),
        }
    }

    fn string_vec(value: &Value) -> Vec<String> {
        value
            .as_array()
            .unwrap()
            .iter()
            .map(|item| item.as_str().unwrap().to_string())
            .collect()
    }

    fn github_outcomes_from_fixture(case: &Value) -> BTreeMap<String, bool> {
        case.get("github_outcomes")
            .and_then(Value::as_object)
            .map(|object| {
                object
                    .iter()
                    .map(|(name, value)| (name.clone(), value.as_bool().unwrap()))
                    .collect()
            })
            .unwrap_or_default()
    }
}
