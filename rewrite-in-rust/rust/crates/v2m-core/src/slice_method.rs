//! Slicing method and custom-bound contract helpers.
//!
//! This module mirrors the deterministic contract shared by
//! `inference/API/slicer_api.py` and `scripts/slice_asr_cli.py`. Python remains
//! the runtime owner for actual audio slicing, CLI argument parsing, filesystem
//! work, ASR, RMVPE, and model execution.

use encoding_rs::{Encoding, GB18030, GBK};
use std::fmt::Write as _;

/// Default method used when Python callers pass `None`.
pub const DEFAULT_SLICE_METHOD: &str = "default";

/// Canonical slicing methods accepted by the legacy API and batch CLI.
pub const SLICE_METHOD_CHOICES: &[&str] = &["default", "smart", "heuristic", "grid"];

const SUPPORTED_METHODS_MESSAGE: &str = "default, smart, heuristic, grid";

/// Canonical slicing method names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlicingMethod {
    /// Represents the Python-compatible default case.
    Default,
    /// Represents the Python-compatible smart case.
    Smart,
    /// Represents the Python-compatible heuristic case.
    Heuristic,
    /// Represents the Python-compatible grid case.
    Grid,
}

impl SlicingMethod {
    /// Returns the canonical string used by Python callers.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Smart => "smart",
            Self::Heuristic => "heuristic",
            Self::Grid => "grid",
        }
    }
}

/// Unsupported slicing method error that maps to Python `ValueError`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnsupportedSlicingMethod {
    repr: String,
}

impl UnsupportedSlicingMethod {
    /// Returns the Python-compatible error message.
    pub fn message(&self) -> String {
        format!(
            "Unsupported slicing method: {}. Supported values: {}",
            self.repr, SUPPORTED_METHODS_MESSAGE
        )
    }
}

impl std::fmt::Display for UnsupportedSlicingMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message())
    }
}

impl std::error::Error for UnsupportedSlicingMethod {}

/// Custom min/max duration bounds accepted by slicer contract helpers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CustomSliceBounds {
    /// The min seconds.
    pub min_seconds: f64,
    /// The max seconds.
    pub max_seconds: f64,
}

/// Custom-bound validation failure that maps to Python `ValueError`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CustomSliceBoundsError {
    /// Represents the Python-compatible cli pair required case.
    CliPairRequired,
    /// Represents the Python-compatible cli min negative case.
    CliMinNegative,
    /// Represents the Python-compatible cli max not positive case.
    CliMaxNotPositive,
    /// Represents the Python-compatible cli min greater than max case.
    CliMinGreaterThanMax,
    /// Represents the Python-compatible api pair required case.
    ApiPairRequired,
    /// Represents the Python-compatible api min negative case.
    ApiMinNegative,
    /// Represents the Python-compatible api max not positive case.
    ApiMaxNotPositive,
    /// Represents the Python-compatible api min greater than max case.
    ApiMinGreaterThanMax,
}

impl CustomSliceBoundsError {
    /// Returns the Python-compatible error message.
    pub const fn message(self) -> &'static str {
        match self {
            Self::CliPairRequired => "--min-seconds and --max-seconds must be provided together",
            Self::CliMinNegative => "--min-seconds must be greater than or equal to 0",
            Self::CliMaxNotPositive => "--max-seconds must be greater than 0",
            Self::CliMinGreaterThanMax => {
                "--min-seconds must be less than or equal to --max-seconds"
            }
            Self::ApiPairRequired => "min_len_sec and max_len_sec must be provided together",
            Self::ApiMinNegative => "min_len_sec must be greater than or equal to 0 seconds",
            Self::ApiMaxNotPositive => "max_len_sec must be greater than 0 seconds",
            Self::ApiMinGreaterThanMax => "min_len_sec must be less than or equal to max_len_sec",
        }
    }
}

impl std::fmt::Display for CustomSliceBoundsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.message())
    }
}

