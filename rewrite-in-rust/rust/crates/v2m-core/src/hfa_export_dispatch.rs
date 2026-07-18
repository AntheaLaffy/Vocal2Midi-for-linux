//! HubertFA export dispatch policy.
//!
//! This module mirrors `Exporter.export` and the default/status part of
//! `InferenceBase.export` with injected sinks. Python remains the runtime owner
//! for production inference, printing, file-system effects, and caller routing.

use crate::hfa_htk_export::{HfaHtkExportPlan, HfaHtkPrediction, plan_htk_label_export};
use crate::hfa_textgrid_export::{
    HfaTextGridExportPlan, HfaTextGridPrediction, plan_textgrid_export,
};
use std::error::Error;
use std::fmt;
use std::path::Path;

/// Exact final status line printed by `InferenceBase.export` after success.
pub const INFERENCE_EXPORT_STATUS: &str =
    "Output files are saved to the same folder as the input wav files.\n";

/// Python-shaped format inputs accepted by `Exporter.export`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HfaExportFormats {
    /// List/tuple-style membership against complete string items.
    Iterable(Vec<String>),
    /// Python string membership, where format names are substring checks.
    String(String),
    /// Dict-style membership against mapping keys.
    MappingKeys(Vec<String>),
    /// `None`, which raises before any sink is called in `Exporter.export`.
    None,
}

impl HfaExportFormats {
    /// Builds an iterable format set from borrowed string items.
    pub fn iterable(items: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self::Iterable(items.into_iter().map(Into::into).collect())
    }

    /// Returns whether Python's `needle in out_formats` would succeed.
    ///
    /// # Errors
    ///
    /// Returns the Python-compatible `TypeError` raised by membership against
    /// `None`.
    pub fn contains(&self, needle: &str) -> Result<bool, HfaExportDispatchError> {
        match self {
            Self::Iterable(items) | Self::MappingKeys(items) => {
                Ok(items.iter().any(|item| item == needle))
            }
            Self::String(value) => Ok(value.contains(needle)),
            Self::None => Err(HfaExportDispatchError::none_not_iterable()),
        }
    }
}

/// Sink called by the dispatch layer.
pub trait HfaExportSink {
    /// Saves or plans TextGrid output.
    ///
    /// # Errors
    ///
    /// Propagates the downstream TextGrid sink failure without calling later
    /// formats.
    fn save_textgrids(&mut self) -> Result<(), HfaExportDispatchError>;

    /// Saves or plans HTK label output.
    ///
    /// # Errors
    ///
    /// Propagates the downstream HTK sink failure.
    fn save_htk(&mut self) -> Result<(), HfaExportDispatchError>;
}

/// Python-compatible dispatch/downstream failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HfaExportDispatchError {
    exception_type: String,
    message: String,
}

impl HfaExportDispatchError {
    /// Builds a downstream error projected as a Python exception type/message.
    pub fn downstream(exception_type: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            exception_type: exception_type.into(),
            message: message.into(),
        }
    }

    fn none_not_iterable() -> Self {
        Self::downstream("TypeError", "argument of type 'NoneType' is not iterable")
    }

    /// Legacy Python exception type.
    pub fn exception_type(&self) -> &str {
        &self.exception_type
    }

    /// Legacy Python error message.
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for HfaExportDispatchError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for HfaExportDispatchError {}

/// Runs `Exporter.export` dispatch against an injected sink.
///
/// TextGrid is checked and called before HTK regardless of caller input order.
/// Unknown formats and duplicates are ignored by Python's membership checks.
///
/// # Errors
///
/// Returns the Python-compatible `None` membership error or the first
/// downstream sink error.
pub fn export_with_sink(
    sink: &mut impl HfaExportSink,
    out_formats: &HfaExportFormats,
) -> Result<(), HfaExportDispatchError> {
    if out_formats.contains("textgrid")? {
        sink.save_textgrids()?;
    }
    if out_formats.contains("htk")? {
        sink.save_htk()?;
    }
    Ok(())
}

