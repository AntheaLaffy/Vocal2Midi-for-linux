//! HubertFA HTK label export planning.
//!
//! This module mirrors `inference/HubertFA/tools/export_tool.py::Exporter.save_htk`
//! as an in-memory plan. Python remains the runtime owner for directory
//! creation, file writes, status printing, export dispatch, and production
//! routing.

use crate::hfa_word::Word;
use std::error::Error;
use std::fmt;
use std::path::{Component, Path, PathBuf};

const HTK_PAD_VALUE: f64 = 10_000_000.0;

/// One prediction tuple consumed by the HTK exporter.
#[derive(Debug, Clone, PartialEq)]
pub struct HfaHtkPrediction {
    /// The wav path.
    pub wav_path: PathBuf,
    /// The wav length.
    pub wav_length: f64,
    /// The ordered words.
    pub words: Vec<Word>,
}

/// Directory creation planned by the legacy exporter before each write pair.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HfaHtkDirectoryPlan {
    /// The filesystem path.
    pub path: PathBuf,
    /// Whether parent directories are created.
    pub parents: bool,
    /// Whether an existing directory is accepted.
    pub exist_ok: bool,
}

/// One UTF-8 HTK `.lab` write planned by the legacy exporter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HfaHtkPlannedFile {
    /// The filesystem path.
    pub path: PathBuf,
    /// The encoding.
    pub encoding: &'static str,
    /// The content.
    pub content: String,
}

/// Ordered side-effect plan for one `save_htk` call.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HfaHtkExportPlan {
    /// The ordered directories.
    pub directories: Vec<HfaHtkDirectoryPlan>,
    /// The ordered files.
    pub files: Vec<HfaHtkPlannedFile>,
}

/// Python-compatible conversion failure from `int(float(time) * 10000000)`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HfaHtkExportError {
    exception_type: &'static str,
    message: String,
}

impl HfaHtkExportError {
    /// Legacy Python exception type.
    pub const fn exception_type(&self) -> &'static str {
        self.exception_type
    }

    /// Exact legacy Python error message.
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for HfaHtkExportError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for HfaHtkExportError {}

/// Failed export plan with side effects that Python would already have
/// completed before the conversion failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HfaHtkExportFailure {
    /// The error message.
    pub error: HfaHtkExportError,
    /// The partial plan.
    pub partial_plan: HfaHtkExportPlan,
}

/// Plans the HTK label files produced by `Exporter.save_htk`.
///
/// # Errors
///
/// Returns a Python-compatible conversion error and partial side-effect plan
/// when a timestamp cannot be converted by Python's `int(float(...))` path.
pub fn plan_htk_label_export(
    predictions: &[HfaHtkPrediction],
    output_folder: Option<&Path>,
) -> Result<HfaHtkExportPlan, HfaHtkExportFailure> {
    let output_folder = output_folder.filter(|path| !path.as_os_str().is_empty());
    let mut plan = HfaHtkExportPlan::default();
    let mut word_output = String::new();
    let mut phoneme_output = String::new();

    for prediction in predictions {
        let _wav_length = prediction.wav_length;
        for word in &prediction.words {
            let word_start = render_htk_time(word.start).map_err(|error| HfaHtkExportFailure {
                error,
                partial_plan: plan.clone(),
            })?;
            let word_end = render_htk_time(word.end).map_err(|error| HfaHtkExportFailure {
                error,
                partial_plan: plan.clone(),
            })?;
            word_output.push_str(&format!("{word_start} {word_end} {}\n", word.text));

            for phoneme in &word.phonemes {
                let phoneme_start =
                    render_htk_time(phoneme.start).map_err(|error| HfaHtkExportFailure {
                        error,
                        partial_plan: plan.clone(),
                    })?;
                let phoneme_end =
                    render_htk_time(phoneme.end).map_err(|error| HfaHtkExportFailure {
                        error,
                        partial_plan: plan.clone(),
                    })?;
                phoneme_output
                    .push_str(&format!("{phoneme_start} {phoneme_end} {}\n", phoneme.text));
            }
        }

        let (phone_path, word_path) =
            planned_paths(&prediction.wav_path, output_folder).map_err(|error| {
                HfaHtkExportFailure {
                    error,
                    partial_plan: plan.clone(),
                }
            })?;
        plan.directories.push(HfaHtkDirectoryPlan {
            path: phone_path
                .parent()
                .map(Path::to_path_buf)
                .unwrap_or_default(),
            parents: true,
            exist_ok: true,
        });
        plan.directories.push(HfaHtkDirectoryPlan {
            path: word_path
                .parent()
                .map(Path::to_path_buf)
                .unwrap_or_default(),
            parents: true,
            exist_ok: true,
        });
        plan.files.push(HfaHtkPlannedFile {
            path: phone_path,
            encoding: "utf-8",
            content: phoneme_output.clone(),
        });
        plan.files.push(HfaHtkPlannedFile {
            path: word_path,
            encoding: "utf-8",
            content: word_output.clone(),
        });
    }

    Ok(plan)
}

