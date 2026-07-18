//! Grid-search slicer policy compatibility helpers.
//!
//! This module mirrors the deterministic policy layer from
//! `inference/API/slicer_api.py::grid_search_slice`. Python remains the runtime
//! owner for production slicing, audio IO, CLI parsing, GUI/web callers, and
//! pitch/RMVPE smart slicing.

use crate::slicer_default::Slicer;
use crate::slicer_segment::{Segment, Waveform};

/// One grid-search parameter candidate.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GridSearchParams {
    /// The threshold db.
    pub threshold_db: f64,
    /// The min length ms.
    pub min_length_ms: f64,
    /// The min interval ms.
    pub min_interval_ms: f64,
    /// The max sil kept ms.
    pub max_sil_kept_ms: f64,
}

/// Caller-provided grid-search slicer parameters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GridSearchConfig {
    /// The min len sec.
    pub min_len_sec: f64,
    /// The max len sec.
    pub max_len_sec: f64,
    /// The min interval ms.
    pub min_interval_ms: f64,
    /// The max sil kept ms.
    pub max_sil_kept_ms: f64,
}

impl Default for GridSearchConfig {
    fn default() -> Self {
        Self {
            min_len_sec: 4.0,
            max_len_sec: 20.0,
            min_interval_ms: 200.0,
            max_sil_kept_ms: 500.0,
        }
    }
}

impl GridSearchConfig {
    /// Returns the legacy `itertools.product(thresholds, min_lengths_ms)` order.
    pub fn parameter_grid(self) -> Vec<GridSearchParams> {
        const THRESHOLDS_DB: [f64; 6] = [-45.0, -40.0, -35.0, -30.0, -25.0, -20.0];
        const MIN_LENGTHS_MS: [f64; 5] = [8000.0, 6000.0, 4000.0, 2500.0, 1500.0];

        THRESHOLDS_DB
            .into_iter()
            .flat_map(|threshold_db| {
                MIN_LENGTHS_MS
                    .into_iter()
                    .map(move |min_length_ms| GridSearchParams {
                        threshold_db,
                        min_length_ms,
                        min_interval_ms: self.min_interval_ms,
                        max_sil_kept_ms: self.max_sil_kept_ms,
                    })
            })
            .collect()
    }
}

/// One scored non-empty candidate.
#[derive(Debug, Clone, PartialEq)]
pub struct GridSearchScore {
    /// The params.
    pub params: GridSearchParams,
    /// The score.
    pub score: f64,
    /// The chunk count.
    pub chunk_count: usize,
    /// The short count.
    pub short_count: usize,
    /// The long count.
    pub long_count: usize,
}

/// Grid-search policy result plus review/debug metadata.
#[derive(Debug, Clone, PartialEq)]
pub struct GridSearchResult {
    /// The ordered chunks.
    pub chunks: Vec<Segment>,
    /// The optional best params.
    pub best_params: Option<GridSearchParams>,
    /// The optional best score.
    pub best_score: Option<f64>,
    /// The ordered score log.
    pub score_log: Vec<GridSearchScore>,
}

/// Runs grid search by composing the verified default slicer dependency.
pub fn grid_search_slice(
    waveform: &Waveform,
    sample_rate: f64,
    config: GridSearchConfig,
) -> Vec<Segment> {
    apply_grid_search_policy(waveform, sample_rate, config, |params| {
        let slicer = Slicer::new(
            sample_rate,
            params.threshold_db,
            params.min_length_ms,
            params.min_interval_ms,
            20.0,
            params.max_sil_kept_ms,
        )?;
        let chunks = slicer
            .slice(waveform)?
            .into_iter()
            .map(|chunk| Segment {
                offset: chunk.offset,
                waveform: chunk.waveform,
            })
            .collect();
        Ok::<Vec<Segment>, crate::slicer_default::SlicerDefaultError>(chunks)
    })
    .chunks
}

/// Applies the grid-search policy over dependency-provided slicer outputs.
///
/// Dependency errors and empty outputs are skipped like the legacy Python
/// `try`/`continue` block.
pub fn apply_grid_search_policy<F, E>(
    waveform: &Waveform,
    sample_rate: f64,
    config: GridSearchConfig,
    mut run_slicer: F,
) -> GridSearchResult
where
    F: FnMut(&GridSearchParams) -> Result<Vec<Segment>, E>,
{
    let mut best_chunks = Vec::new();
    let mut best_params = None;
    let mut best_score = f64::INFINITY;
    let mut score_log = Vec::new();

    for params in config.parameter_grid() {
        let Ok(chunks) = run_slicer(&params) else {
            continue;
        };
        if chunks.is_empty() {
            continue;
        }

        let Some(score) = score_chunks(waveform, &chunks, sample_rate, config, params) else {
            continue;
        };

        if score.score < best_score {
            best_score = score.score;
            best_params = Some(params);
            best_chunks = chunks;
        }

        score_log.push(score);
    }

    GridSearchResult {
        chunks: best_chunks,
        best_params,
        best_score: best_params.map(|_| best_score),
        score_log,
    }
}