/// Runs the `InferenceBase.export` default/status policy against an injected
/// sink.
///
/// `None` defaults to `['textgrid']`; an explicit empty format set remains a
/// no-op. The returned status line is emitted only after downstream success.
///
/// # Errors
///
/// Propagates the first dispatch or downstream sink error and returns no status
/// line.
pub fn inference_export_with_sink(
    sink: &mut impl HfaExportSink,
    output_format: Option<&HfaExportFormats>,
) -> Result<Vec<&'static str>, HfaExportDispatchError> {
    let default_formats;
    let formats = match output_format {
        Some(formats) => formats,
        None => {
            default_formats = HfaExportFormats::iterable(["textgrid"]);
            &default_formats
        }
    };

    export_with_sink(sink, formats)?;
    Ok(vec![INFERENCE_EXPORT_STATUS])
}

/// Planning sink that composes the verified TextGrid and HTK planner modules.
#[derive(Debug)]
pub struct HfaPlanningExportSink<'a> {
    textgrid_predictions: &'a [HfaTextGridPrediction],
    htk_predictions: &'a [HfaHtkPrediction],
    output_folder: Option<&'a Path>,
    /// TextGrid plan captured if TextGrid dispatch runs successfully.
    pub textgrid_plan: Option<HfaTextGridExportPlan>,
    /// HTK plan captured if HTK dispatch runs successfully.
    pub htk_plan: Option<HfaHtkExportPlan>,
}

impl<'a> HfaPlanningExportSink<'a> {
    /// Creates a planning sink over already-normalized exporter predictions.
    pub fn new(
        textgrid_predictions: &'a [HfaTextGridPrediction],
        htk_predictions: &'a [HfaHtkPrediction],
        output_folder: Option<&'a Path>,
    ) -> Self {
        Self {
            textgrid_predictions,
            htk_predictions,
            output_folder,
            textgrid_plan: None,
            htk_plan: None,
        }
    }
}

