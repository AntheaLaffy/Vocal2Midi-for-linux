//! Romaji ASR vocab and CTC decode helper contract.
//!
//! This module mirrors the deterministic helper behavior from
//! `inference/romaji_asr/common.py`. It is not wired into Python runtime loading
//! and does not own audio IO, ONNX Runtime sessions, or model execution.

use std::collections::BTreeMap;
use std::fmt;

use ndarray::{ArrayView2, ArrayView3, Axis};
use serde_json::Value;

/// Integer token identifier after Python-compatible `int()` conversion.
pub type TokenId = i128;
/// Deterministically ordered token-ID to phoneme mapping.
pub type Id2Token = BTreeMap<TokenId, String>;

#[derive(Debug, Clone, PartialEq)]
/// Python-compatible vocabulary or chunk-size conversion failure.
pub struct RomajiDecodeError {
    /// The Python-compatible error type.
    pub error_type: &'static str,
    /// The message text.
    pub message: String,
}

impl RomajiDecodeError {
    fn type_error(message: impl Into<String>) -> Self {
        Self {
            error_type: "TypeError",
            message: message.into(),
        }
    }

    fn value_error(message: impl Into<String>) -> Self {
        Self {
            error_type: "ValueError",
            message: message.into(),
        }
    }
}

impl fmt::Display for RomajiDecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.error_type, self.message)
    }
}

impl std::error::Error for RomajiDecodeError {}

/// Parses a token-to-ID JSON object and resolves its CTC blank token.
///
/// `<blank>` takes precedence over `PAD`; zero is used when neither exists.
///
/// # Errors
///
/// Returns [`RomajiDecodeError`] for invalid JSON, a non-object root, or a
/// token ID that Python-compatible integer conversion rejects.
pub fn load_vocab_from_json_str(
    vocab_json: &str,
) -> Result<(Id2Token, TokenId), RomajiDecodeError> {
    let value: Value = serde_json::from_str(vocab_json)
        .map_err(|err| RomajiDecodeError::value_error(err.to_string()))?;
    let vocab = value
        .as_object()
        .ok_or_else(|| RomajiDecodeError::type_error("'list' object has no attribute 'items'"))?;

    let mut id2token = Id2Token::new();
    for (token, id_value) in vocab {
        id2token.insert(python_int(id_value)?, token.clone());
    }

    let blank_id = vocab
        .get("<blank>")
        .or_else(|| vocab.get("PAD"))
        .map(python_int)
        .transpose()?
        .unwrap_or(0);
    Ok((id2token, blank_id))
}

/// Collapses repeated CTC IDs, removes blanks, and maps unknown IDs to `<unk>`.
pub fn decode_pred_ids<I>(pred_ids: I, id2token: &Id2Token, blank_id: TokenId) -> Vec<String>
where
    I: IntoIterator<Item = TokenId>,
{
    let mut out = Vec::new();
    let mut prev = -1;
    for token_id in pred_ids {
        if token_id != prev && token_id != blank_id {
            out.push(
                id2token
                    .get(&token_id)
                    .cloned()
                    .unwrap_or_else(|| "<unk>".to_string()),
            );
        }
        prev = token_id;
    }
    out
}

/// Decodes one `f32` logits matrix with NumPy-compatible last-axis `argmax`.
///
/// # Panics
///
/// Panics when the logits matrix has zero columns, matching the unsupported
/// empty-class input assumed by the legacy helper.
pub fn decode_logits_f32(
    logits: ArrayView2<'_, f32>,
    id2token: &Id2Token,
    blank_id: TokenId,
) -> Vec<String> {
    decode_pred_ids(argmax_axis_last_f32(logits), id2token, blank_id)
}

/// Decodes one `f64` logits matrix with NumPy-compatible last-axis `argmax`.
///
/// # Panics
///
/// Panics when the logits matrix has zero columns.
pub fn decode_logits_f64(
    logits: ArrayView2<'_, f64>,
    id2token: &Id2Token,
    blank_id: TokenId,
) -> Vec<String> {
    decode_pred_ids(argmax_axis_last_f64(logits), id2token, blank_id)
}

