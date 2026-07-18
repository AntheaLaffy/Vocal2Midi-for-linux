//! Batch CLI planning and source-index helpers.
//!
//! This module mirrors deterministic planning behavior from
//! `scripts/slice_asr_cli.py`. Python remains the runtime owner for audio IO,
//! model execution, ASR/RMVPE lifecycles, and chunk writing.

use std::collections::{BTreeMap, BTreeSet};

use encoding_rs::{Encoding, GB18030, GBK};
use md5::{Digest, Md5};
use serde_json::{Map, Value};

#[cfg(test)]
use serde_json::json;

const INPUT_AUDIO_EXTENSIONS: &[&str] = &[".flac", ".m4a", ".mp3", ".wav"];
const DEFAULT_SLICE_METHOD: &str = "default";
const SLICE_METHOD_CHOICES: &[&str] = &["default", "smart", "heuristic", "grid"];
const SLICE_METHOD_ALIASES: &[(&str, &str)] = &[
    ("auto", "default"),
    ("default", "default"),
    ("smart", "smart"),
    ("heuristic", "heuristic"),
    ("grid", "grid"),
    ("默认切片", "default"),
    ("智能切片", "smart"),
    ("启发式切片", "heuristic"),
    ("网格搜索切片", "grid"),
];
const SLICE_METHOD_KEYWORDS: &[(&str, &[&str])] = &[
    ("smart", &["smart", "智能"]),
    ("heuristic", &["heuristic", "启发式"]),
    ("grid", &["grid", "网格"]),
    ("default", &["default", "默认", "auto"]),
];

/// Fixture-backed file-tree entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FsEntry {
    /// The filesystem path.
    pub path: String,
    /// The kind.
    pub kind: FsEntryKind,
    /// The content.
    pub content: String,
}

/// File-tree entry kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FsEntryKind {
    /// Represents the Python-compatible file case.
    File,
    /// Represents the Python-compatible dir case.
    Dir,
}

/// Legacy-compatible planning error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanningError {
    /// The message text.
    pub message: String,
}

/// Fake processing outcome for batch-loop accounting fixtures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessOutcome {
    /// The chunks.
    pub chunks: usize,
    /// The labs.
    pub labs: usize,
    /// The optional error message.
    pub error: Option<String>,
}

/// Options for the fake batch-loop planner.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchPlanOptions {
    /// The input dir.
    pub input_dir: String,
    /// The output directory.
    pub output_dir: String,
    /// Whether file discovery is recursive.
    pub recursive: bool,
    /// The file batch size.
    pub file_batch_size: i64,
    /// Whether existing outputs should be reprocessed.
    pub no_skip_existing: bool,
    /// The md5 errors.
    pub md5_errors: BTreeSet<String>,
    /// The process outcomes.
    pub process_outcomes: BTreeMap<String, ProcessOutcome>,
}

/// Processed file record from the fake batch-loop planner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessedFile {
    /// The filesystem path.
    pub path: String,
    /// The output key.
    pub output_key: String,
    /// The chunks.
    pub chunks: usize,
    /// The labs.
    pub labs: usize,
}

/// Skipped file record from the fake batch-loop planner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkippedFile {
    /// The filesystem path.
    pub path: String,
    /// The reason.
    pub reason: String,
    /// The optional output key.
    pub output_key: Option<String>,
}

/// Fake batch-loop accounting result.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchPlanResult {
    /// The ordered audio files.
    pub audio_files: Vec<String>,
    /// The ordered batches.
    pub batches: Vec<Vec<String>>,
    /// The total chunks.
    pub total_chunks: usize,
    /// The total labs.
    pub total_labs: usize,
    /// The skipped existing.
    pub skipped_existing: usize,
    /// The skipped failed.
    pub skipped_failed: usize,
    /// The ordered processed.
    pub processed: Vec<ProcessedFile>,
    /// The ordered skipped.
    pub skipped: Vec<SkippedFile>,
    /// The source index.
    pub source_index: Map<String, Value>,
}

impl PlanningError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

/// Mirrors `batch_iter` grouping and its batch-size error.
///
/// # Errors
///
/// Returns [`PlanningError`] when `batch_size` is not positive.
pub fn batch_iter(items: &[String], batch_size: i64) -> Result<Vec<Vec<String>>, PlanningError> {
    if batch_size <= 0 {
        return Err(PlanningError::new("batch_size must be greater than 0"));
    }
    Ok(items
        .chunks(batch_size as usize)
        .map(|chunk| chunk.to_vec())
        .collect())
}

