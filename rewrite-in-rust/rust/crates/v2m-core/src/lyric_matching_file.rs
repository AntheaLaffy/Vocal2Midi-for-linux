//! Lyric matching file/state contract helpers.
//!
//! This module mirrors deterministic file, state, and JSON behavior from
//! `inference/LyricFA/tools/lyric_matcher.py`. Python remains the runtime owner
//! for language processors, G2P, sequence alignment routing, console display
//! text, model execution, GUI/Web/CLI callers, and production routing.

use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde_json::{Map, Value};

use crate::lyric_sequence::calculate_difference_count;

/// Processed lyric data produced by a language processor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LyricData {
    pub text_list: Vec<String>,
    pub phonetic_list: Vec<String>,
    pub raw_text: String,
}

/// Result from processing one lab file against one lyric reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessResult {
    pub lab_name: String,
    pub matched_text: String,
    pub matched_phonetic: String,
    pub asr_phonetic: Vec<String>,
    pub asr_text: Vec<String>,
    pub reason: String,
}

/// Mutable counters tracked by legacy `LyricMatchingPipeline`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PipelineState {
    pub total_files: usize,
    pub success_count: usize,
    pub diff_count: usize,
    pub no_match_count: usize,
    pub missing_lyrics: Vec<String>,
}

/// Backend used to inject language and alignment behavior into the file
/// contract seam.
pub trait LyricMatcherBackend {
    fn process_lyric_file(&mut self, lyric_path: &Path) -> Result<LyricData, String>;

    fn process_asr_content(&mut self, lab_content: &str) -> (Vec<String>, Vec<String>);

    fn align_lyric_with_asr(
        &mut self,
        asr_phonetic: &[String],
        lyric_text: &[String],
        lyric_phonetic: &[String],
    ) -> (String, String, String);
}

/// File/state portion of `LyricMatchingPipeline`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LyricMatchingFilePipeline {
    pub lyric_folder: PathBuf,
    pub lab_folder: PathBuf,
    pub json_folder: PathBuf,
    pub language: String,
    pub diff_threshold: i64,
    pub state: PipelineState,
}

impl LyricMatchingFilePipeline {
    /// Creates a pipeline state holder for the file-contract seam.
    pub fn new(
        lyric_folder: impl Into<PathBuf>,
        lab_folder: impl Into<PathBuf>,
        json_folder: impl Into<PathBuf>,
        language: impl Into<String>,
        diff_threshold: i64,
    ) -> Self {
        Self {
            lyric_folder: lyric_folder.into(),
            lab_folder: lab_folder.into(),
            json_folder: json_folder.into(),
            language: language.into(),
            diff_threshold,
            state: PipelineState::default(),
        }
    }

    /// Adds a missing lyric name once.
    pub fn add_missing_lyric(&mut self, lyric_name: &str) {
        if !self
            .state
            .missing_lyrics
            .iter()
            .any(|existing| existing == lyric_name)
        {
            self.state.missing_lyrics.push(lyric_name.to_string());
        }
    }

    /// Loads lyric files in caller-supplied order.
    pub fn load_all_lyrics_from_paths<B: LyricMatcherBackend>(
        &mut self,
        lyric_paths: &[PathBuf],
        backend: &mut B,
    ) -> HashMap<String, LyricData> {
        let mut lyric_dict = HashMap::new();
        for lyric_path in lyric_paths {
            let lyric_name = extract_filename_without_extension(&lyric_path.to_string_lossy());
            if let Ok(lyric_data) = backend.process_lyric_file(lyric_path) {
                lyric_dict.insert(lyric_name, lyric_data);
            }
        }
        lyric_dict
    }

