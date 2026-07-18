//! HubertFA config validation compatibility helpers.
//!
//! This module mirrors the deterministic `check_configs` behavior in
//! `inference/HubertFA/tools/config_utils.py` after a vocab loader outcome has
//! been supplied. Python remains the runtime owner for `load_yaml`, PyYAML
//! SafeLoader behavior, ONNX/model setup, caller routing, and all production
//! config loading.

use std::error::Error;
use std::fmt;
use std::path::{Path, PathBuf};

/// JSON-compatible value domain consumed by the first config validation seam.
///
/// This intentionally excludes PyYAML-specific values such as dates, bytes,
/// sets, ordered pairs, arbitrary mapping keys, aliases, and tagged nodes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HfaConfigValue {
    /// Represents the Python-compatible null case.
    Null,
    /// Represents the Python-compatible bool case.
    Bool,
    /// Represents the Python-compatible int case.
    Int,
    /// Represents the Python-compatible float case.
    Float,
    /// Carries the Python-compatible string value.
    String(String),
    /// Carries the Python-compatible list value.
    List(Vec<HfaConfigValue>),
    /// Carries the Python-compatible mapping value.
    Mapping(Vec<(String, HfaConfigValue)>),
}

impl HfaConfigValue {
    fn python_type_name(&self) -> &'static str {
        match self {
            Self::Null => "NoneType",
            Self::Bool => "bool",
            Self::Int => "int",
            Self::Float => "float",
            Self::String(_) => "str",
            Self::List(_) => "list",
            Self::Mapping(_) => "dict",
        }
    }

    fn get_mapping_value(
        &self,
        key: &str,
    ) -> Result<Option<&HfaConfigValue>, HfaConfigValidationError> {
        match self {
            Self::Mapping(items) => Ok(items
                .iter()
                .find_map(|(item_key, value)| (item_key == key).then_some(value))),
            _ => Err(HfaConfigValidationError::attribute_error(format!(
                "'{}' object has no attribute 'get'",
                self.python_type_name()
            ))),
        }
    }

    fn mapping_items(&self) -> Result<&[(String, HfaConfigValue)], HfaConfigValidationError> {
        match self {
            Self::Mapping(items) => Ok(items),
            _ => Err(HfaConfigValidationError::attribute_error(format!(
                "'{}' object has no attribute 'items'",
                self.python_type_name()
            ))),
        }
    }
}

/// Compatibility failure from HubertFA config validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HfaConfigValidationError {
    /// Represents the Python-compatible assertion case.
    Assertion {
        /// The message text.
        message: String,
    },
    /// Represents the Python-compatible attribute case.
    Attribute {
        /// The message text.
        message: String,
    },
    /// Represents the Python-compatible type case.
    Type {
        /// The message text.
        message: String,
    },
    /// Represents the Python-compatible loader case.
    Loader {
        /// The exception type.
        exception_type: String,
        /// The message text.
        message: String,
    },
}

impl HfaConfigValidationError {
    /// Constructs an opaque loader failure propagated by `check_configs`.
    pub fn loader(
        exception_type: impl Into<String>,
        message: impl Into<String>,
    ) -> HfaConfigValidationError {
        Self::Loader {
            exception_type: exception_type.into(),
            message: message.into(),
        }
    }

    fn assertion_error(message: String) -> Self {
        Self::Assertion { message }
    }

    fn attribute_error(message: String) -> Self {
        Self::Attribute { message }
    }

    fn type_error(message: String) -> Self {
        Self::Type { message }
    }

    /// Legacy Python exception type used by fixture and future bridge
    /// projections.
    pub fn exception_type(&self) -> &str {
        match self {
            Self::Assertion { .. } => "AssertionError",
            Self::Attribute { .. } => "AttributeError",
            Self::Type { .. } => "TypeError",
            Self::Loader { exception_type, .. } => exception_type,
        }
    }

    /// Exact legacy compatibility message.
    pub fn message(&self) -> &str {
        match self {
            Self::Assertion { message }
            | Self::Attribute { message }
            | Self::Type { message }
            | Self::Loader { message, .. } => message,
        }
    }
}

impl fmt::Display for HfaConfigValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.message())
    }
}