/// Mirrors legacy mojibake repair candidates used by slicing-method parsing.
pub fn repair_text_candidates(text: &str) -> Vec<String> {
    let stripped = text.trim().to_string();
    let mut candidates = vec![stripped.clone()];
    for encoding in [GB18030, GBK] {
        if let Some(repaired) = encode_then_decode_utf8_ignore(encoding, &stripped) {
            let repaired = repaired.trim().to_string();
            if !repaired.is_empty() && !candidates.contains(&repaired) {
                candidates.push(repaired);
            }
        }
    }
    candidates
}

/// Mirrors `normalize_slicing_method`.
///
/// # Errors
///
/// Returns [`PlanningError`] when no exact, repaired, or keyword alias matches
/// a supported slicing method.
pub fn normalize_slicing_method(method: Option<&str>) -> Result<String, PlanningError> {
    let Some(method) = method else {
        return Ok(DEFAULT_SLICE_METHOD.to_string());
    };

    let mut candidates = Vec::new();
    for candidate in repair_text_candidates(method) {
        let lowered = candidate.to_lowercase();
        for value in [candidate, lowered] {
            if !value.is_empty() && !candidates.contains(&value) {
                candidates.push(value);
            }
        }
    }

    for candidate in &candidates {
        if let Some((_, normalized)) = SLICE_METHOD_ALIASES
            .iter()
            .find(|(alias, _)| *alias == candidate)
        {
            return Ok((*normalized).to_string());
        }
    }

    for candidate in &candidates {
        for (normalized, keywords) in SLICE_METHOD_KEYWORDS {
            if keywords.iter().any(|keyword| candidate.contains(keyword)) {
                return Ok((*normalized).to_string());
            }
        }
    }

    Err(PlanningError::new(format!(
        "Unsupported slicing method: {}. Supported values: {}",
        python_string_repr(method),
        SLICE_METHOD_CHOICES.join(", ")
    )))
}

/// Mirrors `resolve_slice_bounds`.
///
/// # Errors
///
/// Returns [`PlanningError`] when only one bound is supplied, a bound is out of
/// range, or the minimum exceeds the maximum.
pub fn resolve_slice_bounds(
    min_seconds: Option<f64>,
    max_seconds: Option<f64>,
) -> Result<Option<(f64, f64)>, PlanningError> {
    match (min_seconds, max_seconds) {
        (None, None) => Ok(None),
        (None, Some(_)) | (Some(_), None) => Err(PlanningError::new(
            "--min-seconds and --max-seconds must be provided together",
        )),
        (Some(min_seconds), Some(max_seconds)) => {
            if min_seconds < 0.0 {
                return Err(PlanningError::new(
                    "--min-seconds must be greater than or equal to 0",
                ));
            }
            if max_seconds <= 0.0 {
                return Err(PlanningError::new("--max-seconds must be greater than 0"));
            }
            if min_seconds > max_seconds {
                return Err(PlanningError::new(
                    "--min-seconds must be less than or equal to --max-seconds",
                ));
            }
            Ok(Some((min_seconds, max_seconds)))
        }
    }
}