fn score_chunks(
    waveform: &Waveform,
    chunks: &[Segment],
    sample_rate: f64,
    config: GridSearchConfig,
    params: GridSearchParams,
) -> Option<GridSearchScore> {
    if sample_rate == 0.0 {
        return None;
    }

    let mut score = 0.0;
    let mut short_count = 0;
    let mut long_count = 0;

    for chunk in chunks {
        let duration = chunk.waveform.outer_len() as f64 / sample_rate;
        if duration < config.min_len_sec {
            score += (config.min_len_sec - duration) * 1.5;
            short_count += 1;
        } else if duration > config.max_len_sec {
            score += duration - config.max_len_sec;
            long_count += 1;
        }
    }

    let average_target = (config.min_len_sec + config.max_len_sec) / 2.0;
    if average_target == 0.0 {
        return None;
    }
    let target_count = waveform.outer_len() as f64 / sample_rate / average_target;
    score += (chunks.len() as f64 - target_count).abs() * 0.5;

    Some(GridSearchScore {
        params,
        score,
        chunk_count: chunks.len(),
        short_count,
        long_count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    const FIXTURES: &str =
        include_str!("../../../../fixtures/slicer_grid_search_policy_core.jsonl");

    fn parse_waveform(value: &Value) -> Waveform {
        let rows = value.as_array().unwrap();
        if rows.first().is_some_and(Value::is_array) {
            Waveform::Stereo(
                rows.iter()
                    .map(|row| {
                        row.as_array()
                            .unwrap()
                            .iter()
                            .map(|sample| sample.as_f64().unwrap())
                            .collect()
                    })
                    .collect(),
            )
        } else {
            Waveform::Mono(rows.iter().map(|sample| sample.as_f64().unwrap()).collect())
        }
    }

    fn parse_segment(value: &Value) -> Segment {
        Segment {
            offset: value["offset"].as_f64().unwrap(),
            waveform: parse_waveform(&value["waveform"]),
        }
    }

    fn parse_segments(value: &Value) -> Vec<Segment> {
        value
            .as_array()
            .unwrap()
            .iter()
            .map(parse_segment)
            .collect()
    }

    fn parse_config(value: &Value) -> GridSearchConfig {
        GridSearchConfig {
            min_len_sec: value["min_len_sec"].as_f64().unwrap(),
            max_len_sec: value["max_len_sec"].as_f64().unwrap(),
            min_interval_ms: value["min_interval_ms"].as_f64().unwrap(),
            max_sil_kept_ms: value["max_sil_kept_ms"].as_f64().unwrap(),
        }
    }

    fn encode_waveform(waveform: &Waveform) -> Value {
        match waveform {
            Waveform::Mono(samples) => {
                Value::Array(samples.iter().map(|sample| json!(sample)).collect())
            }
            Waveform::Stereo(channels) => Value::Array(
                channels
                    .iter()
                    .map(|channel| {
                        Value::Array(channel.iter().map(|sample| json!(sample)).collect())
                    })
                    .collect(),
            ),
        }
    }

    fn encode_segment(segment: &Segment) -> Value {
        json!({
            "offset": segment.offset,
            "waveform": encode_waveform(&segment.waveform),
        })
    }

    fn encode_segments(segments: &[Segment]) -> Value {
        Value::Array(segments.iter().map(encode_segment).collect())
    }

    fn encode_params(params: GridSearchParams) -> Value {
        json!([params.threshold_db as i64, params.min_length_ms as i64,])
    }

    fn encode_call_static(calls: &[GridSearchParams], sample_rate: f64) -> Value {
        json!({
            "sr_values": unique_numbers(std::iter::once(sample_rate)),
            "min_interval_values": unique_numbers(calls.iter().map(|call| call.min_interval_ms)),
            "max_sil_kept_values": unique_numbers(calls.iter().map(|call| call.max_sil_kept_ms)),
        })
    }

    fn unique_numbers<I>(values: I) -> Value
    where
        I: IntoIterator<Item = f64>,
    {
        let mut numbers: Vec<f64> = values.into_iter().collect();
        numbers.sort_by(f64::total_cmp);
        numbers.dedup_by(|left, right| (*left - *right).abs() <= f64::EPSILON);
        Value::Array(numbers.into_iter().map(|value| json!(value)).collect())
    }

    fn python_print_score(value: f64) -> Value {
        json!(format!("{value:.2}").parse::<f64>().unwrap())
    }

    fn encode_score_log(score_log: &[GridSearchScore]) -> Value {
        Value::Array(
            score_log
                .iter()
                .map(|entry| {
                    json!({
                        "params": encode_params(entry.params),
                        "score": python_print_score(entry.score),
                        "chunks": entry.chunk_count,
                        "short": entry.short_count,
                        "long": entry.long_count,
                    })
                })
                .collect(),
        )
    }

    fn expanded_call_pairs(source: &Value) -> Value {
        let thresholds = source["thresholds"].as_array().unwrap();
        let min_lengths = source["min_lengths_ms"].as_array().unwrap();
        Value::Array(
            thresholds
                .iter()
                .flat_map(|threshold| {
                    min_lengths
                        .iter()
                        .map(move |min_length| json!([threshold, min_length]))
                })
                .collect(),
        )
    }

    fn assert_json_close(actual: &Value, expected: &Value, context: &str) {
        match (actual, expected) {
            (Value::Number(left), Value::Number(right)) => {
                let left = left.as_f64().unwrap();
                let right = right.as_f64().unwrap();
                assert!(
                    (left - right).abs() <= 1e-2,
                    "{context}: {left:?} != {right:?}"
                );
            }
            (Value::Array(left), Value::Array(right)) => {
                assert_eq!(left.len(), right.len(), "{context}: array lengths differ");
                for (index, (left_item, right_item)) in left.iter().zip(right).enumerate() {
                    assert_json_close(left_item, right_item, &format!("{context}[{index}]"));
                }
            }
            (Value::Object(left), Value::Object(right)) => {
                for (key, right_value) in right {
                    if key == "slicer_call_pairs_from" {
                        let left_value = left
                            .get("slicer_call_pairs")
                            .unwrap_or_else(|| panic!("{context}: missing slicer_call_pairs"));
                        assert_json_close(
                            left_value,
                            &expanded_call_pairs(right_value),
                            &format!("{context}.slicer_call_pairs"),
                        );
                        continue;
                    }

                    let left_value = left
                        .get(key)
                        .unwrap_or_else(|| panic!("{context}: missing {key}"));
                    assert_json_close(left_value, right_value, &format!("{context}.{key}"));
                }
            }
            _ => assert_eq!(actual, expected, "{context}"),
        }
    }

    #[test]
    fn slicer_grid_policy_follows_parity_fixture_table() {
        for (line_index, line) in FIXTURES.lines().enumerate() {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let case: Value = serde_json::from_str(line).unwrap();
            let case_id = case["case_id"].as_str().unwrap();
            let waveform = parse_waveform(&case["waveform"]);
            let sample_rate = case["sr"].as_f64().unwrap();
            let config = parse_config(&case["config"]);
            let mut calls = Vec::new();

            let result = apply_grid_search_policy(&waveform, sample_rate, config, |params| {
                calls.push(*params);
                for outcome in case["outcomes"].as_array().unwrap_or(&Vec::new()) {
                    let pair = outcome["params"].as_array().unwrap();
                    if pair[0].as_f64().unwrap() == params.threshold_db
                        && pair[1].as_f64().unwrap() == params.min_length_ms
                    {
                        if outcome.get("error").is_some() {
                            return Err(());
                        }
                        return Ok(parse_segments(&outcome["chunks"]));
                    }
                }

                let default = &case["default_outcome"];
                if default.get("error").is_some() {
                    return Err(());
                }
                Ok(parse_segments(&default["chunks"]))
            });

            let actual = json!({
                "chunks": encode_segments(&result.chunks),
                "best_params": result.best_params.map(encode_params),
                "best_score": result.best_score.map(python_print_score),
                "call_count": calls.len(),
                "slicer_call_pairs": Value::Array(calls.iter().map(|params| encode_params(*params)).collect()),
                "slicer_call_static": encode_call_static(&calls, sample_rate),
                "score_log": encode_score_log(&result.score_log),
            });

            assert_json_close(
                &actual,
                &case["expect"],
                &format!("{case_id} fixture line {}", line_index + 1),
            );
        }
    }

    #[test]
    fn grid_search_config_defaults_match_python_signature() {
        let config = GridSearchConfig::default();
        assert_eq!(config.min_len_sec, 4.0);
        assert_eq!(config.max_len_sec, 20.0);
        assert_eq!(config.min_interval_ms, 200.0);
        assert_eq!(config.max_sil_kept_ms, 500.0);
    }

    #[test]
    fn grid_search_slice_composes_verified_slicer_dependency() {
        let waveform = Waveform::Mono(vec![1.0; 100]);
        let config = GridSearchConfig {
            min_len_sec: 0.2,
            max_len_sec: 1.0,
            min_interval_ms: 200.0,
            max_sil_kept_ms: 500.0,
        };

        let chunks = grid_search_slice(&waveform, 1000.0, config);

        assert_eq!(
            encode_segments(&chunks),
            json!([{"offset": 0.0, "waveform": vec![1.0; 100]}])
        );
    }
}