impl Error for HfaConfigValidationError {}

/// Validates a HubertFA model config folder with an injected vocab loader.
///
/// `suffix_text` is the already-rendered Python f-string suffix value. For the
/// Python default pass `"yaml"`; for an explicit Python `None`, pass `"None"`.
///
/// # Errors
///
/// Returns Python-compatible assertion, attribute, type, or propagated loader
/// errors for the fixture-bound validation contract.
pub fn check_configs_with_loader<P, F>(
    model_dir: P,
    suffix_text: &str,
    mut load_vocab: F,
) -> Result<(), HfaConfigValidationError>
where
    P: AsRef<Path>,
    F: FnMut(&Path) -> Result<HfaConfigValue, HfaConfigValidationError>,
{
    let model_dir = model_dir.as_ref();
    let vocab_file = model_dir.join(format!("vocab.{suffix_text}"));
    if !vocab_file.exists() {
        return Err(HfaConfigValidationError::assertion_error(format!(
            "{} does not exist",
            vocab_file.display()
        )));
    }

    let config_file = model_dir.join(format!("config.{suffix_text}"));
    if !config_file.exists() {
        return Err(HfaConfigValidationError::assertion_error(format!(
            "{} does not exist",
            config_file.display()
        )));
    }

    let vocab = load_vocab(&vocab_file)?;
    let default_dictionaries = HfaConfigValue::List(Vec::new());
    let dictionaries = vocab
        .get_mapping_value("dictionaries")?
        .unwrap_or(&default_dictionaries);

    for (_language, dictionary) in dictionaries.mapping_items()? {
        match dictionary {
            HfaConfigValue::Null => {}
            HfaConfigValue::String(path_text) => {
                let dictionary_path = model_dir.join(path_text);
                if !dictionary_path.exists() {
                    return Err(HfaConfigValidationError::assertion_error(format!(
                        "{} does not exist",
                        python_absolute_path(&dictionary_path).display()
                    )));
                }
            }
            other => {
                return Err(HfaConfigValidationError::type_error(format!(
                    "unsupported operand type(s) for /: 'PosixPath' and '{}'",
                    other.python_type_name()
                )));
            }
        }
    }

    Ok(())
}

