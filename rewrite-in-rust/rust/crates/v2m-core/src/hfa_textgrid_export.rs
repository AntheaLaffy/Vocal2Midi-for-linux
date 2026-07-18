//! HubertFA TextGrid export planning.
//!
//! This module mirrors
//! `inference/HubertFA/tools/export_tool.py::Exporter.save_textgrids` as an
//! in-memory plan for the pinned `textgrid==1.6.1` long TextGrid writer subset.
//! Python remains the runtime owner for directory creation, file writes, status
//! printing, artifact copying, export dispatch, and production routing.

use crate::hfa_word::Word;
use std::error::Error;
use std::fmt;
use std::path::{Component, Path, PathBuf};

/// One prediction tuple consumed by the TextGrid exporter.
#[derive(Debug, Clone, PartialEq)]
pub struct HfaTextGridPrediction {
    pub wav_path: PathBuf,
    pub wav_length: f64,
    pub words: Vec<Word>,
}

/// Directory creation planned by the legacy exporter before each TextGrid write.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HfaTextGridDirectoryPlan {
    pub path: PathBuf,
    pub parents: bool,
    pub exist_ok: bool,
}

/// One UTF-8 TextGrid write planned by the legacy exporter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HfaTextGridPlannedFile {
    pub path: PathBuf,
    pub encoding: &'static str,
    pub content: String,
}

/// Ordered side-effect plan for one `save_textgrids` call.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HfaTextGridExportPlan {
    pub directories: Vec<HfaTextGridDirectoryPlan>,
    pub files: Vec<HfaTextGridPlannedFile>,
}

/// Python-compatible TextGrid export failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HfaTextGridExportError {
    exception_type: &'static str,
    message: String,
}

impl HfaTextGridExportError {
    /// Legacy Python exception type.
    pub const fn exception_type(&self) -> &'static str {
        self.exception_type
    }

    /// Exact legacy Python error message.
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for HfaTextGridExportError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for HfaTextGridExportError {}

/// Failed export plan with side effects that Python would already have
/// completed before the failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HfaTextGridExportFailure {
    pub error: HfaTextGridExportError,
    pub partial_plan: HfaTextGridExportPlan,
}

#[derive(Debug, Clone, PartialEq)]
struct TextGridInterval {
    min_time: TimeValue,
    max_time: TimeValue,
    mark: String,
}

#[derive(Debug, Clone, PartialEq)]
struct TimeValue {
    value: f64,
    text: String,
}

/// Plans the TextGrid files produced by `Exporter.save_textgrids`.
///
/// # Errors
///
/// Returns a Python-compatible error and partial side-effect plan when interval
/// construction, path projection, or TextGrid serialization fails.
pub fn plan_textgrid_export(
    predictions: &[HfaTextGridPrediction],
    output_folder: Option<&Path>,
) -> Result<HfaTextGridExportPlan, HfaTextGridExportFailure> {
    let output_folder = output_folder.filter(|path| !path.as_os_str().is_empty());
    let mut plan = HfaTextGridExportPlan::default();

    for prediction in predictions {
        let (word_intervals, phone_intervals) =
            build_prediction_tiers(prediction).map_err(|error| HfaTextGridExportFailure {
                error,
                partial_plan: plan.clone(),
            })?;

        let textgrid_path =
            planned_textgrid_path(&prediction.wav_path, output_folder).map_err(|error| {
                HfaTextGridExportFailure {
                    error,
                    partial_plan: plan.clone(),
                }
            })?;

        plan.directories.push(HfaTextGridDirectoryPlan {
            path: textgrid_path
                .parent()
                .map(Path::to_path_buf)
                .unwrap_or_default(),
            parents: true,
            exist_ok: true,
        });

        let content = render_textgrid(prediction.wav_length, &word_intervals, &phone_intervals)
            .map_err(|error| HfaTextGridExportFailure {
                error,
                partial_plan: plan.clone(),
            })?;

        plan.files.push(HfaTextGridPlannedFile {
            path: textgrid_path,
            encoding: "UTF-8",
            content,
        });
    }

    Ok(plan)
}

