//! Qwen ASR language validation and schema DTO contracts.
//!
//! This module mirrors fixture-backed behavior from
//! `inference/qwen3asr_dml/utils.py` and `schema.py`. It is not wired into the
//! Python runtime or any model execution path.

use serde_json::Value;

/// Canonical language names accepted by the legacy Qwen ASR validator.
pub const SUPPORTED_LANGUAGES: &[&str] = &[
    "Chinese",
    "English",
    "Cantonese",
    "Arabic",
    "German",
    "French",
    "Spanish",
    "Portuguese",
    "Indonesian",
    "Italian",
    "Korean",
    "Russian",
    "Thai",
    "Vietnamese",
    "Japanese",
    "Turkish",
    "Hindi",
    "Malay",
    "Dutch",
    "Swedish",
    "Danish",
    "Finnish",
    "Polish",
    "Czech",
    "Filipino",
    "Persian",
    "Greek",
    "Romanian",
    "Hungarian",
    "Macedonian",
];

#[derive(Debug, Clone, PartialEq, Eq)]
/// Error type and message projected from the Python schema helpers.
pub struct SchemaError {
    /// The Python-compatible error type.
    pub error_type: &'static str,
    /// The message text.
    pub message: String,
}

impl SchemaError {
    fn value_error(message: impl Into<String>) -> Self {
        Self {
            error_type: "ValueError",
            message: message.into(),
        }
    }

    fn type_error(message: impl Into<String>) -> Self {
        Self {
            error_type: "TypeError",
            message: message.into(),
        }
    }
}

/// Normalizes an optional language name using the legacy trim and title-case policy.
///
/// # Errors
///
/// Returns [`SchemaError`] when the value is absent or empty after trimming.
pub fn normalize_language_name(value: Option<&str>) -> Result<String, SchemaError> {
    match value {
        Some(value) => normalize_language_display(value),
        None => Err(SchemaError::value_error("language is None")),
    }
}

/// Normalizes a JSON value after projecting it to the Python display form.
///
/// # Errors
///
/// Returns [`SchemaError`] for JSON `null` or a value whose display form is
/// empty after trimming.
pub fn normalize_language_value(value: &Value) -> Result<String, SchemaError> {
    if value.is_null() {
        return Err(SchemaError::value_error("language is None"));
    }
    normalize_language_display(&python_display_value(value))
}

fn normalize_language_display(value: &str) -> Result<String, SchemaError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(SchemaError::value_error("language is empty"));
    }
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return Err(SchemaError::value_error("language is empty"));
    };
    let mut normalized = first.to_uppercase().collect::<String>();
    normalized.push_str(&chars.flat_map(char::to_lowercase).collect::<String>());
    Ok(normalized)
}

/// Validates an exact canonical language name.
///
/// # Errors
///
/// Returns [`SchemaError`] when `language` is not in
/// [`SUPPORTED_LANGUAGES`].
pub fn validate_language(language: &str) -> Result<(), SchemaError> {
    if SUPPORTED_LANGUAGES.contains(&language) {
        Ok(())
    } else {
        Err(SchemaError::value_error(format!(
            "Unsupported language: {language}. Supported: {}",
            supported_languages_python_repr()
        )))
    }
}

/// Validates a JSON value after projecting it to the Python display form.
///
/// # Errors
///
/// Returns [`SchemaError`] when the projected value is not supported.
pub fn validate_language_value(value: &Value) -> Result<(), SchemaError> {
    validate_language(&python_display_value(value))
}