fn python_absolute_path(path: &Path) -> PathBuf {
    std::path::absolute(path).unwrap_or_else(|_| {
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join(path)
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};

    const FIXTURES: &str = include_str!("../../../../fixtures/hfa_config_validation_core.jsonl");
    static NEXT_TEMP_DIRECTORY: AtomicU64 = AtomicU64::new(0);

    struct TestDirectory(PathBuf);

    impl TestDirectory {
        fn new() -> Self {
            loop {
                let sequence = NEXT_TEMP_DIRECTORY.fetch_add(1, Ordering::Relaxed);
                let path = std::env::temp_dir().join(format!(
                    "v2m-hfa-config-validation-{}-{sequence}",
                    std::process::id()
                ));
                match fs::create_dir(&path) {
                    Ok(()) => return Self(path),
                    Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                    Err(error) => panic!("failed to create test directory: {error}"),
                }
            }
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn parse_value(value: &Value) -> HfaConfigValue {
        match value {
            Value::Null => HfaConfigValue::Null,
            Value::Bool(_) => HfaConfigValue::Bool,
            Value::Number(number) if number.is_i64() || number.is_u64() => HfaConfigValue::Int,
            Value::Number(_) => HfaConfigValue::Float,
            Value::String(text) => HfaConfigValue::String(text.clone()),
            Value::Array(items) => HfaConfigValue::List(items.iter().map(parse_value).collect()),
            Value::Object(items) => HfaConfigValue::Mapping(
                items
                    .iter()
                    .map(|(key, value)| (key.clone(), parse_value(value)))
                    .collect(),
            ),
        }
    }

    fn suffix_text(case: &Value) -> String {
        match case.get("suffix") {
            None => "yaml".to_string(),
            Some(Value::Null) => "None".to_string(),
            Some(Value::Bool(true)) => "True".to_string(),
            Some(Value::Bool(false)) => "False".to_string(),
            Some(Value::Number(number)) => number.to_string(),
            Some(Value::String(text)) => text.clone(),
            Some(Value::Array(_)) => panic!("array suffix is outside this fixture seam"),
            Some(Value::Object(_)) => panic!("object suffix is outside this fixture seam"),
        }
    }

    fn replace_temp(value: &Value, temp_root: &Path) -> Value {
        match value {
            Value::String(text) => {
                let temp_root = temp_root.to_string_lossy();
                Value::String(text.replace("<TMP>", temp_root.as_ref()))
            }
            Value::Array(items) => Value::Array(
                items
                    .iter()
                    .map(|item| replace_temp(item, temp_root))
                    .collect(),
            ),
            Value::Object(items) => {
                let mut replaced = serde_json::Map::new();
                for (key, item) in items {
                    replaced.insert(key.clone(), replace_temp(item, temp_root));
                }
                Value::Object(replaced)
            }
            _ => value.clone(),
        }
    }

    fn render_path(path_text: &str, temp_root: &Path) -> PathBuf {
        let rendered = path_text.replace("<TMP>", temp_root.to_string_lossy().as_ref());
        let path = PathBuf::from(rendered);
        if path.is_absolute() {
            path
        } else {
            temp_root.join(path)
        }
    }

    fn create_files(case: &Value, temp_root: &Path) {
        let Some(files) = case.get("files").and_then(Value::as_array) else {
            return;
        };

        for file_spec in files {
            let (path_text, kind, content) = match file_spec {
                Value::String(path_text) => (path_text.as_str(), "file", ""),
                Value::Object(spec) => (
                    spec["path"].as_str().unwrap(),
                    spec.get("kind").and_then(Value::as_str).unwrap_or("file"),
                    spec.get("content").and_then(Value::as_str).unwrap_or(""),
                ),
                _ => panic!("unsupported file fixture"),
            };
            let path = render_path(path_text, temp_root);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            match kind {
                "file" => fs::write(path, content).unwrap(),
                "directory" => fs::create_dir_all(path).unwrap(),
                _ => panic!("unsupported file kind: {kind}"),
            }
        }
    }

    fn normalize_path(path: &Path, temp_root: &Path) -> String {
        path.to_string_lossy()
            .replace(temp_root.to_string_lossy().as_ref(), "<TMP>")
    }

    fn normalize_message(message: &str, temp_root: &Path) -> String {
        message.replace(temp_root.to_string_lossy().as_ref(), "<TMP>")
    }

    fn encode_result(result: Result<(), HfaConfigValidationError>, temp_root: &Path) -> Value {
        match result {
            Ok(()) => json!({ "value": null }),
            Err(error) => json!({
                "error": {
                    "type": error.exception_type(),
                    "message": normalize_message(&error.to_string(), temp_root),
                }
            }),
        }
    }

    fn run_case(case: &Value) -> Value {
        let temp = TestDirectory::new();
        create_files(case, temp.path());
        let loader_spec = &case["loader"];
        let suffix = suffix_text(case);
        let repeat = case.get("repeat").and_then(Value::as_u64).unwrap_or(1);
        let mut loader_paths = Vec::new();
        let mut calls = Vec::new();

        for _ in 0..repeat {
            let result = check_configs_with_loader(temp.path(), &suffix, |path| {
                loader_paths.push(normalize_path(path, temp.path()));
                if let Some(error_spec) = loader_spec.get("error") {
                    return Err(HfaConfigValidationError::loader(
                        error_spec["type"].as_str().unwrap(),
                        error_spec["message"].as_str().unwrap(),
                    ));
                }
                Ok(parse_value(&replace_temp(
                    &loader_spec["value"],
                    temp.path(),
                )))
            });
            calls.push(encode_result(result, temp.path()));
        }

        json!({
            "calls": calls,
            "loader_paths": loader_paths,
        })
    }

    #[test]
    fn hfa_config_validation_core_fixture_parity() {
        for line in FIXTURES.lines().filter(|line| !line.is_empty()) {
            let case: Value = serde_json::from_str(line).unwrap();
            let actual = run_case(&case);
            assert_eq!(actual, case["expect"], "{}", case["case_id"]);
        }
    }
}
