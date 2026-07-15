//! Application job validation and error mapping.
//!
//! This module mirrors `application/pipeline.py::run_auto_lyric_job` at the
//! guard and error-contract boundary while legacy Python remains the runtime
//! owner. It does not call or replace the hybrid inference pipeline.

use std::path::Path;

const MODEL_NOT_FOUND_MESSAGE: &str = "模型路径验证失败";
const CANCELLED_BEFORE_START_MESSAGE: &str = "Pipeline was cancelled before starting.";
const PIPELINE_INTERRUPTED_MESSAGE: &str = "Pipeline was interrupted by user.";

/// Minimal application job inputs needed by the guard contract.
#[derive(Debug, Clone, Copy)]
pub struct ApplicationJobConfig<'a> {
    pub game_model_dir: &'a Path,
    pub hfa_model_dir: &'a Path,
    pub asr_model_path: &'a Path,
    pub output_lyrics: bool,
    pub cancel_before_start: bool,
}

/// Legacy hybrid pipeline result used to model Python call/error mapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LegacyPipelineResult<'a> {
    Completed,
    Interrupted,
    Vocal2MidiError { message: &'a str, details: &'a str },
    OtherError { display: &'a str },
}

/// Application-layer failure compatible with the Python exception contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApplicationJobError {
    ModelNotFound { details: String },
    Cancelled { message: &'static str },
    Vocal2Midi { message: String, details: String },
}

impl ApplicationJobError {
    /// Stable error kind used by fixture tests.
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::ModelNotFound { .. } => "model_not_found",
            Self::Cancelled { .. } => "cancelled",
            Self::Vocal2Midi { .. } => "vocal2midi_error",
        }
    }

    /// Python-compatible user-facing message.
    pub fn message(&self) -> &str {
        match self {
            Self::ModelNotFound { .. } => MODEL_NOT_FOUND_MESSAGE,
            Self::Cancelled { message } => message,
            Self::Vocal2Midi { message, .. } => message.as_str(),
        }
    }

    /// Python-compatible details string.
    pub fn details(&self) -> &str {
        match self {
            Self::ModelNotFound { details } => details.as_str(),
            Self::Cancelled { .. } => "",
            Self::Vocal2Midi { details, .. } => details.as_str(),
        }
    }
}

impl std::fmt::Display for ApplicationJobError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.message())
    }
}

impl std::error::Error for ApplicationJobError {}

/// Validates the model paths required before the hybrid pipeline starts.
pub fn validate_model_paths(config: &ApplicationJobConfig<'_>) -> Result<(), ApplicationJobError> {
    let mut errors = Vec::new();
    collect_missing_path(&mut errors, "GAME 模型目录", config.game_model_dir);
    if config.output_lyrics {
        collect_missing_path(&mut errors, "HubertFA 模型目录", config.hfa_model_dir);
        collect_missing_path(&mut errors, "ASR 模型路径", config.asr_model_path);
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(ApplicationJobError::ModelNotFound {
            details: errors.join("; "),
        })
    }
}

fn collect_missing_path(errors: &mut Vec<String>, label: &str, path: &Path) {
    if path.as_os_str().is_empty() || !path.exists() {
        errors.push(format!("{label}不存在或无效: {}", path.display()));
    }
}