impl HfaExportSink for HfaPlanningExportSink<'_> {
    fn save_textgrids(&mut self) -> Result<(), HfaExportDispatchError> {
        let plan = plan_textgrid_export(self.textgrid_predictions, self.output_folder).map_err(
            |failure| {
                HfaExportDispatchError::downstream(
                    failure.error.exception_type(),
                    failure.error.message(),
                )
            },
        )?;
        self.textgrid_plan = Some(plan);
        Ok(())
    }

    fn save_htk(&mut self) -> Result<(), HfaExportDispatchError> {
        let plan =
            plan_htk_label_export(self.htk_predictions, self.output_folder).map_err(|failure| {
                HfaExportDispatchError::downstream(
                    failure.error.exception_type(),
                    failure.error.message(),
                )
            })?;
        self.htk_plan = Some(plan);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    const FIXTURES: &str = include_str!("../../../../fixtures/hfa_export_dispatch_contract.jsonl");

    #[derive(Debug)]
    struct RecordingSink {
        events: Vec<Value>,
        errors: Value,
    }

    impl RecordingSink {
        fn new(case: &Value) -> Self {
            let mut events = Vec::new();
            let output_folder = case.get("output_folder").and_then(Value::as_str);
            events.push(json!({
                "event": "init",
                "predictions": python_repr_string(
                    case.get("predictions").and_then(Value::as_str).unwrap_or("PRED")
                ),
                "output_folder": output_folder,
            }));
            Self {
                events,
                errors: case.get("errors").cloned().unwrap_or_else(|| json!({})),
            }
        }

        fn into_events(self) -> Vec<Value> {
            self.events
        }

        fn maybe_error(&self, format: &str) -> Result<(), HfaExportDispatchError> {
            match self.errors.get(format) {
                Some(error) => Err(HfaExportDispatchError::downstream(
                    error.get("type").and_then(Value::as_str).unwrap(),
                    error.get("message").and_then(Value::as_str).unwrap(),
                )),
                None => Ok(()),
            }
        }
    }

    impl HfaExportSink for RecordingSink {
        fn save_textgrids(&mut self) -> Result<(), HfaExportDispatchError> {
            self.events.push(json!({"event": "textgrid"}));
            self.maybe_error("textgrid")
        }

        fn save_htk(&mut self) -> Result<(), HfaExportDispatchError> {
            self.events.push(json!({"event": "htk"}));
            self.maybe_error("htk")
        }
    }

    fn decode_format(value: &Value) -> HfaExportFormats {
        if let Some(object) = value.as_object() {
            match object.get("$kind").and_then(Value::as_str) {
                Some("tuple") => {
                    return HfaExportFormats::Iterable(decode_string_items(
                        object.get("items").unwrap(),
                    ));
                }
                Some("mapping") => {
                    let keys = object
                        .get("items")
                        .and_then(Value::as_array)
                        .unwrap()
                        .iter()
                        .map(|item| {
                            item.as_array()
                                .unwrap()
                                .first()
                                .and_then(Value::as_str)
                                .unwrap()
                                .to_string()
                        })
                        .collect();
                    return HfaExportFormats::MappingKeys(keys);
                }
                Some("none") => return HfaExportFormats::None,
                Some(other) => panic!("unknown format marker {other:?}"),
                None => {}
            }
        }
        if let Some(items) = value.as_array() {
            HfaExportFormats::Iterable(
                items
                    .iter()
                    .map(|item| item.as_str().unwrap().to_string())
                    .collect(),
            )
        } else {
            HfaExportFormats::String(value.as_str().unwrap().to_string())
        }
    }

    fn python_repr_string(value: &str) -> String {
        if value.contains('\'') && !value.contains('"') {
            format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
        } else {
            format!("'{}'", value.replace('\\', "\\\\").replace('\'', "\\'"))
        }
    }

    fn decode_string_items(value: &Value) -> Vec<String> {
        value
            .as_array()
            .unwrap()
            .iter()
            .map(|item| item.as_str().unwrap().to_string())
            .collect()
    }

    fn project_result(result: Result<(), HfaExportDispatchError>) -> Value {
        match result {
            Ok(()) => json!({"ok": Value::Null}),
            Err(error) => json!({
                "error": {
                    "type": error.exception_type(),
                    "message": error.message(),
                },
            }),
        }
    }

    fn run_exporter_case(case: &Value) -> Value {
        let mut sink = RecordingSink::new(case);
        let formats = decode_format(case.get("out_formats").unwrap());
        let mut result = project_result(export_with_sink(&mut sink, &formats));
        result
            .as_object_mut()
            .unwrap()
            .insert("events".to_string(), Value::Array(sink.into_events()));
        result
    }

    fn run_inference_case(case: &Value) -> Value {
        let mut sink = RecordingSink::new(case);
        let output_format = decode_optional_output_format(case.get("output_format"));
        match inference_export_with_sink(&mut sink, output_format.as_ref()) {
            Ok(prints) => json!({
                "ok": Value::Null,
                "events": sink.into_events(),
                "prints": prints,
            }),
            Err(error) => json!({
                "error": {
                    "type": error.exception_type(),
                    "message": error.message(),
                },
                "events": sink.into_events(),
                "prints": [],
            }),
        }
    }

    fn decode_optional_output_format(value: Option<&Value>) -> Option<HfaExportFormats> {
        let value = value?;
        if value
            .as_object()
            .and_then(|object| object.get("$kind"))
            .and_then(Value::as_str)
            == Some("none")
        {
            None
        } else {
            Some(decode_format(value))
        }
    }

    fn run_case(case: &Value) -> Value {
        let repeat = case.get("repeat").and_then(Value::as_u64).unwrap_or(1);
        let kind = case.get("kind").and_then(Value::as_str).unwrap();
        let calls = (0..repeat)
            .map(|_| match kind {
                "exporter" => run_exporter_case(case),
                "inference" => run_inference_case(case),
                other => panic!("unknown case kind {other:?}"),
            })
            .collect::<Vec<_>>();
        json!({"calls": calls})
    }

    #[test]
    fn hfa_export_dispatch_contract_fixture_parity() {
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
