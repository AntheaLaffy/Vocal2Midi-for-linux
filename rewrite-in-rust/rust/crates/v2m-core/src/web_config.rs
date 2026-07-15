//! Web frontend configuration mapping.
//!
//! This module mirrors `web_task_manager.py::TaskManager._build_config` for
//! supported JSON-compatible frontend values while legacy Python remains the
//! runtime owner.

use std::path::{Path, PathBuf};

use crate::{
    device::normalize_runtime_device,
    slice_bounds::{DEFAULT_SLICE_MAX_SEC, DEFAULT_SLICE_MIN_SEC, validate_slice_bounds},
};

/// Supported Web frontend configuration values.
#[derive(Debug, Clone, PartialEq)]
pub struct WebFrontendConfig {
    pub slicing_method: Option<String>,
    pub language: Option<String>,
    pub device: Option<String>,
    pub tempo: Option<f64>,
    pub save_dir: Option<PathBuf>,
    pub lyric_output_mode: Option<String>,
    pub enable_lyrics_match: Option<bool>,
    pub output_lyrics: Option<bool>,
    pub lyrics: Option<String>,
    pub export_ustx: Option<bool>,
    pub output_pitch_curve: Option<bool>,
    pub debug_txt: Option<bool>,
    pub debug_csv: Option<bool>,
    pub debug_chunks: Option<bool>,
    pub pitch_format: Option<String>,
    pub round_pitch: Option<bool>,
    pub seg_threshold: Option<f64>,
    pub seg_radius: Option<f64>,
    pub est_threshold: Option<f64>,
    pub t0: Option<f64>,
    pub nsteps: Option<i64>,
    pub game_batch: Option<i64>,
    pub asr_batch: Option<i64>,
    pub slice_min: Option<f64>,
    pub slice_max: Option<f64>,
    pub game_model_path: Option<String>,
    pub hfa_model_path: Option<String>,
    pub asr_model_path: Option<String>,
    pub rmvpe_model_path: Option<String>,
    pub phoneme_asr_model_path: Option<String>,
}

impl WebFrontendConfig {
    /// Returns an empty Web config, matching Python's missing-key behavior.
    pub fn empty() -> Self {
        Self {
            slicing_method: None,
            language: None,
            device: None,
            tempo: None,
            save_dir: None,
            lyric_output_mode: None,
            enable_lyrics_match: None,
            output_lyrics: None,
            lyrics: None,
            export_ustx: None,
            output_pitch_curve: None,
            debug_txt: None,
            debug_csv: None,
            debug_chunks: None,
            pitch_format: None,
            round_pitch: None,
            seg_threshold: None,
            seg_radius: None,
            est_threshold: None,
            t0: None,
            nsteps: None,
            game_batch: None,
            asr_batch: None,
            slice_min: None,
            slice_max: None,
            game_model_path: None,
            hfa_model_path: None,
            asr_model_path: None,
            rmvpe_model_path: None,
            phoneme_asr_model_path: None,
        }
    }
}

impl Default for WebFrontendConfig {
    fn default() -> Self {
        Self::empty()
    }
}

/// Rust model of the Web-built `PipelineConfig` fields.
#[derive(Debug, Clone, PartialEq)]
pub struct WebPipelineConfig {
    pub audio_path: String,
    pub output_filename: String,
    pub output_dir: PathBuf,
    pub game_model_dir: String,
    pub hfa_model_dir: String,
    pub asr_model_path: String,
    pub device: String,
    pub language: String,
    pub ts: Vec<f64>,
    pub lyric_output_mode: String,
    pub original_lyrics: String,
    pub output_formats: Vec<String>,
    pub slicing_method: String,
    pub slice_min_sec: f64,
    pub slice_max_sec: f64,
    pub tempo: f64,
    pub quantization_step: i64,
    pub quantization_mode: String,
    pub quantization_backend: String,
    pub quantization_bridge_bin: String,
    pub quantization_timeout_sec: f64,
    pub pitch_format: String,
    pub round_pitch: bool,
    pub seg_threshold: f64,
    pub seg_radius: f64,
    pub est_threshold: f64,
    pub batch_size: i64,
    pub asr_batch_size: i64,
    pub output_lyrics: bool,
    pub rmvpe_model_path: String,
    pub phoneme_asr_model_path: String,
    pub output_pitch_curve: bool,
    pub debug_mode: bool,
}