fn planned_paths(
    wav_path: &Path,
    output_folder: Option<&Path>,
) -> Result<(PathBuf, PathBuf), HfaHtkExportError> {
    let root = output_folder
        .map(python_pathlib_normalize)
        .unwrap_or_else(|| {
            wav_path
                .parent()
                .map(python_pathlib_normalize)
                .unwrap_or_default()
        });
    let lab_name = lab_file_name(wav_path)?;
    Ok((
        root.join("HTK").join("Phones").join(&lab_name),
        root.join("HTK").join("Words").join(lab_name),
    ))
}

fn python_pathlib_normalize(path: &Path) -> PathBuf {
    path.components()
        .filter(|component| !matches!(component, Component::CurDir))
        .collect()
}

fn lab_file_name(wav_path: &Path) -> Result<PathBuf, HfaHtkExportError> {
    let Some(file_name) = python_path_name(wav_path) else {
        return Err(HfaHtkExportError {
            exception_type: "ValueError",
            message: format!(
                "PosixPath('{}') has an empty name",
                python_path_display_for_error(wav_path)
            ),
        });
    };
    Ok(PathBuf::from(python_with_suffix_name(&file_name, ".lab")))
}

fn python_path_name(path: &Path) -> Option<String> {
    let mut name = None;
    for component in path.components() {
        match component {
            Component::Normal(value) => name = Some(value.to_string_lossy().into_owned()),
            Component::ParentDir => name = Some("..".to_string()),
            Component::CurDir | Component::RootDir | Component::Prefix(_) => {}
        }
    }
    name
}

fn python_with_suffix_name(name: &str, suffix: &str) -> String {
    let old_suffix_start = name
        .rfind('.')
        .filter(|&index| index > 0 && index < name.len() - 1);
    match old_suffix_start {
        Some(index) => format!("{}{}", &name[..index], suffix),
        None => format!("{name}{suffix}"),
    }
}

fn python_path_display_for_error(path: &Path) -> String {
    let normalized = python_pathlib_normalize(path);
    if normalized.as_os_str().is_empty() {
        ".".to_string()
    } else {
        normalized.to_string_lossy().into_owned()
    }
}

fn render_htk_time(value: f64) -> Result<String, HfaHtkExportError> {
    let scaled = value * HTK_PAD_VALUE;
    if scaled.is_nan() {
        return Err(HfaHtkExportError {
            exception_type: "ValueError",
            message: "cannot convert float NaN to integer".to_string(),
        });
    }
    if scaled.is_infinite() {
        return Err(HfaHtkExportError {
            exception_type: "OverflowError",
            message: "cannot convert float infinity to integer".to_string(),
        });
    }
    Ok(python_int_string_from_f64(scaled))
}

fn python_int_string_from_f64(value: f64) -> String {
    let bits = value.to_bits();
    let sign = bits >> 63 != 0;
    let exponent_bits = ((bits >> 52) & 0x7ff) as i32;
    let fraction = bits & ((1u64 << 52) - 1);
    let (significand, exponent) = if exponent_bits == 0 {
        (fraction, 1 - 1023 - 52)
    } else {
        ((1u64 << 52) | fraction, exponent_bits - 1023 - 52)
    };

    if significand == 0 {
        return "0".to_string();
    }

    let magnitude = if exponent >= 0 {
        decimal_from_u64_shifted(significand, exponent as usize)
    } else {
        let shift = (-exponent) as u32;
        if shift >= 64 {
            "0".to_string()
        } else {
            (significand >> shift).to_string()
        }
    };

    if sign && magnitude != "0" {
        format!("-{magnitude}")
    } else {
        magnitude
    }
}