impl std::error::Error for CustomSliceBoundsError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BoundsSurface {
    Cli,
    Api,
}

impl BoundsSurface {
    const fn pair_required(self) -> CustomSliceBoundsError {
        match self {
            Self::Cli => CustomSliceBoundsError::CliPairRequired,
            Self::Api => CustomSliceBoundsError::ApiPairRequired,
        }
    }

    const fn min_negative(self) -> CustomSliceBoundsError {
        match self {
            Self::Cli => CustomSliceBoundsError::CliMinNegative,
            Self::Api => CustomSliceBoundsError::ApiMinNegative,
        }
    }

    const fn max_not_positive(self) -> CustomSliceBoundsError {
        match self {
            Self::Cli => CustomSliceBoundsError::CliMaxNotPositive,
            Self::Api => CustomSliceBoundsError::ApiMaxNotPositive,
        }
    }

    const fn min_greater_than_max(self) -> CustomSliceBoundsError {
        match self {
            Self::Cli => CustomSliceBoundsError::CliMinGreaterThanMax,
            Self::Api => CustomSliceBoundsError::ApiMinGreaterThanMax,
        }
    }
}

/// Normalizes a slicer method using the legacy API/CLI alias, repair, and
/// keyword fallback contract.
///
/// # Errors
///
/// Returns `UnsupportedSlicingMethod` with the same message text as the Python
/// `ValueError` when the method cannot be resolved.
pub fn normalize_slicing_method(
    method: Option<&str>,
) -> Result<SlicingMethod, UnsupportedSlicingMethod> {
    let Some(raw_method) = method else {
        return Ok(SlicingMethod::Default);
    };

    let mut candidates = Vec::new();
    for candidate in repair_text_candidates(raw_method) {
        let lowered = candidate.to_lowercase();
        for value in [candidate, lowered] {
            if !value.is_empty() && !candidates.contains(&value) {
                candidates.push(value);
            }
        }
    }

    for candidate in &candidates {
        if let Some(method) = alias_method(candidate) {
            return Ok(method);
        }
    }

    for candidate in &candidates {
        for (method, keywords) in [
            (SlicingMethod::Smart, ["smart", "智能"].as_slice()),
            (SlicingMethod::Heuristic, ["heuristic", "启发式"].as_slice()),
            (SlicingMethod::Grid, ["grid", "网格"].as_slice()),
            (
                SlicingMethod::Default,
                ["default", "默认", "auto"].as_slice(),
            ),
        ] {
            if keywords.iter().any(|keyword| candidate.contains(keyword)) {
                return Ok(method);
            }
        }
    }

    Err(UnsupportedSlicingMethod {
        repr: python_string_repr(raw_method),
    })
}

/// Resolves `scripts/slice_asr_cli.py::resolve_slice_bounds` compatible bounds.
///
/// # Errors
///
/// Returns CLI-specific `ValueError` message text for half-specified or invalid
/// bounds.
pub fn resolve_cli_slice_bounds(
    min_seconds: Option<f64>,
    max_seconds: Option<f64>,
) -> Result<Option<CustomSliceBounds>, CustomSliceBoundsError> {
    resolve_slice_bounds(min_seconds, max_seconds, BoundsSurface::Cli)
}

/// Resolves `inference/API/slicer_api.py::_resolve_custom_slice_bounds`
/// compatible bounds.
///
/// # Errors
///
/// Returns API-specific `ValueError` message text for half-specified or invalid
/// bounds.
pub fn resolve_api_slice_bounds(
    min_seconds: Option<f64>,
    max_seconds: Option<f64>,
) -> Result<Option<CustomSliceBounds>, CustomSliceBoundsError> {
    resolve_slice_bounds(min_seconds, max_seconds, BoundsSurface::Api)
}