fn build_prediction_tiers(
    prediction: &HfaTextGridPrediction,
) -> Result<(Vec<TextGridInterval>, Vec<TextGridInterval>), HfaTextGridExportError> {
    let mut word_intervals = Vec::new();
    let mut phone_intervals = Vec::new();
    for word in &prediction.words {
        add_interval(
            &mut word_intervals,
            TextGridInterval::new(
                float_time(word.start),
                float_time(word.end),
                word.text.clone(),
            )?,
            0.0,
            prediction.wav_length,
        )?;

        for phoneme in &word.phonemes {
            add_interval(
                &mut phone_intervals,
                TextGridInterval::new(
                    phone_start_time(phoneme.start),
                    float_time(phoneme.end),
                    phoneme.text.clone(),
                )?,
                0.0,
                prediction.wav_length,
            )?;
        }
    }
    Ok((word_intervals, phone_intervals))
}

impl TextGridInterval {
    fn new(
        min_time: TimeValue,
        max_time: TimeValue,
        mark: String,
    ) -> Result<Self, HfaTextGridExportError> {
        if min_time.value >= max_time.value {
            return Err(value_error(format!(
                "({}, {})",
                min_time.text, max_time.text
            )));
        }
        Ok(Self {
            min_time,
            max_time,
            mark,
        })
    }
}

fn add_interval(
    intervals: &mut Vec<TextGridInterval>,
    interval: TextGridInterval,
    tier_min_time: f64,
    tier_max_time: f64,
) -> Result<(), HfaTextGridExportError> {
    if interval.min_time.value < tier_min_time {
        return Err(value_error(python_float_string(tier_min_time)));
    }
    if python_truthy_float(tier_max_time) && interval.max_time.value > tier_max_time {
        return Err(value_error(python_float_string(tier_max_time)));
    }

    let mut insertion_index = intervals.len();
    for (index, existing) in intervals.iter().enumerate() {
        if intervals_overlap(existing, &interval) {
            return Err(value_error(format!(
                "({}, {})",
                interval_repr(existing),
                interval_repr(&interval)
            )));
        }
        if !interval_lt(existing, &interval) {
            insertion_index = index;
            break;
        }
    }
    intervals.insert(insertion_index, interval);
    Ok(())
}

fn intervals_overlap(left: &TextGridInterval, right: &TextGridInterval) -> bool {
    right.min_time.value < left.max_time.value && left.min_time.value < right.max_time.value
}

fn interval_lt(left: &TextGridInterval, right: &TextGridInterval) -> bool {
    left.min_time.value < right.min_time.value
}

fn interval_repr(interval: &TextGridInterval) -> String {
    let mark = if interval.mark.is_empty() {
        "None"
    } else {
        &interval.mark
    };
    format!(
        "Interval({}, {}, {})",
        interval.min_time.text, interval.max_time.text, mark
    )
}

fn render_textgrid(
    max_time: f64,
    word_intervals: &[TextGridInterval],
    phone_intervals: &[TextGridInterval],
) -> Result<String, HfaTextGridExportError> {
    let max_time_text = textgrid_max_time_text(max_time, word_intervals, phone_intervals)?;
    let mut output = String::new();

    output.push_str("File type = \"ooTextFile\"\n");
    output.push_str("Object class = \"TextGrid\"\n\n");
    output.push_str("xmin = 0\n");
    output.push_str(&format!("xmax = {max_time_text}\n"));
    output.push_str("tiers? <exists>\n");
    output.push_str("size = 2\n");
    output.push_str("item []:\n");
    render_interval_tier(
        &mut output,
        1,
        "words",
        &max_time_text,
        max_time,
        word_intervals,
    );
    render_interval_tier(
        &mut output,
        2,
        "phones",
        &max_time_text,
        max_time,
        phone_intervals,
    );
    Ok(output)
}