/// Mirrors audio file collection over a fixture-backed file tree.
pub fn collect_audio_files(entries: &[FsEntry], input_dir: &str, recursive: bool) -> Vec<String> {
    let mut files = entries
        .iter()
        .filter(|entry| entry.kind == FsEntryKind::File)
        .filter_map(|entry| {
            let relative = strip_prefix_path(&entry.path, input_dir)?;
            if relative.is_empty() {
                return None;
            }
            if !recursive && relative.contains('/') {
                return None;
            }
            if is_audio_path(relative) {
                Some(relative.to_string())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    files.sort();
    files
}

/// Mirrors `safe_stem`.
pub fn safe_stem(path: &str) -> String {
    file_stem(path).replace(' ', "_")
}

/// Mirrors `file_md5` for fixture-provided file bytes.
pub fn file_md5_bytes(bytes: &[u8]) -> String {
    let mut digest = Md5::new();
    digest.update(bytes);
    format!("{:x}", digest.finalize())
}

/// Mirrors `source_key` when the source MD5 is already available.
pub fn source_key(audio_path: &str, source_md5: &str) -> String {
    let prefix = &source_md5[..source_md5.len().min(8)];
    format!("{}_{}", safe_stem(audio_path), prefix)
}

/// Mirrors `source_index_path`.
pub fn source_index_path(output_dir: &str) -> String {
    path_join(&path_join(output_dir, "jsons"), "_source_index.json")
}

/// Mirrors `load_source_index` from optional JSON text.
pub fn load_source_index_from_content(content: Option<&str>) -> Map<String, Value> {
    content
        .and_then(|content| serde_json::from_str::<Value>(content).ok())
        .and_then(|value| value.as_object().cloned())
        .unwrap_or_default()
}

/// Mirrors `update_source_index`.
pub fn update_source_index(
    index: &mut Map<String, Value>,
    audio_path: &str,
    resolved_source_path: &str,
    output_key: &str,
    source_md5: &str,
    chunks: usize,
    labs: usize,
) {
    let mut record = Map::new();
    record.insert(
        "output_key".to_string(),
        Value::String(output_key.to_string()),
    );
    record.insert(
        "source_name".to_string(),
        Value::String(file_name(audio_path).to_string()),
    );
    record.insert(
        "source_path".to_string(),
        Value::String(resolved_source_path.to_string()),
    );
    record.insert("chunks".to_string(), Value::from(chunks as u64));
    record.insert("labs".to_string(), Value::from(labs as u64));
    index.insert(source_md5.to_string(), Value::Object(record));
}

/// Renders source-index JSON using the same two-space pretty layout as Python.
pub fn render_source_index_json(index: &Map<String, Value>) -> String {
    let mut output = String::from("{\n");
    for (record_index, (source_md5, record)) in index.iter().enumerate() {
        if record_index > 0 {
            output.push_str(",\n");
        }
        output.push_str(&format!("  {}: ", json_compact_str(source_md5)));
        if let Some(record) = record.as_object() {
            output.push_str("{\n");
            let fields = legacy_source_index_fields(record);
            for (field_index, field) in fields.iter().enumerate() {
                if field_index > 0 {
                    output.push_str(",\n");
                }
                output.push_str(&format!(
                    "    {}: {}",
                    json_compact_str(field),
                    json_compact_value(&record[field])
                ));
            }
            output.push_str("\n  }");
        } else {
            output.push_str(&json_compact_value(record));
        }
    }
    output.push_str("\n}");
    output
}

/// Mirrors direct output-tree skip detection.
pub fn has_existing_outputs(
    audio_path: &str,
    entries: &[FsEntry],
    output_dir: &str,
    source_md5: &str,
    recursive_output: bool,
) -> bool {
    let stem = source_key(audio_path, source_md5);
    let mut lab_dir = path_join(output_dir, "labs");
    let mut slice_dir = path_join(output_dir, "slices");
    if recursive_output {
        lab_dir = path_join(&lab_dir, &stem);
        slice_dir = path_join(&slice_dir, &stem);
    }

    if is_file(
        entries,
        &path_join(&path_join(output_dir, "jsons"), &format!("{stem}.json")),
    ) {
        return true;
    }
    if is_dir(entries, &lab_dir)
        && direct_child_files(entries, &lab_dir)
            .iter()
            .any(|name| name.starts_with(&format!("{stem}_chunk")) && name.ends_with(".lab"))
    {
        return true;
    }
    if is_dir(entries, &slice_dir)
        && direct_child_files(entries, &slice_dir)
            .iter()
            .any(|name| name.starts_with(&format!("{stem}_chunk")) && name.ends_with(".wav"))
    {
        return true;
    }
    false
}

/// Mirrors md5-index skip detection.
///
/// # Errors
///
/// Returns [`PlanningError`] when a legacy index record has an invalid output
/// key shape.
pub fn index_has_completed_output(
    index: &Map<String, Value>,
    source_md5: &str,
    entries: &[FsEntry],
    output_dir: &str,
) -> Result<Option<String>, PlanningError> {
    let Some(record) = index.get(source_md5).and_then(Value::as_object) else {
        return Ok(None);
    };
    let Some(key) = legacy_output_key(record)? else {
        return Ok(None);
    };

    if is_file(
        entries,
        &path_join(&path_join(output_dir, "jsons"), &format!("{key}.json")),
    ) {
        return Ok(Some(key));
    }
    let lab_dir = path_join(&path_join(output_dir, "labs"), &key);
    if is_dir(entries, &lab_dir)
        && direct_child_files(entries, &lab_dir)
            .iter()
            .any(|name| name.ends_with(".lab"))
    {
        return Ok(Some(key));
    }
    let slice_dir = path_join(&path_join(output_dir, "slices"), &key);
    if is_dir(entries, &slice_dir)
        && direct_child_files(entries, &slice_dir)
            .iter()
            .any(|name| name.ends_with(".wav"))
    {
        return Ok(Some(key));
    }
    Ok(None)
}

/// Mirrors the main-loop accounting shape with fake process outcomes.
///
/// # Errors
///
/// Returns [`PlanningError`] for an invalid file batch size or malformed legacy
/// index record reached during skip detection.
pub fn plan_batch_loop(
    source_entries: &[FsEntry],
    output_entries: &[FsEntry],
    mut source_index: Map<String, Value>,
    options: &BatchPlanOptions,
) -> Result<BatchPlanResult, PlanningError> {
    let audio_files = collect_audio_files(source_entries, &options.input_dir, options.recursive);
    let batches = batch_iter(&audio_files, options.file_batch_size)?;
    let mut total_chunks = 0;
    let mut total_labs = 0;
    let mut skipped_existing = 0;
    let mut skipped_failed = 0;
    let mut processed = Vec::new();
    let mut skipped = Vec::new();

    for file_batch in &batches {
        for audio_file in file_batch {
            let audio_path = path_join(&options.input_dir, audio_file);
            if options.md5_errors.contains(&audio_path) {
                skipped_failed += 1;
                skipped.push(SkippedFile {
                    path: audio_path,
                    reason: "md5_failed".to_string(),
                    output_key: None,
                });
                continue;
            }

            let source_md5 = file_md5_bytes(file_content(source_entries, &audio_path).as_bytes());
            let output_key = source_key(&audio_path, &source_md5);
            if !options.no_skip_existing {
                if let Some(indexed_key) = index_has_completed_output(
                    &source_index,
                    &source_md5,
                    output_entries,
                    &options.output_dir,
                )? {
                    skipped_existing += 1;
                    skipped.push(SkippedFile {
                        path: audio_path,
                        reason: "indexed".to_string(),
                        output_key: Some(indexed_key),
                    });
                    continue;
                }
                if has_existing_outputs(
                    &audio_path,
                    output_entries,
                    &options.output_dir,
                    &source_md5,
                    true,
                ) {
                    skipped_existing += 1;
                    skipped.push(SkippedFile {
                        path: audio_path,
                        reason: "existing".to_string(),
                        output_key: Some(output_key),
                    });
                    continue;
                }
            }

            let outcome = options
                .process_outcomes
                .get(&audio_path)
                .cloned()
                .unwrap_or(ProcessOutcome {
                    chunks: 0,
                    labs: 0,
                    error: None,
                });
            if outcome.error.is_some() {
                skipped_failed += 1;
                skipped.push(SkippedFile {
                    path: audio_path,
                    reason: "process_failed".to_string(),
                    output_key: None,
                });
                continue;
            }

            update_source_index(
                &mut source_index,
                &audio_path,
                &format!("__case__/{audio_path}"),
                &output_key,
                &source_md5,
                outcome.chunks,
                outcome.labs,
            );
            total_chunks += outcome.chunks;
            total_labs += outcome.labs;
            processed.push(ProcessedFile {
                path: audio_path,
                output_key,
                chunks: outcome.chunks,
                labs: outcome.labs,
            });
        }
    }

    Ok(BatchPlanResult {
        audio_files,
        batches,
        total_chunks,
        total_labs,
        skipped_existing,
        skipped_failed,
        processed,
        skipped,
        source_index,
    })
}

fn encode_then_decode_utf8_ignore(encoding: &'static Encoding, text: &str) -> Option<String> {
    let (encoded, _, had_errors) = encoding.encode(text);
    if had_errors {
        return None;
    }
    Some(decode_utf8_ignore(encoded.as_ref()))
}

fn decode_utf8_ignore(bytes: &[u8]) -> String {
    let mut output = String::new();
    let mut offset = 0;
    while offset < bytes.len() {
        match std::str::from_utf8(&bytes[offset..]) {
            Ok(valid) => {
                output.push_str(valid);
                break;
            }
            Err(error) => {
                let valid_up_to = error.valid_up_to();
                if valid_up_to > 0 {
                    output.push_str(
                        std::str::from_utf8(&bytes[offset..offset + valid_up_to]).unwrap(),
                    );
                    offset += valid_up_to;
                } else {
                    offset += error.error_len().unwrap_or(1);
                }
            }
        }
    }
    output
}

fn is_audio_path(path: &str) -> bool {
    let extension = path_extension(path).to_lowercase();
    INPUT_AUDIO_EXTENSIONS.contains(&extension.as_str())
}

fn path_extension(path: &str) -> String {
    let name = file_name(path);
    name.rfind('.')
        .map(|index| name[index..].to_string())
        .unwrap_or_default()
}

fn file_name(path: &str) -> &str {
    path.rsplit_once('/').map_or(path, |(_, name)| name)
}

fn file_stem(path: &str) -> String {
    let name = file_name(path);
    name.rfind('.')
        .map(|index| name[..index].to_string())
        .unwrap_or_else(|| name.to_string())
}

fn strip_prefix_path<'a>(path: &'a str, prefix: &str) -> Option<&'a str> {
    let prefix = prefix.trim_end_matches('/');
    if path == prefix {
        return Some("");
    }
    path.strip_prefix(&format!("{prefix}/"))
}

fn path_join(left: &str, right: &str) -> String {
    let left = left.trim_end_matches('/');
    let right = right.trim_start_matches('/');
    if left.is_empty() {
        right.to_string()
    } else if right.is_empty() {
        left.to_string()
    } else {
        format!("{left}/{right}")
    }
}

fn is_file(entries: &[FsEntry], path: &str) -> bool {
    entries
        .iter()
        .any(|entry| entry.kind == FsEntryKind::File && entry.path == path)
}

fn is_dir(entries: &[FsEntry], dir: &str) -> bool {
    let prefix = format!("{}/", dir.trim_end_matches('/'));
    entries.iter().any(|entry| {
        (entry.kind == FsEntryKind::Dir && entry.path == dir)
            || (entry.kind == FsEntryKind::File && entry.path.starts_with(&prefix))
    })
}

fn direct_child_files(entries: &[FsEntry], dir: &str) -> Vec<String> {
    let prefix = format!("{}/", dir.trim_end_matches('/'));
    entries
        .iter()
        .filter(|entry| entry.kind == FsEntryKind::File)
        .filter_map(|entry| entry.path.strip_prefix(&prefix))
        .filter(|remaining| !remaining.contains('/'))
        .map(str::to_string)
        .collect()
}

fn file_content(entries: &[FsEntry], path: &str) -> String {
    entries
        .iter()
        .find(|entry| entry.kind == FsEntryKind::File && entry.path == path)
        .map(|entry| entry.content.clone())
        .unwrap_or_default()
}

fn legacy_source_index_fields(record: &Map<String, Value>) -> Vec<String> {
    let mut fields = Vec::new();
    for field in ["output_key", "source_name", "source_path", "chunks", "labs"] {
        if record.contains_key(field) {
            fields.push(field.to_string());
        }
    }
    for field in record.keys() {
        if !fields.iter().any(|known| known == field) {
            fields.push(field.clone());
        }
    }
    fields
}

fn json_compact_str(value: &str) -> String {
    serde_json::to_string(value).unwrap()
}

fn json_compact_value(value: &Value) -> String {
    serde_json::to_string(value).unwrap()
}

fn legacy_output_key(record: &Map<String, Value>) -> Result<Option<String>, PlanningError> {
    let Some(value) = record.get("output_key") else {
        return Ok(None);
    };
    if !legacy_truthy(value) {
        return Ok(None);
    }
    if let Some(key) = value.as_str() {
        return Ok(Some(key.to_string()));
    }
    Err(PlanningError::new(format!(
        "unsupported operand type(s) for /: 'PosixPath' and '{}'",
        legacy_python_type(value)
    )))
}

fn legacy_truthy(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(value) => *value,
        Value::Number(value) => value.as_f64().is_some_and(|number| number != 0.0),
        Value::String(value) => !value.is_empty(),
        Value::Array(value) => !value.is_empty(),
        Value::Object(value) => !value.is_empty(),
    }
}