fn resolve_slice_bounds(
    min_seconds: Option<f64>,
    max_seconds: Option<f64>,
    surface: BoundsSurface,
) -> Result<Option<CustomSliceBounds>, CustomSliceBoundsError> {
    let (Some(min_seconds), Some(max_seconds)) = (min_seconds, max_seconds) else {
        return if min_seconds.is_none() && max_seconds.is_none() {
            Ok(None)
        } else {
            Err(surface.pair_required())
        };
    };

    if min_seconds < 0.0 {
        return Err(surface.min_negative());
    }
    if max_seconds <= 0.0 {
        return Err(surface.max_not_positive());
    }
    if min_seconds > max_seconds {
        return Err(surface.min_greater_than_max());
    }
    Ok(Some(CustomSliceBounds {
        min_seconds,
        max_seconds,
    }))
}

fn alias_method(value: &str) -> Option<SlicingMethod> {
    match value {
        "auto" | "default" | "默认切片" => Some(SlicingMethod::Default),
        "smart" | "智能切片" => Some(SlicingMethod::Smart),
        "heuristic" | "启发式切片" => Some(SlicingMethod::Heuristic),
        "grid" | "网格搜索切片" => Some(SlicingMethod::Grid),
        _ => None,
    }
}

fn repair_text_candidates(text: &str) -> Vec<String> {
    let stripped = text.trim();
    let mut candidates = vec![stripped.to_owned()];

    for encoding in [GB18030, GBK] {
        if let Some(repaired) = repair_text_candidate(stripped, encoding)
            && !repaired.is_empty()
            && !candidates.contains(&repaired)
        {
            candidates.push(repaired);
        }
    }

    candidates
}

fn repair_text_candidate(text: &str, encoding: &'static Encoding) -> Option<String> {
    let (encoded, _, had_errors) = encoding.encode(text);
    if had_errors {
        return None;
    }

    let repaired = decode_utf8_ignore(&encoded).trim().to_owned();
    Some(repaired)
}

fn decode_utf8_ignore(mut bytes: &[u8]) -> String {
    let mut output = String::new();
    while !bytes.is_empty() {
        match std::str::from_utf8(bytes) {
            Ok(valid) => {
                output.push_str(valid);
                break;
            }
            Err(error) => {
                let valid_up_to = error.valid_up_to();
                if valid_up_to > 0 {
                    output.push_str(std::str::from_utf8(&bytes[..valid_up_to]).unwrap());
                }
                let Some(error_len) = error.error_len() else {
                    break;
                };
                bytes = &bytes[valid_up_to + error_len..];
            }
        }
    }
    output
}