fn textgrid_max_time_text(
    max_time: f64,
    word_intervals: &[TextGridInterval],
    phone_intervals: &[TextGridInterval],
) -> Result<String, HfaTextGridExportError> {
    if python_truthy_float(max_time) {
        return Ok(python_float_string(max_time));
    }

    let word = tier_bounds_fallback(max_time, word_intervals)?;
    let phone = tier_bounds_fallback(max_time, phone_intervals)?;
    if word.value >= phone.value {
        Ok(word.text)
    } else {
        Ok(phone.text)
    }
}

fn tier_bounds_fallback(
    max_time: f64,
    intervals: &[TextGridInterval],
) -> Result<TimeValue, HfaTextGridExportError> {
    if python_truthy_float(max_time) {
        Ok(float_time(max_time))
    } else {
        intervals
            .last()
            .map(|interval| interval.max_time.clone())
            .ok_or_else(|| index_error("list index out of range"))
    }
}

fn render_interval_tier(
    output: &mut String,
    index: usize,
    name: &str,
    max_time_text: &str,
    max_time: f64,
    intervals: &[TextGridInterval],
) {
    output.push_str(&format!("\titem [{index}]:\n"));
    output.push_str("\t\tclass = \"IntervalTier\"\n");
    output.push_str(&format!("\t\tname = \"{name}\"\n"));
    output.push_str("\t\txmin = 0.0\n");
    output.push_str(&format!("\t\txmax = {max_time_text}\n"));
    let intervals_with_gaps = fill_gaps(max_time, intervals);
    output.push_str(&format!(
        "\t\tintervals: size = {}\n",
        intervals_with_gaps.len()
    ));
    for (interval_index, interval) in intervals_with_gaps.iter().enumerate() {
        let item_index = interval_index + 1;
        output.push_str(&format!("\t\t\tintervals [{item_index}]:\n"));
        output.push_str(&format!("\t\t\t\txmin = {}\n", interval.min_time.text));
        output.push_str(&format!("\t\t\t\txmax = {}\n", interval.max_time.text));
        output.push_str(&format!(
            "\t\t\t\ttext = \"{}\"\n",
            format_mark(&interval.mark)
        ));
    }
}

fn fill_gaps(max_time: f64, intervals: &[TextGridInterval]) -> Vec<TextGridInterval> {
    let mut previous = float_time(0.0);
    let mut output = Vec::new();
    for interval in intervals {
        if previous.value < interval.min_time.value {
            output.push(TextGridInterval {
                min_time: previous.clone(),
                max_time: interval.min_time.clone(),
                mark: String::new(),
            });
        }
        output.push(interval.clone());
        previous = interval.max_time.clone();
    }
    if previous.value < max_time {
        output.push(TextGridInterval {
            min_time: previous,
            max_time: float_time(max_time),
            mark: String::new(),
        });
    }
    output
}

fn planned_textgrid_path(
    wav_path: &Path,
    output_folder: Option<&Path>,
) -> Result<PathBuf, HfaTextGridExportError> {
    let root = output_folder
        .map(python_pathlib_normalize)
        .unwrap_or_else(|| python_path_parent_normalize(wav_path));
    let textgrid_name = textgrid_file_name(wav_path)?;
    Ok(root.join("TextGrid").join(textgrid_name))
}

fn textgrid_file_name(wav_path: &Path) -> Result<PathBuf, HfaTextGridExportError> {
    let Some(file_name) = python_path_name(wav_path) else {
        return Err(value_error(format!(
            "PosixPath('{}') has an empty name",
            python_path_display_for_error(wav_path)
        )));
    };
    Ok(PathBuf::from(python_with_suffix_name(
        &file_name,
        ".TextGrid",
    )))
}