/// Failure while preparing the Web pipeline config.
#[derive(Debug)]
pub enum WebConfigBuildError {
    CreateOutputDir {
        path: PathBuf,
        source: std::io::Error,
    },
}

impl WebConfigBuildError {
    /// Stable error kind used by parity fixtures.
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::CreateOutputDir { .. } => "create_output_dir",
        }
    }
}

impl std::fmt::Display for WebConfigBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CreateOutputDir { path, source } => {
                write!(
                    f,
                    "failed to create output directory {}: {source}",
                    path.display()
                )
            }
        }
    }
}

impl std::error::Error for WebConfigBuildError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::CreateOutputDir { source, .. } => Some(source),
        }
    }
}

/// Builds the Web pipeline config mapping and creates the output directory.
///
/// # Errors
///
/// Returns an error when the output directory cannot be created.
pub fn build_web_pipeline_config(
    frontend_config: &WebFrontendConfig,
    audio_path: &str,
) -> Result<WebPipelineConfig, WebConfigBuildError> {
    let slicing_method = string_or(frontend_config.slicing_method.as_deref(), "auto");
    let language = string_or(frontend_config.language.as_deref(), "zh");
    let device_raw = frontend_config.device.as_deref().unwrap_or("cpu");
    let device = normalize_runtime_device(Some(device_raw));
    let tempo = frontend_config.tempo.unwrap_or(120.0);
    let output_dir = frontend_config
        .save_dir
        .clone()
        .unwrap_or_else(|| PathBuf::from("./output"));

    std::fs::create_dir_all(&output_dir).map_err(|source| {
        WebConfigBuildError::CreateOutputDir {
            path: output_dir.clone(),
            source,
        }
    })?;

    let lyric_output_mode = string_or(frontend_config.lyric_output_mode.as_deref(), "auto");
    let enable_lyrics_match = frontend_config.enable_lyrics_match.unwrap_or(false);
    let output_lyrics = frontend_config.output_lyrics.unwrap_or(true);
    let lyrics = string_or(frontend_config.lyrics.as_deref(), "");
    let original_lyrics = if enable_lyrics_match {
        lyrics
    } else {
        String::new()
    };

    let export_ustx = frontend_config.export_ustx.unwrap_or(false);
    let output_pitch_curve = if export_ustx {
        frontend_config.output_pitch_curve.unwrap_or(false)
    } else {
        false
    };

    let debug_txt = frontend_config.debug_txt.unwrap_or(false);
    let debug_csv = frontend_config.debug_csv.unwrap_or(false);
    let debug_chunks = frontend_config.debug_chunks.unwrap_or(false);
    let pitch_format = string_or(frontend_config.pitch_format.as_deref(), "name");
    let round_pitch = frontend_config.round_pitch.unwrap_or(true);

    let seg_threshold = frontend_config.seg_threshold.unwrap_or(0.2);
    let seg_radius = frontend_config.seg_radius.unwrap_or(0.02);
    let est_threshold = frontend_config.est_threshold.unwrap_or(0.2);
    let t0 = frontend_config.t0.unwrap_or(0.0);
    let nsteps = frontend_config.nsteps.unwrap_or(8);
    let batch_size = frontend_config.game_batch.unwrap_or(1);
    let asr_batch_size = frontend_config.asr_batch.unwrap_or(2);

    let mut slice_min = frontend_config.slice_min.unwrap_or(DEFAULT_SLICE_MIN_SEC);
    let mut slice_max = frontend_config.slice_max.unwrap_or(DEFAULT_SLICE_MAX_SEC);
    if validate_slice_bounds(slice_min, slice_max).is_err() {
        slice_min = DEFAULT_SLICE_MIN_SEC;
        slice_max = DEFAULT_SLICE_MAX_SEC;
    }

    let mut output_formats = vec!["mid".to_string()];
    if debug_txt {
        output_formats.push("txt".to_string());
    }
    if debug_csv {
        output_formats.push("csv".to_string());
    }
    if debug_chunks {
        output_formats.push("chunks".to_string());
    }
    if export_ustx {
        output_formats.push("ustx".to_string());
    }

    let ts = if nsteps > 0 {
        (0..nsteps)
            .map(|i| t0 + (i as f64) * (1.0 - t0) / (nsteps as f64))
            .collect()
    } else {
        Vec::new()
    };

    Ok(WebPipelineConfig {
        audio_path: audio_path.to_string(),
        output_filename: output_stem(audio_path),
        output_dir,
        game_model_dir: string_or(frontend_config.game_model_path.as_deref(), ""),
        hfa_model_dir: string_or(frontend_config.hfa_model_path.as_deref(), ""),
        asr_model_path: string_or(frontend_config.asr_model_path.as_deref(), ""),
        device,
        language,
        ts,
        lyric_output_mode,
        original_lyrics,
        output_formats,
        slicing_method,
        slice_min_sec: slice_min,
        slice_max_sec: slice_max,
        tempo,
        quantization_step: 16,
        quantization_mode: "bayes".to_string(),
        quantization_backend: String::new(),
        quantization_bridge_bin: String::new(),
        quantization_timeout_sec: 30.0,
        pitch_format,
        round_pitch,
        seg_threshold,
        seg_radius,
        est_threshold,
        batch_size,
        asr_batch_size,
        output_lyrics,
        rmvpe_model_path: string_or(frontend_config.rmvpe_model_path.as_deref(), ""),
        phoneme_asr_model_path: string_or(frontend_config.phoneme_asr_model_path.as_deref(), ""),
        output_pitch_curve,
        debug_mode: false,
    })
}