/// Decodes a batch of integer token-ID rows.
pub fn decode_outputs_int(
    outputs: ArrayView2<'_, i64>,
    id2token: &Id2Token,
    blank_id: TokenId,
) -> Vec<Vec<String>> {
    outputs
        .axis_iter(Axis(0))
        .map(|row| {
            decode_pred_ids(
                row.iter().map(|value| *value as TokenId),
                id2token,
                blank_id,
            )
        })
        .collect()
}

/// Decodes a batch of unsigned integer token-ID rows.
pub fn decode_outputs_uint(
    outputs: ArrayView2<'_, u64>,
    id2token: &Id2Token,
    blank_id: TokenId,
) -> Vec<Vec<String>> {
    outputs
        .axis_iter(Axis(0))
        .map(|row| {
            decode_pred_ids(
                row.iter().map(|value| *value as TokenId),
                id2token,
                blank_id,
            )
        })
        .collect()
}

/// Decodes a batch of `f32` logits matrices.
///
/// # Panics
///
/// Panics when the class dimension is empty.
pub fn decode_outputs_logits_f32(
    outputs: ArrayView3<'_, f32>,
    id2token: &Id2Token,
    blank_id: TokenId,
) -> Vec<Vec<String>> {
    outputs
        .axis_iter(Axis(0))
        .map(|item| decode_logits_f32(item, id2token, blank_id))
        .collect()
}

/// Decodes a batch of `f64` logits matrices.
///
/// # Panics
///
/// Panics when the class dimension is empty.
pub fn decode_outputs_logits_f64(
    outputs: ArrayView3<'_, f64>,
    id2token: &Id2Token,
    blank_id: TokenId,
) -> Vec<Vec<String>> {
    outputs
        .axis_iter(Axis(0))
        .map(|item| decode_logits_f64(item, id2token, blank_id))
        .collect()
}

/// Clones a slice into chunks, treating values below one as a step of one.
pub fn chunked<T: Clone>(items: &[T], chunk_size: i64) -> Vec<Vec<T>> {
    let step = chunk_size.max(1) as usize;
    items.chunks(step).map(<[T]>::to_vec).collect()
}

/// Converts a JSON chunk size with Python `int()` semantics and chunks values.
///
/// # Errors
///
/// Returns [`RomajiDecodeError`] when the JSON value cannot be converted to an
/// integer or does not fit in `i64`.
pub fn chunked_json_values(
    items: &[Value],
    chunk_size: &Value,
) -> Result<Vec<Vec<Value>>, RomajiDecodeError> {
    let chunk_size = i64::try_from(python_int(chunk_size)?)
        .map_err(|_| RomajiDecodeError::value_error("Python int value is out of Rust i64 range"))?;
    Ok(chunked(items, chunk_size))
}

fn argmax_axis_last_f32(values: ArrayView2<'_, f32>) -> Vec<TokenId> {
    values
        .axis_iter(Axis(0))
        .map(|row| {
            let mut best_index = 0usize;
            let mut best_value = row[0];
            for (index, value) in row.iter().copied().enumerate().skip(1) {
                if numpy_argmax_should_update_f32(best_value, value) {
                    best_index = index;
                    best_value = value;
                }
            }
            best_index as TokenId
        })
        .collect()
}

fn argmax_axis_last_f64(values: ArrayView2<'_, f64>) -> Vec<TokenId> {
    values
        .axis_iter(Axis(0))
        .map(|row| {
            let mut best_index = 0usize;
            let mut best_value = row[0];
            for (index, value) in row.iter().copied().enumerate().skip(1) {
                if numpy_argmax_should_update_f64(best_value, value) {
                    best_index = index;
                    best_value = value;
                }
            }
            best_index as TokenId
        })
        .collect()
}

fn numpy_argmax_should_update_f32(best: f32, value: f32) -> bool {
    (!best.is_nan() && value.is_nan()) || (!best.is_nan() && value > best)
}

fn numpy_argmax_should_update_f64(best: f64, value: f64) -> bool {
    (!best.is_nan() && value.is_nan()) || (!best.is_nan() && value > best)
}