fn legacy_python_type(value: &Value) -> &'static str {
    match value {
        Value::Null => "NoneType",
        Value::Bool(_) => "bool",
        Value::Number(number) if number.is_i64() || number.is_u64() => "int",
        Value::Number(_) => "float",
        Value::String(_) => "str",
        Value::Array(_) => "list",
        Value::Object(_) => "dict",
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
        include_str!("../../../../fixtures/batch_cli_planning_and_index_core.jsonl");

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
    fn batch_cli_planning_fixtures_match() {
        for case in load_cases() {
            let actual = match case["operation"].as_str().unwrap() {
                "batch_iter" => run_batch_iter(&case),
                "normalize_method" => run_normalize_method(&case),
                "slice_bounds" => run_slice_bounds(&case),
                "scan" => run_scan(&case),
                "source_identity" => run_source_identity(&case),
                "source_index" => run_source_index(&case),
                "existing_outputs" => run_existing_outputs(&case),
                "batch_plan" => run_batch_plan(&case),
                other => panic!("unknown operation {other:?}"),
            };
            assert_subset(&actual, &case["expect"]);
        }
    }

    fn run_batch_iter(case: &Value) -> Value {
        let items = string_vec(&case["items"]);
        match batch_iter(&items, case["batch_size"].as_i64().unwrap()) {
            Ok(batches) => json!({
                "status": "ok",
                "batches": batches,
            }),
            Err(error) => json!({
                "status": "error",
                "error": error.message,
            }),
        }
    }

    fn run_normalize_method(case: &Value) -> Value {
        let method = case.get("method").and_then(Value::as_str);
        match normalize_slicing_method(method) {
            Ok(method) => json!({
                "status": "ok",
                "method": method,
            }),
            Err(error) => json!({
                "status": "error",
                "error": error.message,
            }),
        }
    }

    fn run_slice_bounds(case: &Value) -> Value {
        let min_seconds = case.get("min_seconds").and_then(Value::as_f64);
        let max_seconds = case.get("max_seconds").and_then(Value::as_f64);
        match resolve_slice_bounds(min_seconds, max_seconds) {
            Ok(bounds) => json!({
                "status": "ok",
                "bounds": bounds.map(|(min_seconds, max_seconds)| vec![min_seconds, max_seconds]),
            }),
            Err(error) => json!({
                "status": "error",
                "error": error.message,
            }),
        }
    }

    fn run_scan(case: &Value) -> Value {
        let entries = entries_from_fixture(&case["entries"]);
        json!({
            "files": collect_audio_files(
                &entries,
                case["input_dir"].as_str().unwrap(),
                case["recursive"].as_bool().unwrap(),
            ),
        })
    }

    fn run_source_identity(case: &Value) -> Value {
        let audio_path = case["audio_path"].as_str().unwrap();
        let md5 = file_md5_bytes(case["content"].as_str().unwrap_or("").as_bytes());
        let source_md5 = case
            .get("provided_md5")
            .and_then(Value::as_str)
            .unwrap_or(&md5);
        json!({
            "safe_stem": safe_stem(audio_path),
            "md5": md5,
            "source_key": source_key(audio_path, source_md5),
        })
    }

    fn run_source_index(case: &Value) -> Value {
        let mut index = if let Some(content) = case.get("initial_content").and_then(Value::as_str) {
            load_source_index_from_content(Some(content))
        } else {
            case.get("initial_index")
                .and_then(Value::as_object)
                .cloned()
                .unwrap_or_default()
        };
        let mut actual = json!({
            "source_index_path": strip_prefix_path(&source_index_path("output"), "output").unwrap(),
            "loaded": Value::Object(index.clone()),
        });
        if let Some(update) = case.get("update") {
            let audio_path = update["audio_path"].as_str().unwrap();
            let output_key = update["output_key"].as_str().unwrap();
            let source_md5 = update["source_md5"].as_str().unwrap();
            let chunks = update["chunks"].as_u64().unwrap() as usize;
            let labs = update["labs"].as_u64().unwrap() as usize;
            update_source_index(
                &mut index,
                audio_path,
                &format!("__case__/{audio_path}"),
                output_key,
                source_md5,
                chunks,
                labs,
            );
            actual["updated"] = Value::Object(index.clone());
            actual["saved_json"] = Value::String(render_source_index_json(&index));
        }
        actual
    }

    fn run_existing_outputs(case: &Value) -> Value {
        let entries = entries_from_fixture(case.get("entries").unwrap_or(&Value::Array(vec![])));
        let index = case
            .get("source_index")
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default();
        let source_md5 = case["source_md5"].as_str().unwrap();
        let mut actual = json!({
            "existing": has_existing_outputs(
                case["audio_path"].as_str().unwrap(),
                &entries,
                "output",
                source_md5,
                case["recursive_output"].as_bool().unwrap(),
            ),
        });
        match index_has_completed_output(&index, source_md5, &entries, "output") {
            Ok(indexed_key) => {
                actual["indexed_key"] = indexed_key.map(Value::String).unwrap_or(Value::Null);
                actual["indexed_status"] = Value::String("ok".to_string());
            }
            Err(error) => {
                actual["indexed_status"] = Value::String("error".to_string());
                actual["indexed_error"] = Value::String(error.message);
            }
        }
        actual
    }

    fn run_batch_plan(case: &Value) -> Value {
        let source_entries = entries_from_fixture(&case["source_entries"]);
        let output_entries = entries_from_fixture(&case["output_entries"]);
        let source_index = case
            .get("source_index")
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default();
        let options = BatchPlanOptions {
            input_dir: case["input_dir"].as_str().unwrap().to_string(),
            output_dir: case["output_dir"].as_str().unwrap().to_string(),
            recursive: case["recursive"].as_bool().unwrap(),
            file_batch_size: case["file_batch_size"].as_i64().unwrap(),
            no_skip_existing: case["no_skip_existing"].as_bool().unwrap(),
            md5_errors: string_set(case.get("md5_errors")),
            process_outcomes: process_outcomes_from_fixture(case.get("process_outcomes")),
        };
        let result = plan_batch_loop(&source_entries, &output_entries, source_index, &options)
            .unwrap_or_else(|error| panic!("{}", error.message));
        json!({
            "audio_files": result.audio_files,
            "batches": result.batches,
            "total_chunks": result.total_chunks,
            "total_labs": result.total_labs,
            "skipped_existing": result.skipped_existing,
            "skipped_failed": result.skipped_failed,
            "processed": result.processed.iter().map(|record| {
                json!({
                    "path": record.path,
                    "output_key": record.output_key,
                    "chunks": record.chunks,
                    "labs": record.labs,
                })
            }).collect::<Vec<_>>(),
            "skipped": result.skipped.iter().map(|record| {
                let mut value = json!({
                    "path": record.path,
                    "reason": record.reason,
                });
                if let Some(output_key) = &record.output_key {
                    value["output_key"] = Value::String(output_key.clone());
                }
                value
            }).collect::<Vec<_>>(),
            "source_index": Value::Object(result.source_index),
        })
    }

    fn entries_from_fixture(value: &Value) -> Vec<FsEntry> {
        value
            .as_array()
            .unwrap()
            .iter()
            .map(|entry| FsEntry {
                path: entry["path"].as_str().unwrap().to_string(),
                kind: if entry.get("kind").and_then(Value::as_str) == Some("dir") {
                    FsEntryKind::Dir
                } else {
                    FsEntryKind::File
                },
                content: entry
                    .get("content")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string(),
            })
            .collect()
    }

    fn string_vec(value: &Value) -> Vec<String> {
        value
            .as_array()
            .unwrap()
            .iter()
            .map(|item| item.as_str().unwrap().to_string())
            .collect()
    }

    fn string_set(value: Option<&Value>) -> BTreeSet<String> {
        value
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .map(|item| item.as_str().unwrap().to_string())
                    .collect()
            })
            .unwrap_or_default()
    }

    fn process_outcomes_from_fixture(value: Option<&Value>) -> BTreeMap<String, ProcessOutcome> {
        value
            .and_then(Value::as_object)
            .map(|object| {
                object
                    .iter()
                    .map(|(path, outcome)| {
                        (
                            path.clone(),
                            ProcessOutcome {
                                chunks: outcome.get("chunks").and_then(Value::as_u64).unwrap_or(0)
                                    as usize,
                                labs: outcome.get("labs").and_then(Value::as_u64).unwrap_or(0)
                                    as usize,
                                error: outcome
                                    .get("error")
                                    .and_then(Value::as_str)
                                    .map(str::to_string),
                            },
                        )
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
}