/// Runs the application job contract around a legacy pipeline closure.
pub fn run_auto_lyric_job_contract<'a, F>(
    config: &ApplicationJobConfig<'_>,
    run_pipeline: F,
) -> Result<(), ApplicationJobError>
where
    F: FnOnce() -> LegacyPipelineResult<'a>,
{
    validate_model_paths(config)?;

    if config.cancel_before_start {
        return Err(ApplicationJobError::Cancelled {
            message: CANCELLED_BEFORE_START_MESSAGE,
        });
    }

    match run_pipeline() {
        LegacyPipelineResult::Completed => Ok(()),
        LegacyPipelineResult::Interrupted => Err(ApplicationJobError::Cancelled {
            message: PIPELINE_INTERRUPTED_MESSAGE,
        }),
        LegacyPipelineResult::Vocal2MidiError { message, details } => {
            Err(ApplicationJobError::Vocal2Midi {
                message: message.to_string(),
                details: details.to_string(),
            })
        }
        LegacyPipelineResult::OtherError { display } => Err(ApplicationJobError::Vocal2Midi {
            message: format!("Pipeline execution failed: {display}"),
            details: display.to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    const FIXTURES: &str = include_str!("../../../../fixtures/application_job_contract.tsv");

    fn parse_bool(value: &str) -> bool {
        match value {
            "true" => true,
            "false" => false,
            _ => panic!("unknown bool value {value}"),
        }
    }

    fn parse_cancel_before_start(value: &str) -> bool {
        match value {
            "true" => true,
            "false" | "none" => false,
            _ => panic!("unknown cancel checker marker {value}"),
        }
    }

    fn decode_empty(value: &str) -> &str {
        if value == "__empty__" { "" } else { value }
    }

    fn labels_from_fixture(value: &str) -> Vec<&str> {
        if value == "__empty__" {
            Vec::new()
        } else {
            value.split('|').collect()
        }
    }

    fn resolve_fixture_path(base_dir: &Path, marker: &str, name: &str) -> PathBuf {
        match marker {
            "empty" => PathBuf::new(),
            "missing" => base_dir.join(format!("missing-{name}")),
            "exists_dir" => {
                let path = base_dir.join(format!("{name}-dir"));
                fs::create_dir_all(&path).unwrap();
                path
            }
            "exists_file" => {
                let path = base_dir.join(format!("{name}.bin"));
                fs::write(&path, "fixture").unwrap();
                path
            }
            _ => panic!("unknown path marker {marker}"),
        }
    }

    fn parse_pipeline_result(value: &str) -> LegacyPipelineResult<'static> {
        match value {
            "ok" => LegacyPipelineResult::Completed,
            "interrupted" => LegacyPipelineResult::Interrupted,
            "v2m_error" => LegacyPipelineResult::Vocal2MidiError {
                message: "legacy failure",
                details: "legacy details",
            },
            "generic" => LegacyPipelineResult::OtherError { display: "boom" },
            _ => panic!("unknown pipeline result {value}"),
        }
    }

    fn make_case_dir(case_id: &str) -> PathBuf {
        let base_dir = std::env::temp_dir().join(format!(
            "v2m-application-job-contract-{}-{case_id}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&base_dir);
        fs::create_dir_all(&base_dir).unwrap();
        base_dir
    }

    fn assert_model_details(details: &str, labels: &[&str]) {
        let parts: Vec<_> = if details.is_empty() {
            Vec::new()
        } else {
            details.split("; ").collect()
        };
        assert_eq!(parts.len(), labels.len(), "details: {details}");
        for (part, label) in parts.iter().zip(labels.iter()) {
            let expected_prefix = format!("{label}不存在或无效: ");
            assert!(
                part.starts_with(&expected_prefix),
                "{part:?} should start with {expected_prefix:?}"
            );
        }
    }

    #[test]
    fn application_job_contract_follows_parity_fixture_table() {
        for (line_index, line) in FIXTURES.lines().enumerate() {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let fields: Vec<_> = line.split('\t').collect();
            assert_eq!(fields.len(), 12, "fixture line {}", line_index + 1);

            let [
                case_id,
                output_lyrics_raw,
                game_path_marker,
                hfa_path_marker,
                asr_path_marker,
                cancel_checker_raw,
                pipeline_result_raw,
                expected_kind,
                expected_message_raw,
                expected_details_raw,
                expected_detail_labels_raw,
                expected_pipeline_called_raw,
            ] = fields.as_slice()
            else {
                unreachable!();
            };

            let case_dir = make_case_dir(case_id);
            let game_model_dir = resolve_fixture_path(&case_dir, game_path_marker, "game");
            let hfa_model_dir = resolve_fixture_path(&case_dir, hfa_path_marker, "hfa");
            let asr_model_path = resolve_fixture_path(&case_dir, asr_path_marker, "asr");
            let config = ApplicationJobConfig {
                game_model_dir: &game_model_dir,
                hfa_model_dir: &hfa_model_dir,
                asr_model_path: &asr_model_path,
                output_lyrics: parse_bool(output_lyrics_raw),
                cancel_before_start: parse_cancel_before_start(cancel_checker_raw),
            };
            let pipeline_result = parse_pipeline_result(pipeline_result_raw);
            let expected_message = decode_empty(expected_message_raw);
            let expected_details = decode_empty(expected_details_raw);
            let expected_pipeline_called = parse_bool(expected_pipeline_called_raw);
            let expected_detail_labels = labels_from_fixture(expected_detail_labels_raw);

            let mut pipeline_called = false;
            let result = run_auto_lyric_job_contract(&config, || {
                pipeline_called = true;
                pipeline_result
            });

            match (*expected_kind, result) {
                ("ok", Ok(())) => {}
                ("ok", Err(error)) => panic!(
                    "fixture line {} failed unexpectedly: {error:?}",
                    line_index + 1
                ),
                (_, Ok(())) => panic!("fixture line {} passed unexpectedly", line_index + 1),
                (kind, Err(error)) => {
                    assert_eq!(error.kind(), kind, "fixture line {}", line_index + 1);
                    assert_eq!(
                        error.message(),
                        expected_message,
                        "fixture line {}",
                        line_index + 1
                    );
                    if expected_details == "__model_path_details__" {
                        assert_model_details(error.details(), &expected_detail_labels);
                    } else {
                        assert_eq!(
                            error.details(),
                            expected_details,
                            "fixture line {}",
                            line_index + 1
                        );
                    }
                }
            }

            assert_eq!(
                pipeline_called,
                expected_pipeline_called,
                "fixture line {}",
                line_index + 1
            );
            let _ = fs::remove_dir_all(&case_dir);
        }
    }

    #[test]
    fn no_lyrics_mode_only_requires_game_path() {
        let case_dir = make_case_dir("no-lyrics-only-game");
        let game_model_dir = resolve_fixture_path(&case_dir, "exists_dir", "game");
        let hfa_model_dir = resolve_fixture_path(&case_dir, "missing", "hfa");
        let asr_model_path = resolve_fixture_path(&case_dir, "missing", "asr");
        let config = ApplicationJobConfig {
            game_model_dir: &game_model_dir,
            hfa_model_dir: &hfa_model_dir,
            asr_model_path: &asr_model_path,
            output_lyrics: false,
            cancel_before_start: false,
        };

        assert_eq!(
            run_auto_lyric_job_contract(&config, || LegacyPipelineResult::Completed),
            Ok(())
        );
        let _ = fs::remove_dir_all(&case_dir);
    }

    #[test]
    fn cancellation_before_start_does_not_call_pipeline() {
        let case_dir = make_case_dir("cancellation-order");
        let game_model_dir = resolve_fixture_path(&case_dir, "exists_dir", "game");
        let hfa_model_dir = resolve_fixture_path(&case_dir, "exists_dir", "hfa");
        let asr_model_path = resolve_fixture_path(&case_dir, "exists_dir", "asr");
        let config = ApplicationJobConfig {
            game_model_dir: &game_model_dir,
            hfa_model_dir: &hfa_model_dir,
            asr_model_path: &asr_model_path,
            output_lyrics: true,
            cancel_before_start: true,
        };

        let mut called = false;
        let error = run_auto_lyric_job_contract(&config, || {
            called = true;
            LegacyPipelineResult::Completed
        })
        .unwrap_err();

        assert!(!called);
        assert_eq!(error.kind(), "cancelled");
        assert_eq!(error.message(), CANCELLED_BEFORE_START_MESSAGE);
        let _ = fs::remove_dir_all(&case_dir);
    }
}