fn supported_languages_python_repr() -> String {
    format!(
        "[{}]",
        SUPPORTED_LANGUAGES
            .iter()
            .map(|language| format!("'{language}'"))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn python_display_value(value: &Value) -> String {
    match value {
        Value::Null => "None".to_string(),
        Value::Bool(true) => "True".to_string(),
        Value::Bool(false) => "False".to_string(),
        Value::Number(number) => number.to_string(),
        Value::String(value) => value.clone(),
        other => other.to_string(),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Numeric message kinds used by the legacy Qwen streaming protocol.
pub enum MsgType {
    /// Represents the Python-compatible cmd encode case.
    CmdEncode,
    /// Represents the Python-compatible cmd stop case.
    CmdStop,
    /// Represents the Python-compatible msg embd case.
    MsgEmbd,
    /// Represents the Python-compatible msg ready case.
    MsgReady,
    /// Represents the Python-compatible msg done case.
    MsgDone,
    /// Represents the Python-compatible msg error case.
    MsgError,
}

impl MsgType {
    /// Returns the Python enum's stable integer value.
    pub const fn value(self) -> i64 {
        match self {
            Self::CmdEncode => 1,
            Self::CmdStop => 2,
            Self::MsgEmbd => 3,
            Self::MsgReady => 4,
            Self::MsgDone => 5,
            Self::MsgError => 6,
        }
    }

    /// Returns the Python enum member name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::CmdEncode => "CMD_ENCODE",
            Self::CmdStop => "CMD_STOP",
            Self::MsgEmbd => "MSG_EMBD",
            Self::MsgReady => "MSG_READY",
            Self::MsgDone => "MSG_DONE",
            Self::MsgError => "MSG_ERROR",
        }
    }

    /// Renders the value using Python's `str(EnumMember)` form.
    pub fn python_str(self) -> String {
        format!("MsgType.{}", self.name())
    }

    /// Renders the value using Python's `repr(EnumMember)` form.
    pub fn python_repr(self) -> String {
        format!("<MsgType.{}: {}>", self.name(), self.value())
    }

    /// Returns all message kinds in numeric order.
    pub const fn all() -> [Self; 6] {
        [
            Self::CmdEncode,
            Self::CmdStop,
            Self::MsgEmbd,
            Self::MsgReady,
            Self::MsgDone,
            Self::MsgError,
        ]
    }
}

#[derive(Debug, Clone, PartialEq)]
/// Data transfer object compatible with `schema.py::StreamingMessage`.
pub struct StreamingMessage {
    /// The msg type.
    pub msg_type: MsgType,
    /// The optional data.
    pub data: Option<Value>,
    /// Whether this is the final streaming message.
    pub is_last: bool,
    /// Encoder time in seconds.
    pub encode_time: f64,
}

impl StreamingMessage {
    /// Creates a message with Python dataclass defaults.
    pub fn new(msg_type: MsgType) -> Self {
        Self {
            msg_type,
            data: None,
            is_last: false,
            encode_time: 0.0,
        }
    }

    /// Renders the Python dataclass representation used by parity fixtures.
    pub fn python_repr(&self) -> String {
        format!(
            "StreamingMessage(msg_type={}, data={}, is_last={}, encode_time={})",
            self.msg_type.python_repr(),
            self.data
                .as_ref()
                .map(python_repr_value)
                .unwrap_or_else(|| "None".to_string()),
            python_bool(self.is_last),
            python_float(self.encode_time)
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
/// Decoder output and performance counters compatible with `DecodeResult`.
pub struct DecodeResult {
    /// The text.
    pub text: String,
    /// The ordered stable tokens.
    pub stable_tokens: Vec<i64>,
    /// The t prefill.
    pub t_prefill: f64,
    /// The t generate.
    pub t_generate: f64,
    /// The n prefill.
    pub n_prefill: i64,
    /// The n generate.
    pub n_generate: i64,
    /// Whether decoding was aborted.
    pub is_aborted: bool,
}

impl Default for DecodeResult {
    fn default() -> Self {
        Self {
            text: String::new(),
            stable_tokens: Vec::new(),
            t_prefill: 0.0,
            t_generate: 0.0,
            n_prefill: 0,
            n_generate: 0,
            is_aborted: false,
        }
    }
}

impl DecodeResult {
    /// Renders the Python dataclass representation used by parity fixtures.
    pub fn python_repr(&self) -> String {
        format!(
            "DecodeResult(text={}, stable_tokens={}, t_prefill={}, t_generate={}, n_prefill={}, n_generate={}, is_aborted={})",
            python_repr_value(&Value::String(self.text.clone())),
            python_repr_value(&Value::Array(
                self.stable_tokens
                    .iter()
                    .copied()
                    .map(Value::from)
                    .collect()
            )),
            python_float(self.t_prefill),
            python_float(self.t_generate),
            self.n_prefill,
            self.n_generate,
            python_bool(self.is_aborted)
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
/// Qwen engine configuration compatible with the Python dataclass defaults.
pub struct ASREngineConfig {
    /// The model directory.
    pub model_dir: String,
    /// The encoder frontend fn.
    pub encoder_frontend_fn: String,
    /// The encoder backend fn.
    pub encoder_backend_fn: String,
    /// The llm fn.
    pub llm_fn: String,
    /// Whether DirectML execution is enabled.
    pub use_dml: bool,
    /// The n ctx.
    pub n_ctx: i64,
    /// The chunk size.
    pub chunk_size: f64,
    /// The memory num.
    pub memory_num: i64,
    /// The max decode tokens.
    pub max_decode_tokens: i64,
    /// The llama backend.
    pub llama_backend: String,
    /// Whether verbose logging is enabled.
    pub verbose: bool,
}

impl ASREngineConfig {
    /// Creates a configuration for `model_dir` with Python dataclass defaults.
    pub fn new(model_dir: impl Into<String>) -> Self {
        Self {
            model_dir: model_dir.into(),
            encoder_frontend_fn: "qwen3_asr_encoder_frontend.fp16.onnx".to_string(),
            encoder_backend_fn: "qwen3_asr_encoder_backend.fp16.onnx".to_string(),
            llm_fn: "qwen3_asr_llm.f16.gguf".to_string(),
            use_dml: true,
            n_ctx: 2048,
            chunk_size: 40.0,
            memory_num: 1,
            max_decode_tokens: 512,
            llama_backend: "auto".to_string(),
            verbose: true,
        }
    }

    /// Renders the Python dataclass representation used by parity fixtures.
    pub fn python_repr(&self) -> String {
        format!(
            "ASREngineConfig(model_dir={}, encoder_frontend_fn={}, encoder_backend_fn={}, llm_fn={}, use_dml={}, n_ctx={}, chunk_size={}, memory_num={}, max_decode_tokens={}, llama_backend={}, verbose={})",
            python_repr_value(&Value::String(self.model_dir.clone())),
            python_repr_value(&Value::String(self.encoder_frontend_fn.clone())),
            python_repr_value(&Value::String(self.encoder_backend_fn.clone())),
            python_repr_value(&Value::String(self.llm_fn.clone())),
            python_bool(self.use_dml),
            self.n_ctx,
            python_float(self.chunk_size),
            self.memory_num,
            self.max_decode_tokens,
            python_repr_value(&Value::String(self.llama_backend.clone())),
            python_bool(self.verbose)
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
/// Transcription text and optional performance payload.
pub struct TranscribeResult {
    /// The text.
    pub text: String,
    /// The optional performance.
    pub performance: Option<Value>,
}

impl TranscribeResult {
    /// Creates a result with no performance payload.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            performance: None,
        }
    }

    /// Renders the Python dataclass representation used by parity fixtures.
    pub fn python_repr(&self) -> String {
        format!(
            "TranscribeResult(text={}, performance={})",
            python_repr_value(&Value::String(self.text.clone())),
            self.performance
                .as_ref()
                .map(python_repr_value)
                .unwrap_or_else(|| "None".to_string())
        )
    }
}

/// Returns the Python missing-argument error for a required dataclass constructor.
///
/// Unknown constructor names return `None`.
pub fn constructor_missing_error(name: &str) -> Option<SchemaError> {
    match name {
        "StreamingMessage" => Some(SchemaError::type_error(
            "StreamingMessage.__init__() missing 1 required positional argument: 'msg_type'",
        )),
        "ASREngineConfig" => Some(SchemaError::type_error(
            "ASREngineConfig.__init__() missing 1 required positional argument: 'model_dir'",
        )),
        "TranscribeResult" => Some(SchemaError::type_error(
            "TranscribeResult.__init__() missing 1 required positional argument: 'text'",
        )),
        _ => None,
    }
}

fn python_bool(value: bool) -> &'static str {
    if value { "True" } else { "False" }
}

fn python_float(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{value:.1}")
    } else {
        value.to_string()
    }
}

fn python_repr_value(value: &Value) -> String {
    match value {
        Value::Null => "None".to_string(),
        Value::Bool(value) => python_bool(*value).to_string(),
        Value::Number(number) => number.to_string(),
        Value::String(value) => format!("'{}'", value.replace('\\', "\\\\").replace('\'', "\\'")),
        Value::Array(items) => format!(
            "[{}]",
            items
                .iter()
                .map(python_repr_value)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Value::Object(object) => format!(
            "{{{}}}",
            object
                .iter()
                .map(|(key, value)| format!(
                    "{}: {}",
                    python_repr_value(&Value::String(key.clone())),
                    python_repr_value(value)
                ))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    const FIXTURES: &str =
        include_str!("../../../../fixtures/asr_qwen_language_schema_contract.jsonl");

    #[test]
    fn asr_qwen_language_schema_contract_fixture_parity() {
        for (index, line) in FIXTURES.lines().enumerate() {
            let case: Value = serde_json::from_str(line).unwrap();
            let kind = case["kind"].as_str().unwrap();
            let category = case["category"].as_str().unwrap();
            let actual = actual_for_case(kind, category, &case);
            let expected = expected_for_case(kind, category, &case);
            assert_eq!(actual, expected, "case {} {kind} {category}", index + 1);
        }
    }

    fn actual_for_case(kind: &str, category: &str, case: &Value) -> Value {
        match kind {
            "normalize_language_name" => result_json(normalize_language_value(&case["input"])),
            "supported_languages" => json!(SUPPORTED_LANGUAGES),
            "validate_language" => result_json(validate_language_value(&case["input"])),
            "msg_types" => json!(
                MsgType::all()
                    .iter()
                    .map(|msg_type| json!({
                        "name": msg_type.name(),
                        "value": msg_type.value(),
                        "str": msg_type.python_str(),
                        "repr": msg_type.python_repr(),
                    }))
                    .collect::<Vec<_>>()
            ),
            "streaming_message" => match category {
                "defaults" => streaming_snapshot(&StreamingMessage::new(MsgType::CmdEncode)),
                "custom" => streaming_snapshot(&StreamingMessage {
                    msg_type: MsgType::MsgDone,
                    data: Some(json!({"k": [1, 2]})),
                    is_last: true,
                    encode_time: 1.25,
                }),
                _ => panic!("unknown streaming category {category}"),
            },
            "decode_result" => match category {
                "defaults" => decode_result_payload(&DecodeResult::default()),
                "custom" => decode_result_payload(&DecodeResult {
                    text: "abc".to_string(),
                    stable_tokens: vec![1, 2],
                    t_prefill: 0.5,
                    t_generate: 1.5,
                    n_prefill: 3,
                    n_generate: 4,
                    is_aborted: true,
                }),
                "stable_tokens_independent" => {
                    let first = DecodeResult::default();
                    let second = DecodeResult {
                        stable_tokens: vec![1, 2],
                        ..Default::default()
                    };
                    json!({
                        "same_object": false,
                        "first": first.stable_tokens,
                        "second": second.stable_tokens,
                    })
                }
                _ => panic!("unknown decode category {category}"),
            },
            "asr_engine_config" => match category {
                "defaults" => asr_engine_config_payload(&ASREngineConfig::new("model")),
                "custom" => asr_engine_config_payload(&ASREngineConfig {
                    model_dir: "m".to_string(),
                    encoder_frontend_fn: "front.onnx".to_string(),
                    encoder_backend_fn: "back.onnx".to_string(),
                    llm_fn: "llm.gguf".to_string(),
                    use_dml: false,
                    n_ctx: 1024,
                    chunk_size: 12.5,
                    memory_num: 2,
                    max_decode_tokens: 42,
                    llama_backend: "cpu".to_string(),
                    verbose: false,
                }),
                _ => panic!("unknown config category {category}"),
            },
            "transcribe_result" => match category {
                "defaults" => transcribe_result_payload(&TranscribeResult::new("txt")),
                "custom" => transcribe_result_payload(&TranscribeResult {
                    text: "txt".to_string(),
                    performance: Some(json!({"rtf": 0.5})),
                }),
                _ => panic!("unknown transcribe category {category}"),
            },
            "constructor_error" => result_json::<()>(Err(
                constructor_missing_error(category).expect("known constructor error")
            )),
            _ => panic!("unknown kind {kind}"),
        }
    }

    fn expected_for_case(kind: &str, category: &str, case: &Value) -> Value {
        match kind {
            "normalize_language_name" | "validate_language" | "constructor_error" => {
                case["result"].clone()
            }
            "supported_languages" => case["languages"].clone(),
            "msg_types" => case["items"].clone(),
            "streaming_message" => case["snapshot"].clone(),
            "decode_result" if category == "stable_tokens_independent" => json!({
                "same_object": case["same_object"],
                "first": case["first"],
                "second": case["second"],
            }),
            "decode_result" | "asr_engine_config" | "transcribe_result" => json!({
                "snapshot": case["snapshot"],
                "repr": case["repr"],
            }),
            _ => panic!("unknown expected kind {kind}"),
        }
    }

    fn result_json<T: Into<Value>>(result: Result<T, SchemaError>) -> Value {
        match result {
            Ok(value) => json!({"ok": true, "value": value.into()}),
            Err(error) => {
                json!({"ok": false, "error_type": error.error_type, "message": error.message})
            }
        }
    }

    fn streaming_snapshot(message: &StreamingMessage) -> Value {
        json!({
            "msg_type_name": message.msg_type.name(),
            "msg_type_value": message.msg_type.value(),
            "data": message.data,
            "is_last": message.is_last,
            "encode_time": message.encode_time,
            "repr": message.python_repr(),
        })
    }

    fn decode_result_payload(value: &DecodeResult) -> Value {
        json!({
            "snapshot": {
                "text": value.text,
                "stable_tokens": value.stable_tokens,
                "t_prefill": value.t_prefill,
                "t_generate": value.t_generate,
                "n_prefill": value.n_prefill,
                "n_generate": value.n_generate,
                "is_aborted": value.is_aborted,
            },
            "repr": value.python_repr(),
        })
    }

    fn asr_engine_config_payload(value: &ASREngineConfig) -> Value {
        json!({
            "snapshot": {
                "model_dir": value.model_dir,
                "encoder_frontend_fn": value.encoder_frontend_fn,
                "encoder_backend_fn": value.encoder_backend_fn,
                "llm_fn": value.llm_fn,
                "use_dml": value.use_dml,
                "n_ctx": value.n_ctx,
                "chunk_size": value.chunk_size,
                "memory_num": value.memory_num,
                "max_decode_tokens": value.max_decode_tokens,
                "llama_backend": value.llama_backend,
                "verbose": value.verbose,
            },
            "repr": value.python_repr(),
        })
    }

    fn transcribe_result_payload(value: &TranscribeResult) -> Value {
        json!({
            "snapshot": {
                "text": value.text,
                "performance": value.performance,
            },
            "repr": value.python_repr(),
        })
    }
}