fn python_int(value: &Value) -> Result<TokenId, RomajiDecodeError> {
    match value {
        Value::Number(number) => {
            if let Some(value) = number.as_i64() {
                Ok(value as TokenId)
            } else if let Some(value) = number.as_u64() {
                Ok(value as TokenId)
            } else if let Some(value) = number.as_f64() {
                if value.is_finite() {
                    Ok(value.trunc() as TokenId)
                } else {
                    Err(RomajiDecodeError::value_error(
                        "cannot convert float NaN to integer",
                    ))
                }
            } else {
                Err(RomajiDecodeError::value_error(
                    "unsupported JSON number for int()",
                ))
            }
        }
        Value::String(value) => value.parse::<TokenId>().map_err(|_| {
            RomajiDecodeError::value_error(format!(
                "invalid literal for int() with base 10: '{value}'"
            ))
        }),
        Value::Bool(value) => Ok(TokenId::from(*value as i8)),
        Value::Null => Err(RomajiDecodeError::type_error(
            "int() argument must be a string, a bytes-like object or a real number, not 'NoneType'",
        )),
        Value::Array(_) => Err(RomajiDecodeError::type_error(
            "int() argument must be a string, a bytes-like object or a real number, not 'list'",
        )),
        Value::Object(_) => Err(RomajiDecodeError::type_error(
            "int() argument must be a string, a bytes-like object or a real number, not 'dict'",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::{Array2, Array3};
    use serde_json::Map;

    const FIXTURES: &str =
        include_str!("../../../../fixtures/asr_romaji_vocab_ctc_decode_core.jsonl");

    #[test]
    fn asr_romaji_vocab_ctc_decode_core_matches_fixtures() {
        for (index, line) in FIXTURES.lines().filter(|line| !line.is_empty()).enumerate() {
            let case: Value = serde_json::from_str(line).unwrap();
            let actual = result_for(&case);
            let expected = case["expected"].clone();
            assert_eq!(actual, expected, "case {index}: {}", case["category"]);
        }
    }

    fn result_for(case: &Value) -> Value {
        match case["call"].as_str().unwrap() {
            "load_vocab" => match load_vocab_from_json_str(case["vocab_json"].as_str().unwrap()) {
                Ok((id2token, blank_id)) => json_map_result(id2token, blank_id),
                Err(err) => error_result(err),
            },
            "decode_pred_ids" => {
                let pred_ids = int_vec(&case["pred_ids"]);
                let tokens = decode_pred_ids(pred_ids, &id2token_from_case(case), blank_id(case));
                serde_json::json!({"ok": true, "tokens": tokens})
            }
            "decode_logits" => {
                let tokens = match case["dtype"].as_str().unwrap() {
                    "float64" => decode_logits_f64(
                        array2_f64(&case["logits"]).view(),
                        &id2token_from_case(case),
                        blank_id(case),
                    ),
                    _ => decode_logits_f32(
                        array2_f32(&case["logits"]).view(),
                        &id2token_from_case(case),
                        blank_id(case),
                    ),
                };
                serde_json::json!({"ok": true, "tokens": tokens})
            }
            "decode_outputs" => {
                let predictions = match case["dtype"].as_str().unwrap() {
                    "uint32" | "uint64" => decode_outputs_uint(
                        array2_u64(&case["outputs"]).view(),
                        &id2token_from_case(case),
                        blank_id(case),
                    ),
                    "int64" | "int32" => decode_outputs_int(
                        array2_i64(&case["outputs"]).view(),
                        &id2token_from_case(case),
                        blank_id(case),
                    ),
                    "float64" => decode_outputs_logits_f64(
                        array3_f64(&case["outputs"]).view(),
                        &id2token_from_case(case),
                        blank_id(case),
                    ),
                    _ => decode_outputs_logits_f32(
                        array3_f32(&case["outputs"]).view(),
                        &id2token_from_case(case),
                        blank_id(case),
                    ),
                };
                serde_json::json!({"ok": true, "predictions": predictions})
            }
            "chunked" => {
                let items = case["items"].as_array().unwrap();
                match chunked_json_values(items, &case["chunk_size"]) {
                    Ok(chunks) => serde_json::json!({"ok": true, "chunks": chunks}),
                    Err(err) => error_result(err),
                }
            }
            other => panic!("unknown call {other}"),
        }
    }

    fn json_map_result(id2token: Id2Token, blank_id: TokenId) -> Value {
        let mut map = Map::new();
        for (key, value) in id2token {
            map.insert(key.to_string(), Value::String(value));
        }
        serde_json::json!({"ok": true, "id2token": map, "blank_id": blank_id})
    }

    fn error_result(err: RomajiDecodeError) -> Value {
        serde_json::json!({"ok": false, "error_type": err.error_type, "message": err.message})
    }

    fn id2token_from_case(case: &Value) -> Id2Token {
        case["id2token"]
            .as_object()
            .unwrap()
            .iter()
            .map(|(key, value)| {
                (
                    key.parse::<TokenId>().unwrap(),
                    value.as_str().unwrap().to_string(),
                )
            })
            .collect()
    }

    fn blank_id(case: &Value) -> TokenId {
        json_int(&case["blank_id"])
    }

    fn int_vec(value: &Value) -> Vec<TokenId> {
        value.as_array().unwrap().iter().map(json_int).collect()
    }

    fn json_int(value: &Value) -> TokenId {
        if let Some(value) = value.as_i64() {
            value as TokenId
        } else if let Some(value) = value.as_u64() {
            value as TokenId
        } else {
            value.as_f64().unwrap().trunc() as TokenId
        }
    }

    fn array2_i64(value: &Value) -> Array2<i64> {
        let rows = value.as_array().unwrap();
        let row_count = rows.len();
        let col_count = rows.first().unwrap().as_array().unwrap().len();
        let data = rows
            .iter()
            .flat_map(|row| {
                row.as_array()
                    .unwrap()
                    .iter()
                    .map(|value| value.as_i64().unwrap())
            })
            .collect();
        Array2::from_shape_vec((row_count, col_count), data).unwrap()
    }

    fn array2_u64(value: &Value) -> Array2<u64> {
        let rows = value.as_array().unwrap();
        let row_count = rows.len();
        let col_count = rows.first().unwrap().as_array().unwrap().len();
        let data = rows
            .iter()
            .flat_map(|row| {
                row.as_array()
                    .unwrap()
                    .iter()
                    .map(|value| value.as_u64().unwrap())
            })
            .collect();
        Array2::from_shape_vec((row_count, col_count), data).unwrap()
    }

    fn array2_f32(value: &Value) -> Array2<f32> {
        let rows = value.as_array().unwrap();
        let row_count = rows.len();
        let col_count = rows.first().unwrap().as_array().unwrap().len();
        let data = rows
            .iter()
            .flat_map(|row| {
                row.as_array()
                    .unwrap()
                    .iter()
                    .map(|value| json_float(value) as f32)
            })
            .collect();
        Array2::from_shape_vec((row_count, col_count), data).unwrap()
    }

    fn array2_f64(value: &Value) -> Array2<f64> {
        let rows = value.as_array().unwrap();
        let row_count = rows.len();
        let col_count = rows.first().unwrap().as_array().unwrap().len();
        let data = rows
            .iter()
            .flat_map(|row| row.as_array().unwrap().iter().map(json_float))
            .collect();
        Array2::from_shape_vec((row_count, col_count), data).unwrap()
    }

    fn array3_f32(value: &Value) -> Array3<f32> {
        let (shape, data) = array3_shape_and_data(value, |value| json_float(value) as f32);
        Array3::from_shape_vec(shape, data).unwrap()
    }

    fn array3_f64(value: &Value) -> Array3<f64> {
        let (shape, data) = array3_shape_and_data(value, json_float);
        Array3::from_shape_vec(shape, data).unwrap()
    }

    fn array3_shape_and_data<T>(
        value: &Value,
        convert: impl Fn(&Value) -> T,
    ) -> ((usize, usize, usize), Vec<T>) {
        let batches = value.as_array().unwrap();
        let batch_count = batches.len();
        let row_count = batches.first().unwrap().as_array().unwrap().len();
        let col_count = batches
            .first()
            .unwrap()
            .as_array()
            .unwrap()
            .first()
            .unwrap()
            .as_array()
            .unwrap()
            .len();
        let data = batches
            .iter()
            .flat_map(|batch| {
                batch
                    .as_array()
                    .unwrap()
                    .iter()
                    .flat_map(|row| row.as_array().unwrap().iter().map(&convert))
            })
            .collect();
        ((batch_count, row_count, col_count), data)
    }

    fn json_float(value: &Value) -> f64 {
        match value {
            Value::String(value) if value == "nan" => f64::NAN,
            Value::String(value) if value == "inf" => f64::INFINITY,
            Value::String(value) if value == "-inf" => f64::NEG_INFINITY,
            value => value.as_f64().unwrap(),
        }
    }
}