    /// Processes one lab file. Missing lyrics, read errors, and empty ASR
    /// phonetics all return `None`, matching the legacy branch shape.
    pub fn process_single_file<B: LyricMatcherBackend>(
        &mut self,
        lab_path: &Path,
        lyric_dict: &HashMap<String, LyricData>,
        backend: &mut B,
    ) -> Option<ProcessResult> {
        let lab_name = extract_filename_without_extension(&lab_path.to_string_lossy());
        let lyric_name = lyric_name_for_lab_name(&lab_name);

        let lyric_data = if let Some(lyric_data) = lyric_dict.get(&lyric_name) {
            lyric_data
        } else {
            self.add_missing_lyric(&lyric_name);
            return None;
        };

        let lab_content = fs::read_to_string(lab_path).ok()?.trim().to_string();
        let (asr_text, asr_phonetic) = backend.process_asr_content(&lab_content);
        if asr_phonetic.is_empty() {
            return None;
        }

        let (matched_text, matched_phonetic, reason) = backend.align_lyric_with_asr(
            &asr_phonetic,
            &lyric_data.text_list,
            &lyric_data.phonetic_list,
        );

        Some(ProcessResult {
            lab_name,
            matched_text,
            matched_phonetic,
            asr_phonetic,
            asr_text,
            reason,
        })
    }

    /// Applies no-match/diff-threshold logic and writes result JSON.
    pub fn compare_and_save_result(&mut self, result: &ProcessResult) -> io::Result<()> {
        if result.matched_text.is_empty() && result.matched_phonetic.is_empty() {
            return self.handle_no_match(result);
        }

        let target_sequence;
        let source_sequence = if self.language == "zh" {
            target_sequence = split_tokens(&result.matched_phonetic);
            &result.asr_phonetic
        } else {
            target_sequence = split_tokens(&result.matched_text);
            &result.asr_text
        };

        let diff_count = calculate_difference_count(source_sequence, &target_sequence) as i64;
        if diff_count > self.diff_threshold {
            self.state.diff_count += 1;
        }

        let json_path = self.json_folder.join(format!("{}.json", result.lab_name));
        save_to_json(&json_path, &result.matched_text, &result.matched_phonetic)?;
        self.state.success_count += 1;
        Ok(())
    }

    fn handle_no_match(&mut self, result: &ProcessResult) -> io::Result<()> {
        self.state.no_match_count += 1;
        let json_path = self.json_folder.join(format!("{}.json", result.lab_name));
        save_to_json(&json_path, "", "")?;
        self.state.success_count += 1;
        Ok(())
    }

    /// Executes a caller-supplied single batch of lyric/lab paths.
    pub fn execute_with_paths<B: LyricMatcherBackend>(
        &mut self,
        lyric_paths: &[PathBuf],
        lab_paths: &[PathBuf],
        backend: &mut B,
    ) -> io::Result<()> {
        fs::create_dir_all(&self.json_folder)?;
        let lyric_dict = self.load_all_lyrics_from_paths(lyric_paths, backend);
        self.state.total_files = lab_paths.len();
        for lab_path in lab_paths {
            if let Some(result) = self.process_single_file(lab_path, &lyric_dict, backend) {
                self.compare_and_save_result(&result)?;
            }
        }
        Ok(())
    }
}

/// Extracts the basename without the final extension using Python-like
/// `os.path.basename` plus `os.path.splitext` behavior for dotfiles.
pub fn extract_filename_without_extension(file_path: &str) -> String {
    let basename = file_path.rsplit('/').next().unwrap_or(file_path);
    splitext_root(basename).to_string()
}

/// Maps a lab stem to its lyric stem by removing the last underscore suffix.
pub fn lyric_name_for_lab_name(lab_name: &str) -> String {
    lab_name
        .rsplit_once('_')
        .map(|(left, _)| left)
        .unwrap_or(lab_name)
        .to_string()
}

/// Builds the stable JSON payload used by `LyricMatcher.save_to_json`.
pub fn json_payload(text: &str, phonetic: &str) -> Value {
    let mut map = Map::new();
    map.insert("raw_text".to_string(), Value::String(text.to_string()));
    map.insert("lab".to_string(), Value::String(phonetic.to_string()));
    map.insert(
        "lab_without_tone".to_string(),
        Value::String(phonetic.to_string()),
    );
    Value::Object(map)
}

/// Serializes the JSON payload with the legacy three-space indentation.
pub fn json_payload_string(text: &str, phonetic: &str) -> serde_json::Result<String> {
    Ok(format!(
        "{{\n   \"raw_text\": {},\n   \"lab\": {},\n   \"lab_without_tone\": {}\n}}",
        serde_json::to_string(text)?,
        serde_json::to_string(phonetic)?,
        serde_json::to_string(phonetic)?,
    ))
}