fn python_string_repr(value: &str) -> String {
    let quote = if value.contains('\'') && !value.contains('"') {
        '"'
    } else {
        '\''
    };
    let mut output = String::with_capacity(value.len() + 2);
    output.push(quote);
    for ch in value.chars() {
        match ch {
            '\\' => output.push_str("\\\\"),
            '\'' if quote == '\'' => output.push_str("\\'"),
            '"' if quote == '"' => output.push_str("\\\""),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            _ if ch.is_control() => {
                write!(output, "\\x{:02x}", ch as u32).unwrap();
            }
            _ => output.push(ch),
        }
    }
    output.push(quote);
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    const FIXTURES: &str =
        include_str!("../../../../fixtures/slice_method_and_bounds_contract.jsonl");

    fn parse_fixture_number(value: &Value) -> Option<f64> {
        match value {
            Value::Null => None,
            Value::Number(number) => Some(number.as_f64().unwrap()),
            Value::String(raw) => Some(match raw.as_str() {
                "nan" => f64::NAN,
                "inf" => f64::INFINITY,
                "-inf" => f64::NEG_INFINITY,
                _ => raw.parse().unwrap(),
            }),
            _ => panic!("unsupported numeric fixture value: {value:?}"),
        }
    }

    fn encode_fixture_number(value: f64) -> Value {
        if value.is_nan() {
            return Value::String("nan".to_owned());
        }
        if value.is_infinite() {
            return Value::String(if value.is_sign_positive() {
                "inf".to_owned()
            } else {
                "-inf".to_owned()
            });
        }
        Value::Number(serde_json::Number::from_f64(value).unwrap())
    }

    fn capture_method_result(result: Result<SlicingMethod, UnsupportedSlicingMethod>) -> Value {
        match result {
            Ok(method) => json!({ "ok": method.as_str() }),
            Err(error) => json!({ "err": "ValueError", "message": error.message() }),
        }
    }

    fn capture_bounds_result(
        result: Result<Option<CustomSliceBounds>, CustomSliceBoundsError>,
    ) -> Value {
        match result {
            Ok(None) => json!({ "ok": null }),
            Ok(Some(bounds)) => json!({
                "ok": [
                    encode_fixture_number(bounds.min_seconds),
                    encode_fixture_number(bounds.max_seconds)
                ]
            }),
            Err(error) => json!({ "err": "ValueError", "message": error.message() }),
        }
    }

    #[test]
    fn slice_method_and_custom_bounds_follow_parity_fixture_table() {
        for (line_index, line) in FIXTURES.lines().enumerate() {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let case: Value = serde_json::from_str(line).unwrap();
            let case_id = case["case_id"].as_str().unwrap();
            let kind = case["kind"].as_str().unwrap();

            match kind {
                "normalize_method" => {
                    let input = case["input"].as_str();
                    let actual = capture_method_result(normalize_slicing_method(input));
                    assert_eq!(
                        actual,
                        case["api"],
                        "{case_id} fixture line {} api",
                        line_index + 1
                    );
                    assert_eq!(
                        actual,
                        case["cli"],
                        "{case_id} fixture line {} cli",
                        line_index + 1
                    );
                }
                "cli_bounds" => {
                    let actual = capture_bounds_result(resolve_cli_slice_bounds(
                        parse_fixture_number(&case["min"]),
                        parse_fixture_number(&case["max"]),
                    ));
                    assert_eq!(
                        actual,
                        case["expect"],
                        "{case_id} fixture line {}",
                        line_index + 1
                    );
                }
                "api_bounds" => {
                    let actual = capture_bounds_result(resolve_api_slice_bounds(
                        parse_fixture_number(&case["min"]),
                        parse_fixture_number(&case["max"]),
                    ));
                    assert_eq!(
                        actual,
                        case["expect"],
                        "{case_id} fixture line {}",
                        line_index + 1
                    );
                }
                _ => panic!("{case_id} fixture line {} unknown kind", line_index + 1),
            }
        }
    }

    #[test]
    fn unsupported_method_error_uses_original_python_repr_input() {
        let error = normalize_slicing_method(Some(" unknown\n")).unwrap_err();
        assert_eq!(
            error.message(),
            "Unsupported slicing method: ' unknown\\n'. Supported values: default, smart, heuristic, grid"
        );

        let single_quote_error = normalize_slicing_method(Some("can't")).unwrap_err();
        assert_eq!(
            single_quote_error.message(),
            "Unsupported slicing method: \"can't\". Supported values: default, smart, heuristic, grid"
        );

        let both_quotes_error = normalize_slicing_method(Some("both ' and \"")).unwrap_err();
        assert_eq!(
            both_quotes_error.message(),
            "Unsupported slicing method: 'both \\' and \"'. Supported values: default, smart, heuristic, grid"
        );
    }

    #[test]
    fn custom_bounds_preserve_python_nan_comparison_behavior() {
        let cli_bounds = resolve_cli_slice_bounds(Some(f64::NAN), Some(10.0))
            .unwrap()
            .unwrap();
        assert!(cli_bounds.min_seconds.is_nan());
        assert_eq!(cli_bounds.max_seconds, 10.0);

        let api_bounds = resolve_api_slice_bounds(Some(5.0), Some(f64::NAN))
            .unwrap()
            .unwrap();
        assert_eq!(api_bounds.min_seconds, 5.0);
        assert!(api_bounds.max_seconds.is_nan());
    }
}