fn string_or(value: Option<&str>, default: &str) -> String {
    value.unwrap_or(default).to_string()
}

fn output_stem(audio_path: &str) -> String {
    Path::new(audio_path)
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use std::{fs, path::PathBuf};

    const FIXTURES: &str = include_str!("../../../../fixtures/web_pipeline_config_mapping.jsonl");

    fn case_dir(case_id: &str) -> PathBuf {
        let path =
            std::env::temp_dir().join(format!("v2m-web-config-{}-{case_id}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        path
    }

    fn replace_case_placeholder(value: &str, case_dir: &Path) -> String {
        value.replace("__case__", &case_dir.to_string_lossy())
    }

    fn config_from_json(value: &Value, case_dir: &Path) -> WebFrontendConfig {
        let mut config = WebFrontendConfig::empty();
        let object = value.as_object().unwrap();

        config.slicing_method = string_field(object, "slicing_method", case_dir);
        config.language = string_field(object, "language", case_dir);
        config.device = string_field(object, "device", case_dir);
        config.tempo = f64_field(object, "tempo");
        config.save_dir = string_field(object, "save_dir", case_dir).map(PathBuf::from);
        config.lyric_output_mode = string_field(object, "lyric_output_mode", case_dir);
        config.enable_lyrics_match = bool_field(object, "enable_lyrics_match");
        config.output_lyrics = bool_field(object, "output_lyrics");
        config.lyrics = string_field(object, "lyrics", case_dir);
        config.export_ustx = bool_field(object, "export_ustx");
        config.output_pitch_curve = bool_field(object, "output_pitch_curve");
        config.debug_txt = bool_field(object, "debug_txt");
        config.debug_csv = bool_field(object, "debug_csv");
        config.debug_chunks = bool_field(object, "debug_chunks");
        config.pitch_format = string_field(object, "pitch_format", case_dir);
        config.round_pitch = bool_field(object, "round_pitch");
        config.seg_threshold = f64_field(object, "seg_threshold");
        config.seg_radius = f64_field(object, "seg_radius");
        config.est_threshold = f64_field(object, "est_threshold");
        config.t0 = f64_field(object, "t0");
        config.nsteps = i64_field(object, "nsteps");
        config.game_batch = i64_field(object, "game_batch");
        config.asr_batch = i64_field(object, "asr_batch");
        config.slice_min = f64_field(object, "slice_min");
        config.slice_max = f64_field(object, "slice_max");
        config.game_model_path = string_field(object, "game_model_path", case_dir);
        config.hfa_model_path = string_field(object, "hfa_model_path", case_dir);
        config.asr_model_path = string_field(object, "asr_model_path", case_dir);
        config.rmvpe_model_path = string_field(object, "rmvpe_model_path", case_dir);
        config.phoneme_asr_model_path = string_field(object, "phoneme_asr_model_path", case_dir);

        config
    }

    fn setup_files(fixture: &Value, case_dir: &Path) {
        let Some(files) = fixture.get("setup_files").and_then(Value::as_array) else {
            return;
        };
        for file in files {
            let path = PathBuf::from(replace_case_placeholder(file.as_str().unwrap(), case_dir));
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::write(path, "fixture").unwrap();
        }
    }

    fn string_field(
        object: &serde_json::Map<String, Value>,
        key: &str,
        case_dir: &Path,
    ) -> Option<String> {
        object
            .get(key)
            .and_then(Value::as_str)
            .map(|value| replace_case_placeholder(value, case_dir))
    }

    fn bool_field(object: &serde_json::Map<String, Value>, key: &str) -> Option<bool> {
        object.get(key).and_then(Value::as_bool)
    }

    fn f64_field(object: &serde_json::Map<String, Value>, key: &str) -> Option<f64> {
        object.get(key).and_then(Value::as_f64)
    }

    fn i64_field(object: &serde_json::Map<String, Value>, key: &str) -> Option<i64> {
        object.get(key).and_then(Value::as_i64)
    }

    fn expected_string(expected: &Value, key: &str, case_dir: &Path) -> String {
        replace_case_placeholder(expected.get(key).unwrap().as_str().unwrap(), case_dir)
    }

    fn expected_bool(expected: &Value, key: &str) -> bool {
        expected.get(key).unwrap().as_bool().unwrap()
    }

    fn expected_i64(expected: &Value, key: &str) -> i64 {
        expected.get(key).unwrap().as_i64().unwrap()
    }

    fn expected_f64(expected: &Value, key: &str) -> f64 {
        expected.get(key).unwrap().as_f64().unwrap()
    }

    fn expected_string_vec(expected: &Value, key: &str) -> Vec<String> {
        expected
            .get(key)
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_str().unwrap().to_string())
            .collect()
    }

    fn expected_f64_vec(expected: &Value, key: &str) -> Vec<f64> {
        expected
            .get(key)
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_f64().unwrap())
            .collect()
    }

    fn assert_float_eq(actual: f64, expected: f64, key: &str, case_id: &str) {
        assert!(
            (actual - expected).abs() <= 1e-9,
            "{case_id}: {key} {actual:?} != {expected:?}"
        );
    }

    fn assert_float_vec_eq(actual: &[f64], expected: &[f64], key: &str, case_id: &str) {
        assert_eq!(actual.len(), expected.len(), "{case_id}: {key} length");
        for (index, (actual_item, expected_item)) in actual.iter().zip(expected).enumerate() {
            assert_float_eq(
                *actual_item,
                *expected_item,
                &format!("{key}[{index}]"),
                case_id,
            );
        }
    }

    #[test]
    fn web_config_mapping_follows_parity_fixture_table() {
        for (line_index, line) in FIXTURES.lines().enumerate() {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let fixture: Value = serde_json::from_str(line).unwrap();
            let case_id = fixture["case_id"].as_str().unwrap();
            let case_dir = case_dir(case_id);
            setup_files(&fixture, &case_dir);
            let frontend_config = config_from_json(&fixture["config"], &case_dir);
            let audio_path =
                replace_case_placeholder(fixture["audio_path"].as_str().unwrap(), &case_dir);
            let actual_result = build_web_pipeline_config(&frontend_config, &audio_path);
            if let Some(expected_error) = fixture.get("expected_error") {
                let error = match actual_result {
                    Ok(_) => panic!("fixture line {} passed unexpectedly", line_index + 1),
                    Err(error) => error,
                };
                assert_eq!(
                    error.kind(),
                    expected_error["rust_kind"].as_str().unwrap(),
                    "fixture line {}",
                    line_index + 1
                );
                let _ = fs::remove_dir_all(&case_dir);
                continue;
            }
            let expected = &fixture["expected"];
            let actual = actual_result
                .unwrap_or_else(|error| panic!("fixture line {}: {error}", line_index + 1));

            assert_eq!(
                actual.audio_path,
                expected_string(expected, "audio_path", &case_dir)
            );
            assert_eq!(
                actual.output_filename,
                expected_string(expected, "output_filename", &case_dir)
            );
            assert_eq!(
                actual.output_dir,
                PathBuf::from(expected_string(expected, "output_dir", &case_dir))
            );
            assert!(actual.output_dir.exists());
            assert_eq!(
                actual.game_model_dir,
                expected_string(expected, "game_model_dir", &case_dir)
            );
            assert_eq!(
                actual.hfa_model_dir,
                expected_string(expected, "hfa_model_dir", &case_dir)
            );
            assert_eq!(
                actual.asr_model_path,
                expected_string(expected, "asr_model_path", &case_dir)
            );
            assert_eq!(
                actual.device,
                expected_string(expected, "device", &case_dir)
            );
            assert_eq!(
                actual.language,
                expected_string(expected, "language", &case_dir)
            );
            assert_float_vec_eq(&actual.ts, &expected_f64_vec(expected, "ts"), "ts", case_id);
            assert_eq!(
                actual.lyric_output_mode,
                expected_string(expected, "lyric_output_mode", &case_dir)
            );
            assert_eq!(
                actual.original_lyrics,
                expected_string(expected, "original_lyrics", &case_dir)
            );
            assert_eq!(
                actual.output_formats,
                expected_string_vec(expected, "output_formats")
            );
            assert_eq!(
                actual.slicing_method,
                expected_string(expected, "slicing_method", &case_dir)
            );
            assert_float_eq(
                actual.slice_min_sec,
                expected_f64(expected, "slice_min_sec"),
                "slice_min_sec",
                case_id,
            );
            assert_float_eq(
                actual.slice_max_sec,
                expected_f64(expected, "slice_max_sec"),
                "slice_max_sec",
                case_id,
            );
            assert_float_eq(
                actual.tempo,
                expected_f64(expected, "tempo"),
                "tempo",
                case_id,
            );
            assert_eq!(
                actual.quantization_step,
                expected_i64(expected, "quantization_step")
            );
            assert_eq!(
                actual.quantization_mode,
                expected_string(expected, "quantization_mode", &case_dir)
            );
            assert_eq!(
                actual.quantization_backend,
                expected_string(expected, "quantization_backend", &case_dir)
            );
            assert_eq!(
                actual.quantization_bridge_bin,
                expected_string(expected, "quantization_bridge_bin", &case_dir)
            );
            assert_float_eq(
                actual.quantization_timeout_sec,
                expected_f64(expected, "quantization_timeout_sec"),
                "quantization_timeout_sec",
                case_id,
            );
            assert_eq!(
                actual.pitch_format,
                expected_string(expected, "pitch_format", &case_dir)
            );
            assert_eq!(actual.round_pitch, expected_bool(expected, "round_pitch"));
            assert_float_eq(
                actual.seg_threshold,
                expected_f64(expected, "seg_threshold"),
                "seg_threshold",
                case_id,
            );
            assert_float_eq(
                actual.seg_radius,
                expected_f64(expected, "seg_radius"),
                "seg_radius",
                case_id,
            );
            assert_float_eq(
                actual.est_threshold,
                expected_f64(expected, "est_threshold"),
                "est_threshold",
                case_id,
            );
            assert_eq!(actual.batch_size, expected_i64(expected, "batch_size"));
            assert_eq!(
                actual.asr_batch_size,
                expected_i64(expected, "asr_batch_size")
            );
            assert_eq!(
                actual.output_lyrics,
                expected_bool(expected, "output_lyrics")
            );
            assert_eq!(
                actual.rmvpe_model_path,
                expected_string(expected, "rmvpe_model_path", &case_dir)
            );
            assert_eq!(
                actual.phoneme_asr_model_path,
                expected_string(expected, "phoneme_asr_model_path", &case_dir)
            );
            assert_eq!(
                actual.output_pitch_curve,
                expected_bool(expected, "output_pitch_curve")
            );
            let _ = fs::remove_dir_all(&case_dir);
        }
    }

    #[test]
    fn output_pitch_curve_requires_ustx_export() {
        let case_dir = case_dir("pitch-curve-gate");
        let config = WebFrontendConfig {
            save_dir: Some(case_dir.join("out")),
            export_ustx: Some(false),
            output_pitch_curve: Some(true),
            ..WebFrontendConfig::empty()
        };
        let actual = build_web_pipeline_config(&config, "input.wav").unwrap();
        assert!(!actual.output_pitch_curve);
        let _ = fs::remove_dir_all(&case_dir);
    }
}