fn python_pathlib_normalize(path: &Path) -> PathBuf {
    let normalized: PathBuf = path
        .components()
        .filter(|component| !matches!(component, Component::CurDir))
        .collect();
    if has_python_double_slash_root(path) {
        let normalized_text = normalized.to_string_lossy();
        if normalized_text.starts_with('/') {
            return PathBuf::from(format!("/{normalized_text}"));
        }
    }
    normalized
}

fn python_path_parent_normalize(path: &Path) -> PathBuf {
    let Some(parent) = path.parent() else {
        return PathBuf::new();
    };
    if has_python_double_slash_root(path) && parent == Path::new("/") {
        return PathBuf::from("//");
    }
    python_pathlib_normalize(parent)
}

fn has_python_double_slash_root(path: &Path) -> bool {
    let text = path.to_string_lossy();
    text.starts_with("//") && !text.starts_with("///")
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

fn phone_start_time(value: f64) -> TimeValue {
    if value > 0.0 {
        float_time(value)
    } else {
        TimeValue {
            value: 0.0,
            text: "0".to_string(),
        }
    }
}

fn float_time(value: f64) -> TimeValue {
    TimeValue {
        value,
        text: python_float_string(value),
    }
}

fn python_truthy_float(value: f64) -> bool {
    value != 0.0
}

fn python_float_string(value: f64) -> String {
    if value.is_nan() {
        return "nan".to_string();
    }
    if value == f64::INFINITY {
        return "inf".to_string();
    }
    if value == f64::NEG_INFINITY {
        return "-inf".to_string();
    }

    let rendered = format!("{value:?}");
    let Some(exponent_index) = rendered.find('e') else {
        return rendered;
    };
    let (mantissa, exponent) = rendered.split_at(exponent_index);
    let exponent = exponent[1..]
        .parse::<i32>()
        .expect("Rust float debug exponent is an integer");
    format!("{mantissa}e{exponent:+03}")
}

fn format_mark(text: &str) -> String {
    text.replace('"', "\"\"")
}

fn value_error(message: impl Into<String>) -> HfaTextGridExportError {
    HfaTextGridExportError {
        exception_type: "ValueError",
        message: message.into(),
    }
}

fn index_error(message: impl Into<String>) -> HfaTextGridExportError {
    HfaTextGridExportError {
        exception_type: "IndexError",
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hfa_word::{Phoneme, Word};
    use serde_json::{Value, json};

    const FIXTURES: &str = include_str!("../../../../fixtures/hfa_textgrid_export_core.jsonl");

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

    fn decode_predictions(case: &Value) -> Vec<HfaTextGridPrediction> {
        case.get("predictions")
            .and_then(Value::as_array)
            .unwrap()
            .iter()
            .map(|prediction| HfaTextGridPrediction {
                wav_path: PathBuf::from(prediction.get("wav_path").unwrap().as_str().unwrap()),
                wav_length: decode_float(prediction.get("wav_length").unwrap_or(&json!(0.0))),
                words: prediction
                    .get("words")
                    .and_then(Value::as_array)
                    .map(|words| words.iter().map(decode_word).collect())
                    .unwrap_or_default(),
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
                .map(|phonemes| {
                    phonemes
                        .iter()
                        .map(|phoneme| Phoneme {
                            start: decode_float(phoneme.get("start").unwrap()),
                            end: decode_float(phoneme.get("end").unwrap()),
                            text: phoneme.get("text").unwrap().as_str().unwrap().to_string(),
                        })
                        .collect()
                })
                .unwrap_or_default(),
        }
    }

    fn run_case(case: &Value) -> Value {
        let output_folder = case.get("output_folder").and_then(Value::as_str);
        let output_folder = output_folder.map(Path::new);
        let predictions = decode_predictions(case);
        let repeat = case.get("repeat").and_then(Value::as_u64).unwrap_or(1);
        let calls = (0..repeat)
            .map(
                |_| match plan_textgrid_export(&predictions, output_folder) {
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

    fn project_plan(plan: &HfaTextGridExportPlan) -> Value {
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
    fn hfa_textgrid_export_core_fixture_parity() {
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