fn decimal_from_u64_shifted(value: u64, shift: usize) -> String {
    let mut digits = value.to_string().into_bytes();
    for _ in 0..shift {
        decimal_multiply_by_two(&mut digits);
    }
    String::from_utf8(digits).expect("decimal digits")
}

fn decimal_multiply_by_two(digits: &mut Vec<u8>) {
    let mut carry = 0u8;
    for digit in digits.iter_mut().rev() {
        let doubled = (*digit - b'0') * 2 + carry;
        *digit = b'0' + (doubled % 10);
        carry = doubled / 10;
    }
    if carry > 0 {
        digits.insert(0, b'0' + carry);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hfa_word::{Phoneme, Word};
    use serde_json::{Value, json};

    const FIXTURES: &str = include_str!("../../../../fixtures/hfa_htk_label_export_core.jsonl");

    fn decode_float(value: &Value) -> f64 {
        if let Some(object) = value.as_object() {
            if let Some(kind) = object.get("$float").and_then(Value::as_str) {
                return match kind {
                    "nan" => f64::NAN,
                    "+inf" => f64::INFINITY,
                    "-inf" => f64::NEG_INFINITY,
                    "-0.0" => -0.0,
                    other => panic!("unknown float marker {other:?}"),
                };
            }
        }
        value.as_f64().unwrap()
    }

    fn decode_predictions(case: &Value) -> Vec<HfaHtkPrediction> {
        case.get("predictions")
            .and_then(Value::as_array)
            .unwrap()
            .iter()
            .map(|prediction| HfaHtkPrediction {
                wav_path: PathBuf::from(prediction.get("wav_path").unwrap().as_str().unwrap()),
                wav_length: decode_float(prediction.get("wav_length").unwrap_or(&json!(0.0))),
                words: prediction
                    .get("words")
                    .and_then(Value::as_array)
                    .unwrap_or(&Vec::new())
                    .iter()
                    .map(decode_word)
                    .collect(),
            })
            .collect()
    }

    fn decode_word(value: &Value) -> Word {
        Word {
            start: decode_float(value.get("start").unwrap()),
            end: decode_float(value.get("end").unwrap()),
            text: value.get("text").unwrap().as_str().unwrap().to_string(),
            phonemes: value
                .get("phonemes")
                .and_then(Value::as_array)
                .unwrap_or(&Vec::new())
                .iter()
                .map(|phoneme| Phoneme {
                    start: decode_float(phoneme.get("start").unwrap()),
                    end: decode_float(phoneme.get("end").unwrap()),
                    text: phoneme.get("text").unwrap().as_str().unwrap().to_string(),
                })
                .collect(),
        }
    }

    fn run_case(case: &Value) -> Value {
        let output_folder = case.get("output_folder").and_then(Value::as_str);
        let output_folder = output_folder.map(Path::new);
        let predictions = decode_predictions(case);
        let repeat = case.get("repeat").and_then(Value::as_u64).unwrap_or(1);
        let calls = (0..repeat)
            .map(
                |_| match plan_htk_label_export(&predictions, output_folder) {
                    Ok(plan) => json!({"ok": project_plan(&plan)}),
                    Err(failure) => json!({
                        "error": {
                            "type": failure.error.exception_type(),
                            "message": failure.error.message(),
                        },
                        "partial_plan": project_plan(&failure.partial_plan),
                    }),
                },
            )
            .collect::<Vec<_>>();
        json!({"calls": calls})
    }

    fn project_plan(plan: &HfaHtkExportPlan) -> Value {
        json!({
            "directories": plan.directories.iter().map(|directory| {
                json!({
                    "path": path_text(&directory.path),
                    "parents": directory.parents,
                    "exist_ok": directory.exist_ok,
                })
            }).collect::<Vec<_>>(),
            "files": plan.files.iter().map(|file| {
                json!({
                    "path": path_text(&file.path),
                    "encoding": file.encoding,
                    "content": file.content,
                })
            }).collect::<Vec<_>>(),
        })
    }

    fn path_text(path: &Path) -> String {
        path.to_string_lossy().into_owned()
    }

    #[test]
    fn hfa_htk_label_export_core_fixture_parity() {
        for line in FIXTURES.lines().filter(|line| !line.is_empty()) {
            let case: Value = serde_json::from_str(line).unwrap();
            assert_eq!(
                run_case(&case),
                *case.get("expect").unwrap(),
                "{}",
                case.get("case_id").and_then(Value::as_str).unwrap()
            );
        }
    }
}
