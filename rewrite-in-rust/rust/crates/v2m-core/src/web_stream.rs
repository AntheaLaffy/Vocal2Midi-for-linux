//! Web stream redirector behavior.
//!
//! This module mirrors `web_stream_redirector.py::WebStreamRedirector` for
//! fixture-backed write/flush/delegation behavior while legacy Python remains
//! the runtime owner.

/// Callback call emitted by a non-empty write.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamCallbackCall {
    /// The message text.
    pub message: String,
    /// The level.
    pub level: String,
}

/// Result of modeling one redirector operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamRedirectOutcome {
    /// The ordered stream writes.
    pub stream_writes: Vec<String>,
    /// The ordered callbacks.
    pub callbacks: Vec<StreamCallbackCall>,
    /// Whether the callback error was swallowed.
    pub callback_error_swallowed: bool,
    /// The number of flush calls.
    pub flush_count: usize,
    /// The optional attribute value.
    pub attribute_value: Option<String>,
}

/// Models `WebStreamRedirector.write`.
pub fn redirect_write(text: &str, callback_mode: CallbackMode) -> StreamRedirectOutcome {
    let mut callbacks = Vec::new();
    let mut callback_error_swallowed = false;
    let stripped = text.trim();
    if !stripped.is_empty() && callback_mode != CallbackMode::None {
        callbacks.push(StreamCallbackCall {
            message: stripped.to_string(),
            level: "info".to_string(),
        });
        callback_error_swallowed = callback_mode == CallbackMode::Raise;
    }

    StreamRedirectOutcome {
        stream_writes: vec![text.to_string()],
        callbacks,
        callback_error_swallowed,
        flush_count: 0,
        attribute_value: None,
    }
}

/// Models `WebStreamRedirector.flush`.
pub fn redirect_flush() -> StreamRedirectOutcome {
    StreamRedirectOutcome {
        stream_writes: Vec::new(),
        callbacks: Vec::new(),
        callback_error_swallowed: false,
        flush_count: 1,
        attribute_value: None,
    }
}

/// Models delegated attribute access on the underlying stream.
pub fn redirect_getattr(
    attribute: &str,
    stream_attributes: &[(&str, &str)],
) -> StreamRedirectOutcome {
    let value = stream_attributes
        .iter()
        .find_map(|(name, value)| (*name == attribute).then(|| (*value).to_string()));
    StreamRedirectOutcome {
        stream_writes: Vec::new(),
        callbacks: Vec::new(),
        callback_error_swallowed: false,
        flush_count: 0,
        attribute_value: value,
    }
}

/// Callback behavior in the fixture model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallbackMode {
    /// Represents the Python-compatible record case.
    Record,
    /// Represents the Python-compatible raise case.
    Raise,
    /// Represents the Python-compatible none case.
    None,
}

#[cfg(test)]
impl CallbackMode {
    fn from_fixture(value: &str) -> Self {
        match value {
            "record" => Self::Record,
            "raise" => Self::Raise,
            "none" => Self::None,
            _ => panic!("unknown callback mode {value}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    const FIXTURES: &str =
        include_str!("../../../../fixtures/web_stream_redirector_contract.jsonl");

    fn expected_strings(value: &Value, key: &str) -> Vec<String> {
        value[key]
            .as_array()
            .unwrap()
            .iter()
            .map(|item| item.as_str().unwrap().to_string())
            .collect()
    }

    fn expected_callbacks(value: &Value) -> Vec<StreamCallbackCall> {
        value["callbacks"]
            .as_array()
            .unwrap()
            .iter()
            .map(|item| {
                let item = item.as_array().unwrap();
                StreamCallbackCall {
                    message: item[0].as_str().unwrap().to_string(),
                    level: item[1].as_str().unwrap().to_string(),
                }
            })
            .collect()
    }

    #[test]
    fn web_stream_redirector_follows_parity_fixture_table() {
        for (line_number, line) in FIXTURES.lines().enumerate() {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let case: Value = serde_json::from_str(line).unwrap();
            let expected = &case["expect"];
            let callback_mode = CallbackMode::from_fixture(case["callback"].as_str().unwrap());
            let actual = match case["operation"].as_str().unwrap() {
                "write" => redirect_write(case["text"].as_str().unwrap(), callback_mode),
                "flush" => redirect_flush(),
                "getattr" => {
                    let attribute = case["attribute"].as_str().unwrap();
                    let expected_attribute = case["expect"]["attribute_value"].as_str().unwrap();
                    redirect_getattr(attribute, &[(attribute, expected_attribute)])
                }
                operation => panic!("unknown operation {operation}"),
            };

            assert_eq!(
                actual.stream_writes,
                expected_strings(expected, "stream_writes"),
                "fixture line {}",
                line_number + 1
            );
            assert_eq!(
                actual.callbacks,
                expected_callbacks(expected),
                "fixture line {}",
                line_number + 1
            );
            assert_eq!(
                actual.flush_count,
                expected["flush_count"].as_u64().unwrap() as usize,
                "fixture line {}",
                line_number + 1
            );
            if let Some(value) = expected.get("callback_error_swallowed") {
                assert_eq!(
                    actual.callback_error_swallowed,
                    value.as_bool().unwrap(),
                    "fixture line {}",
                    line_number + 1
                );
            }
            if let Some(value) = expected.get("attribute_value") {
                assert_eq!(
                    actual.attribute_value.as_deref(),
                    value.as_str(),
                    "fixture line {}",
                    line_number + 1
                );
            }
        }
    }
}