/// Writes the stable result JSON payload.
pub fn save_to_json(path: &Path, text: &str, phonetic: &str) -> io::Result<()> {
    let payload = json_payload_string(text, phonetic).map_err(io::Error::other)?;
    fs::write(path, payload)
}

fn splitext_root(basename: &str) -> &str {
    let bytes = basename.as_bytes();
    let Some(last_dot) = bytes.iter().rposition(|byte| *byte == b'.') else {
        return basename;
    };
    let leading_dots = bytes.iter().take_while(|byte| **byte == b'.').count();
    if last_dot < leading_dots || last_dot == 0 {
        basename
    } else {
        &basename[..last_dot]
    }
}

fn split_tokens(value: &str) -> Vec<String> {
    value.split_whitespace().map(str::to_string).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};
    use std::process;
    use std::time::{SystemTime, UNIX_EPOCH};

    const FIXTURES: &str =
        include_str!("../../../../fixtures/lyric_matching_file_contract_core.jsonl");

    #[derive(Debug, Clone, Default)]
    struct FakeBackend {
        lyric_data: Option<LyricData>,
        lyric_data_by_name: HashMap<String, LyricData>,
        asr_text: Vec<String>,
        asr_phonetic: Vec<String>,
        matched_text: String,
        matched_phonetic: String,
        reason: String,
    }

    impl LyricMatcherBackend for FakeBackend {
        fn process_lyric_file(&mut self, lyric_path: &Path) -> Result<LyricData, String> {
            if let Some(lyric_data) = &self.lyric_data {
                return Ok(lyric_data.clone());
            }
            let stem = extract_filename_without_extension(&lyric_path.to_string_lossy());
            self.lyric_data_by_name
                .get(&stem)
                .cloned()
                .ok_or_else(|| format!("fake lyric data missing for {}", lyric_path.display()))
        }

        fn process_asr_content(&mut self, _lab_content: &str) -> (Vec<String>, Vec<String>) {
            (self.asr_text.clone(), self.asr_phonetic.clone())
        }

        fn align_lyric_with_asr(
            &mut self,
            _asr_phonetic: &[String],
            _lyric_text: &[String],
            _lyric_phonetic: &[String],
        ) -> (String, String, String) {
            (
                self.matched_text.clone(),
                self.matched_phonetic.clone(),
                self.reason.clone(),
            )
        }
    }

    fn parse_string_vec(value: &Value) -> Vec<String> {
        value
            .as_array()
            .unwrap()
            .iter()
            .map(|item| item.as_str().unwrap().to_string())
            .collect()
    }

    fn parse_lyric_data(value: &Value) -> LyricData {
        LyricData {
            text_list: parse_string_vec(&value["text_list"]),
            phonetic_list: parse_string_vec(&value["phonetic_list"]),
            raw_text: value["raw_text"].as_str().unwrap_or("").to_string(),
        }
    }

    fn parse_lyric_dict(value: &Value) -> HashMap<String, LyricData> {
        value
            .as_object()
            .unwrap_or(&serde_json::Map::new())
            .iter()
            .map(|(key, value)| (key.clone(), parse_lyric_data(value)))
            .collect()
    }

    fn parse_process_result(value: &Value) -> ProcessResult {
        ProcessResult {
            lab_name: value["lab_name"].as_str().unwrap().to_string(),
            matched_text: value["matched_text"].as_str().unwrap().to_string(),
            matched_phonetic: value["matched_phonetic"].as_str().unwrap().to_string(),
            asr_phonetic: parse_string_vec(&value["asr_phonetic"]),
            asr_text: parse_string_vec(&value["asr_text"]),
            reason: value["reason"].as_str().unwrap_or("").to_string(),
        }
    }

    fn parse_fake_backend(value: Option<&Value>) -> FakeBackend {
        let Some(value) = value else {
            return FakeBackend::default();
        };
        let mut backend = FakeBackend {
            asr_text: parse_string_vec(value.get("asr_text").unwrap_or(&json!([]))),
            asr_phonetic: parse_string_vec(value.get("asr_phonetic").unwrap_or(&json!([]))),
            matched_text: value["matched_text"].as_str().unwrap_or("").to_string(),
            matched_phonetic: value["matched_phonetic"].as_str().unwrap_or("").to_string(),
            reason: value["reason"].as_str().unwrap_or("").to_string(),
            ..FakeBackend::default()
        };
        if let Some(lyric_data) = value.get("lyric_data") {
            backend.lyric_data = Some(parse_lyric_data(lyric_data));
        }
        if let Some(items) = value.get("lyric_data_by_name").and_then(Value::as_object) {
            backend.lyric_data_by_name = items
                .iter()
                .map(|(key, value)| (key.clone(), parse_lyric_data(value)))
                .collect();
        }
        backend
    }

    fn encode_state(state: &PipelineState) -> Value {
        json!({
            "total_files": state.total_files,
            "success_count": state.success_count,
            "diff_count": state.diff_count,
            "no_match_count": state.no_match_count,
            "missing_lyrics": state.missing_lyrics,
        })
    }

    fn encode_process_result(result: Option<ProcessResult>) -> Value {
        match result {
            Some(result) => json!({
                "lab_name": result.lab_name,
                "matched_text": result.matched_text,
                "matched_phonetic": result.matched_phonetic,
                "asr_phonetic": result.asr_phonetic,
                "asr_text": result.asr_text,
                "reason": result.reason,
            }),
            None => Value::Null,
        }
    }

    fn assert_json_close(actual: &Value, expected: &Value, context: &str) {
        match (actual, expected) {
            (Value::Array(left), Value::Array(right)) => {
                assert_eq!(left.len(), right.len(), "{context}: array lengths differ");
                for (index, (left_item, right_item)) in left.iter().zip(right).enumerate() {
                    assert_json_close(left_item, right_item, &format!("{context}[{index}]"));
                }
            }
            (Value::Object(left), Value::Object(right)) => {
                assert_eq!(left.len(), right.len(), "{context}: object lengths differ");
                for (key, right_value) in right {
                    let left_value = left
                        .get(key)
                        .unwrap_or_else(|| panic!("{context}: missing {key}"));
                    assert_json_close(left_value, right_value, &format!("{context}.{key}"));
                }
            }
            _ => assert_eq!(actual, expected, "{context}"),
        }
    }

    fn temp_case_dir(case_id: &str, line_index: usize) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "v2m_lyric_matching_file_{}_{}_{}_{}",
            process::id(),
            line_index,
            nanos,
            case_id
        ))
    }

    fn make_pipeline(tmp: &Path, language: &str, diff_threshold: i64) -> LyricMatchingFilePipeline {
        LyricMatchingFilePipeline::new(
            tmp.join("lyrics"),
            tmp.join("labs"),
            tmp.join("json"),
            language.to_string(),
            diff_threshold,
        )
    }

    fn setup_dirs(tmp: &Path) {
        fs::create_dir_all(tmp.join("lyrics")).unwrap();
        fs::create_dir_all(tmp.join("labs")).unwrap();
        fs::create_dir_all(tmp.join("json")).unwrap();
    }

    fn read_json(path: &Path) -> Value {
        serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap()
    }

    #[test]
    fn lyric_matching_file_contract_follows_parity_fixture_table() {
        for (line_index, line) in FIXTURES.lines().enumerate() {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let case: Value = serde_json::from_str(line).unwrap();
            let case_id = case["case_id"].as_str().unwrap();
            let actual = match case["kind"].as_str().unwrap() {
                "extract_filename" => json!({
                    "values": parse_string_vec(&case["paths"])
                        .iter()
                        .map(|path| extract_filename_without_extension(path))
                        .collect::<Vec<_>>(),
                }),
                "lab_to_lyric_name" => {
                    let items = parse_string_vec(&case["paths"])
                        .iter()
                        .map(|path| {
                            let lab_name = extract_filename_without_extension(path);
                            json!({
                                "lab_name": lab_name,
                                "lyric_name": lyric_name_for_lab_name(&lab_name),
                            })
                        })
                        .collect::<Vec<_>>();
                    json!({ "items": items })
                }
                "missing_lyric_dedup" => {
                    let tmp = temp_case_dir(case_id, line_index);
                    setup_dirs(&tmp);
                    let mut pipeline =
                        make_pipeline(&tmp, case["language"].as_str().unwrap_or("en"), 5);
                    let mut backend = FakeBackend::default();
                    let mut results = Vec::new();
                    for relative in parse_string_vec(&case["lab_paths"]) {
                        let lab_path = tmp.join("labs").join(relative);
                        fs::write(&lab_path, "unused").unwrap();
                        results.push(encode_process_result(pipeline.process_single_file(
                            &lab_path,
                            &HashMap::new(),
                            &mut backend,
                        )));
                    }
                    let actual = json!({
                        "results": results,
                        "state": encode_state(&pipeline.state),
                    });
                    fs::remove_dir_all(&tmp).ok();
                    actual
                }
                "process_single_file" => {
                    let tmp = temp_case_dir(case_id, line_index);
                    setup_dirs(&tmp);
                    let mut pipeline =
                        make_pipeline(&tmp, case["language"].as_str().unwrap_or("en"), 5);
                    let mut backend = parse_fake_backend(case.get("fake"));
                    let lab_path = tmp.join("labs").join(case["lab_path"].as_str().unwrap());
                    fs::write(&lab_path, case["lab_content"].as_str().unwrap_or("")).unwrap();
                    let result = pipeline.process_single_file(
                        &lab_path,
                        &parse_lyric_dict(&case["lyric_dict"]),
                        &mut backend,
                    );
                    let actual = json!({
                        "result": encode_process_result(result),
                        "state": encode_state(&pipeline.state),
                    });
                    fs::remove_dir_all(&tmp).ok();
                    actual
                }
                "compare_result" => {
                    let tmp = temp_case_dir(case_id, line_index);
                    setup_dirs(&tmp);
                    let mut pipeline = make_pipeline(
                        &tmp,
                        case["language"].as_str().unwrap_or("en"),
                        case["diff_threshold"].as_i64().unwrap_or(5),
                    );
                    let result = parse_process_result(&case["result"]);
                    pipeline.compare_and_save_result(&result).unwrap();
                    let actual = json!({
                        "state": encode_state(&pipeline.state),
                        "json": read_json(&tmp.join("json").join(format!("{}.json", result.lab_name))),
                    });
                    fs::remove_dir_all(&tmp).ok();
                    actual
                }
                "execute_single" => {
                    let tmp = temp_case_dir(case_id, line_index);
                    setup_dirs(&tmp);
                    let mut pipeline =
                        make_pipeline(&tmp, case["language"].as_str().unwrap_or("en"), 5);
                    let mut backend = parse_fake_backend(case.get("fake"));
                    let mut lyric_paths = Vec::new();
                    for (filename, content) in case["lyric_files"].as_object().unwrap() {
                        let path = tmp.join("lyrics").join(filename);
                        fs::write(&path, content.as_str().unwrap()).unwrap();
                        lyric_paths.push(path);
                    }
                    let mut lab_paths = Vec::new();
                    for (filename, content) in case["lab_files"].as_object().unwrap() {
                        let path = tmp.join("labs").join(filename);
                        fs::write(&path, content.as_str().unwrap()).unwrap();
                        lab_paths.push(path);
                    }
                    pipeline
                        .execute_with_paths(&lyric_paths, &lab_paths, &mut backend)
                        .unwrap();
                    let mut json_files = Map::new();
                    for entry in fs::read_dir(tmp.join("json")).unwrap() {
                        let path = entry.unwrap().path();
                        json_files.insert(
                            path.file_name().unwrap().to_string_lossy().to_string(),
                            read_json(&path),
                        );
                    }
                    let actual = json!({
                        "state": encode_state(&pipeline.state),
                        "json_files": Value::Object(json_files),
                    });
                    fs::remove_dir_all(&tmp).ok();
                    actual
                }
                other => panic!("unknown fixture kind {other}"),
            };

            assert_json_close(
                &actual,
                &case["expect"],
                &format!("{} fixture line {}", case_id, line_index + 1),
            );
        }
    }
}
