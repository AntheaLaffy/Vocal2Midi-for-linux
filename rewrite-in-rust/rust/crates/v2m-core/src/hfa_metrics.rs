//! HubertFA alignment metric helpers.
//!
//! This module mirrors the deterministic behavior in
//! `inference/HubertFA/tools/metrics.py`. Python remains the runtime owner for
//! production callers, TextGrid parsing/serialization, NumPy compatibility, and
//! any model execution.

use ndarray::Array1;
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

/// One synthetic TextGrid point used by the metrics fixture seam.
#[derive(Debug, Clone, PartialEq)]
pub struct Point {
    pub time: f64,
    pub mark: String,
}

impl Point {
    /// Creates a point with public `time` and `mark` fields.
    pub fn new(time: f64, mark: impl Into<String>) -> Self {
        Self {
            time,
            mark: mark.into(),
        }
    }
}

/// Synthetic `CustomPointTier` model.
///
/// `add_point` intentionally bypasses upstream TextGrid min/max and duplicate
/// validation and inserts at the `bisect_left` position by time.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PointTier {
    pub points: Vec<Point>,
}

impl PointTier {
    /// Creates an empty tier.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a tier by inserting each point through `CustomPointTier.addPoint`.
    pub fn from_points(points: impl IntoIterator<Item = Point>) -> Self {
        let mut tier = Self::new();
        for point in points {
            tier.add_point(point);
        }
        tier
    }

    /// Adds a point using Python `bisect_left` duplicate-time behavior.
    pub fn add_point(&mut self, point: Point) {
        let index = self
            .points
            .iter()
            .position(|existing| !point_lt(existing.time, point.time))
            .unwrap_or(self.points.len());
        self.points.insert(index, point);
    }

    /// Returns the number of points.
    pub fn len(&self) -> usize {
        self.points.len()
    }

    /// Returns true when the tier has no points.
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }
}

fn point_lt(left: f64, right: f64) -> bool {
    left < right
}

/// Python-compatible metric error projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HfaMetricError {
    exception_type: &'static str,
    message: &'static str,
}

impl HfaMetricError {
    fn zero_division() -> Self {
        Self {
            exception_type: "ZeroDivisionError",
            message: "float division by zero",
        }
    }

    fn index_error() -> Self {
        Self {
            exception_type: "IndexError",
            message: "list index out of range",
        }
    }

    fn not_implemented() -> Self {
        Self {
            exception_type: "NotImplementedError",
            message: "",
        }
    }

    /// Legacy Python exception type.
    pub const fn exception_type(&self) -> &'static str {
        self.exception_type
    }

    /// Legacy Python error message.
    pub const fn message(&self) -> &'static str {
        self.message
    }
}

impl fmt::Display for HfaMetricError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.message)
    }
}

impl Error for HfaMetricError {}

/// `VlabelerEditsCount` dynamic-programming metric.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VlabelerEditsCount {
    move_min: i64,
    move_max: i64,
    counts: i64,
}

impl Default for VlabelerEditsCount {
    fn default() -> Self {
        Self::new(1, 2)
    }
}

impl VlabelerEditsCount {
    /// Creates a metric with the legacy frame threshold values.
    pub const fn new(move_min_frames: i64, move_max_frames: i64) -> Self {
        Self {
            move_min: move_min_frames,
            move_max: move_max_frames,
            counts: 0,
        }
    }

    /// Updates the accumulated edit count.
    pub fn update(&mut self, pred: &PointTier, target: &PointTier) {
        let min_len = pred.len().min(target.len());
        let (pred_points, target_points) = if pred.len() != target.len() {
            (&pred.points[..min_len], &target.points[..min_len])
        } else {
            (&pred.points[..], &target.points[..])
        };
        let m = pred_points.len();
        let n = target_points.len();
        let mut dp = vec![vec![0_i64; n + 1]; m + 1];

        for (i, row) in dp.iter_mut().enumerate().skip(1) {
            row[0] = i as i64;
        }
        for (j, cell) in dp[0].iter_mut().enumerate().skip(1) {
            *cell = j as i64 * 2;
        }

        for i in 1..=m {
            for j in 1..=n {
                let mut insert = dp[i][j - 1] + 1;
                if j == 1 || target_points[j - 1].mark != target_points[j - 2].mark {
                    insert += 1;
                }

                let delete = dp[i - 1][j] + 1;

                let mut movement = dp[i - 1][j - 1];
                let time_diff = (pred_points[i - 1].time - target_points[j - 1].time).abs();
                if self.move_min as f64 <= time_diff && time_diff < self.move_max as f64 {
                    movement += 1;
                }
                if pred_points[i - 1].mark != target_points[j - 1].mark {
                    movement += 1;
                }

                dp[i][j] = insert.min(delete).min(movement);
            }
        }

        self.counts += dp[m][n];
    }

    /// Returns the accumulated count.
    pub const fn compute(&self) -> i64 {
        self.counts
    }

    /// Resets the accumulated count.
    pub const fn reset(&mut self) {
        self.counts = 0;
    }
}

/// `VlabelerEditRatio` wrapper over `VlabelerEditsCount`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VlabelerEditRatio {
    pub edit_distance: VlabelerEditsCount,
    pub total: i64,
}

impl Default for VlabelerEditRatio {
    fn default() -> Self {
        Self::new(1, 2)
    }
}

impl VlabelerEditRatio {
    /// Creates a ratio metric with the legacy frame threshold values.
    pub const fn new(move_min_frames: i64, move_max_frames: i64) -> Self {
        Self {
            edit_distance: VlabelerEditsCount::new(move_min_frames, move_max_frames),
            total: 0,
        }
    }

    /// Updates nested edit distance and the target-length denominator.
    pub fn update(&mut self, pred: &PointTier, target: &PointTier) {
        self.edit_distance.update(pred, target);
        self.total += 2 * target.len() as i64;
    }

    /// Returns the rounded ratio or the legacy empty-total default.
    pub fn compute(&self) -> f64 {
        if self.total == 0 {
            1.0
        } else {
            python_round_6(self.edit_distance.compute() as f64 / self.total as f64)
        }
    }

    /// Resets nested edit distance and denominator.
    pub const fn reset(&mut self) {
        self.edit_distance.reset();
        self.total = 0;
    }
}

/// IoU metric over adjacent point spans.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct IntersectionOverUnion {
    pub intersection: BTreeMap<String, f64>,
    pub sum: BTreeMap<String, f64>,
}

impl IntersectionOverUnion {
    /// Creates an empty IoU metric.
    pub fn new() -> Self {
        Self::default()
    }

    /// Updates per-mark duration sums and intersections.
    pub fn update(&mut self, pred: &PointTier, target: &PointTier) {
        let len_pred = pred.len().saturating_sub(1);
        let len_target = target.len().saturating_sub(1);

        for i in 0..len_pred {
            let duration = pred.points[i + 1].time - pred.points[i].time;
            let mark = pred.points[i].mark.clone();
            self.sum
                .entry(mark.clone())
                .and_modify(|value| *value += duration)
                .or_insert_with(|| {
                    self.intersection.insert(mark, 0.0);
                    duration
                });
        }
        for j in 0..len_target {
            let duration = target.points[j + 1].time - target.points[j].time;
            let mark = target.points[j].mark.clone();
            self.sum
                .entry(mark.clone())
                .and_modify(|value| *value += duration)
                .or_insert_with(|| {
                    self.intersection.insert(mark, 0.0);
                    duration
                });
        }

        let mut i = 0;
        let mut j = 0;
        while i < len_pred && j < len_target {
            let pred_point = &pred.points[i];
            let target_point = &target.points[j];
            if pred_point.mark == target_point.mark {
                let intersection = pred.points[i + 1].time.min(target.points[j + 1].time)
                    - pred_point.time.max(target_point.time);
                if intersection > 0.0 {
                    *self
                        .intersection
                        .entry(pred_point.mark.clone())
                        .or_insert(0.0) += intersection;
                }
            }

            if pred.points[i + 1].time < target.points[j + 1].time {
                i += 1;
            } else if pred.points[i + 1].time > target.points[j + 1].time {
                j += 1;
            } else {
                i += 1;
                j += 1;
            }
        }
    }

    /// Computes dict-mode IoU, returning `0.0` for exact zero-union entries.
    pub fn compute_all(&self) -> BTreeMap<String, f64> {
        self.intersection
            .iter()
            .map(|(mark, intersection)| {
                let sum = self.sum.get(mark).copied().unwrap_or(0.0);
                let value = if sum == *intersection {
                    0.0
                } else {
                    python_round_6(intersection / (sum - intersection))
                };
                (mark.clone(), value)
            })
            .collect()
    }

    /// Computes string-mode IoU.
    ///
    /// # Errors
    ///
    /// Returns Python's `ZeroDivisionError` when a present mark has a zero
    /// denominator.
    pub fn compute_one(&self, phoneme: &str) -> Result<Option<f64>, HfaMetricError> {
        let Some(intersection) = self.intersection.get(phoneme).copied() else {
            return Ok(None);
        };
        let denominator = self.sum.get(phoneme).copied().unwrap_or(0.0) - intersection;
        if denominator == 0.0 {
            Err(HfaMetricError::zero_division())
        } else {
            Ok(Some(python_round_6(intersection / denominator)))
        }
    }

    /// Computes list-mode IoU in caller-provided order.
    pub fn compute_list(
        &self,
        phonemes: &[String],
    ) -> Vec<(String, Result<Option<f64>, HfaMetricError>)> {
        phonemes
            .iter()
            .map(|phoneme| (phoneme.clone(), self.compute_one(phoneme)))
            .collect()
    }

    /// Resets accumulated maps.
    pub fn reset(&mut self) {
        self.intersection.clear();
        self.sum.clear();
    }
}

/// Returns LCS index pairs using the legacy target-side decrement tie policy.
pub fn compute_lcs_matches(pred: &PointTier, target: &PointTier) -> Vec<(usize, usize)> {
    let m = pred.len();
    let n = target.len();
    let mut dp = vec![vec![0_usize; n + 1]; m + 1];

    for i in 1..=m {
        for j in 1..=n {
            if pred.points[i - 1].mark == target.points[j - 1].mark {
                dp[i][j] = dp[i - 1][j - 1] + 1;
            } else {
                dp[i][j] = dp[i - 1][j].max(dp[i][j - 1]);
            }
        }
    }

    let mut i = m;
    let mut j = n;
    let mut matches = Vec::new();
    while i > 0 && j > 0 {
        if pred.points[i - 1].mark == target.points[j - 1].mark {
            matches.push((i - 1, j - 1));
            i -= 1;
            j -= 1;
        } else if dp[i - 1][j] > dp[i][j - 1] {
            i -= 1;
        } else {
            j -= 1;
        }
    }
    matches.reverse();
    matches
}

/// Returns matched point lists from `compute_lcs_matches`.
pub fn get_matched_pairs(pred: &PointTier, target: &PointTier) -> (Vec<Point>, Vec<Point>) {
    let matches = compute_lcs_matches(pred, target);
    let pred_matched = matches
        .iter()
        .map(|(i, _)| pred.points[*i].clone())
        .collect();
    let target_matched = matches
        .iter()
        .map(|(_, j)| target.points[*j].clone())
        .collect();
    (pred_matched, target_matched)
}

/// Boundary edit distance metric.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct BoundaryEditDistance {
    pub distance: f64,
    pub phonemes: usize,
    pub error_phonemes: usize,
}

impl BoundaryEditDistance {
    /// Creates an empty distance metric.
    pub fn new() -> Self {
        Self::default()
    }

    /// Updates distance and phoneme counters.
    ///
    /// Returns `false` for equal-length label mismatches after preserving the
    /// previous state, matching the legacy Python early return.
    pub fn update(&mut self, pred: &PointTier, target: &PointTier) -> bool {
        let pred_points: Vec<Point>;
        let target_points: Vec<Point>;
        let (pred_slice, target_slice): (&[Point], &[Point]) = if pred.len() != target.len() {
            let (pred_lcs, target_lcs) = get_matched_pairs(pred, target);
            self.error_phonemes += pred_lcs.len().abs_diff(target.len());
            pred_points = pred_lcs;
            target_points = target_lcs;
            (&pred_points, &target_points)
        } else {
            (&pred.points, &target.points)
        };

        for (pred_point, target_point) in pred_slice.iter().zip(target_slice) {
            if pred_point.mark != target_point.mark {
                return false;
            }
        }

        self.distance += absolute_difference_sum(pred_slice, target_slice);
        self.phonemes += target_slice.len();
        true
    }

    /// Returns rounded accumulated distance.
    pub fn compute(&self) -> f64 {
        python_round_6(self.distance)
    }

    /// Resets distance and phoneme counters, preserving `error_phonemes`.
    pub const fn reset(&mut self) {
        self.distance = 0.0;
        self.phonemes = 0;
    }
}

/// Boundary edit ratio metric.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct BoundaryEditRatio {
    pub distance_metric: BoundaryEditDistance,
    pub duration: f64,
}

impl BoundaryEditRatio {
    /// Creates an empty ratio metric.
    pub fn new() -> Self {
        Self::default()
    }

    /// Updates distance and target duration.
    ///
    /// # Errors
    ///
    /// Returns Python's `IndexError` when the nested distance update succeeds
    /// for an empty target tier.
    pub fn update(&mut self, pred: &PointTier, target: &PointTier) -> Result<(), HfaMetricError> {
        if self.distance_metric.update(pred, target) {
            let first = target
                .points
                .first()
                .ok_or_else(HfaMetricError::index_error)?;
            let last = target
                .points
                .last()
                .ok_or_else(HfaMetricError::index_error)?;
            self.duration += last.time - first.time;
        }
        Ok(())
    }

    /// Returns rounded distance/duration or the legacy zero-duration default.
    pub fn compute(&self) -> f64 {
        if self.duration == 0.0 {
            1.0
        } else {
            python_round_6(self.distance_metric.compute() / self.duration)
        }
    }

    /// Legacy inherited reset gap.
    pub fn reset(&mut self) -> Result<(), HfaMetricError> {
        Err(HfaMetricError::not_implemented())
    }
}

/// Weighted boundary edit ratio metric.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct BoundaryEditRatioWeighted {
    pub distance_metric: BoundaryEditDistance,
    pub duration: f64,
    pub counts: usize,
    pub error: usize,
}

impl BoundaryEditRatioWeighted {
    /// Creates an empty weighted ratio metric.
    pub fn new() -> Self {
        Self::default()
    }

    /// Updates count, nested distance, duration, and mismatch count.
    ///
    /// # Errors
    ///
    /// Returns Python's `IndexError` when the nested distance update succeeds
    /// for an empty target tier.
    pub fn update(&mut self, pred: &PointTier, target: &PointTier) -> Result<(), HfaMetricError> {
        self.counts += 1;
        if self.distance_metric.update(pred, target) {
            let first = target
                .points
                .first()
                .ok_or_else(HfaMetricError::index_error)?;
            let last = target
                .points
                .last()
                .ok_or_else(HfaMetricError::index_error)?;
            self.duration += last.time - first.time;
        } else {
            self.error += 1;
        }
        Ok(())
    }

    /// Returns weighted ratio or the legacy zero denominator defaults.
    pub fn compute(&self) -> f64 {
        if self.duration == 0.0 || self.distance_metric.phonemes == 0 || self.counts == 0 {
            return 1.0;
        }

        let correction =
            1.0 - self.distance_metric.error_phonemes as f64 / self.distance_metric.phonemes as f64;
        let penalty = (self.error as f64 / self.counts as f64) * 0.2;
        if correction + penalty == 0.0 {
            return 1.0;
        }

        python_round_6((self.distance_metric.compute() / self.duration) / correction + penalty)
    }

    /// Legacy inherited reset gap.
    pub fn reset(&mut self) -> Result<(), HfaMetricError> {
        Err(HfaMetricError::not_implemented())
    }
}

fn absolute_difference_sum(pred: &[Point], target: &[Point]) -> f64 {
    let pred_times = Array1::from_iter(pred.iter().map(|point| point.time));
    let target_times = Array1::from_iter(target.iter().map(|point| point.time));
    (&pred_times - &target_times).mapv(f64::abs).sum()
}

fn python_round_6(value: f64) -> f64 {
    let scale = 1_000_000.0;
    let scaled = value * scale;
    if !scaled.is_finite() {
        return value;
    }

    let truncated = scaled.trunc();
    let fractional = (scaled - truncated).abs();
    let tie_epsilon = f64::EPSILON * scaled.abs().max(1.0) * 4.0;
    let rounded = if (fractional - 0.5).abs() <= tie_epsilon {
        if (truncated.abs() % 2.0) == 0.0 {
            truncated
        } else {
            truncated + scaled.signum()
        }
    } else if fractional > 0.5 {
        truncated + scaled.signum()
    } else {
        truncated
    };
    rounded / scale
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Map, Value, json};

    const FIXTURES: &str = include_str!("../../../../fixtures/hfa_metrics_core.jsonl");

    fn point_from_value(value: &Value) -> Point {
        Point::new(
            value.get("time").and_then(Value::as_f64).unwrap(),
            value.get("mark").and_then(Value::as_str).unwrap(),
        )
    }

    fn tier_from_value(value: &Value) -> PointTier {
        PointTier::from_points(value.as_array().unwrap().iter().map(point_from_value))
    }

    fn point_value(point: &Point) -> Value {
        json!({"time": point.time, "mark": point.mark})
    }

    fn points_value(points: &[Point]) -> Value {
        Value::Array(points.iter().map(point_value).collect())
    }

    fn metric_map_value(map: &BTreeMap<String, f64>) -> Value {
        let mut object = Map::new();
        for (key, value) in map {
            object.insert(key.clone(), json!(value));
        }
        Value::Object(object)
    }

    fn boundary_state_value(metric: &BoundaryEditDistance) -> Value {
        json!({
            "distance": metric.distance,
            "phonemes": metric.phonemes,
            "error_phonemes": metric.error_phonemes,
        })
    }

    fn error_value(error: &HfaMetricError) -> Value {
        json!({
            "type": error.exception_type(),
            "message": error.message(),
        })
    }

    fn project_unit_result(result: Result<(), HfaMetricError>) -> Value {
        match result {
            Ok(()) => json!({"ok": Value::Null}),
            Err(error) => json!({"error": error_value(&error)}),
        }
    }

    fn project_optional_float_result(result: Result<Option<f64>, HfaMetricError>) -> Value {
        match result {
            Ok(Some(value)) => json!({"ok": value}),
            Ok(None) => json!({"ok": Value::Null}),
            Err(error) => json!({"error": error_value(&error)}),
        }
    }

    fn run_custom_point_order(case: &Value) -> Value {
        let mut tier = PointTier::new();
        let mut observations = Vec::new();
        for item in case.get("points").and_then(Value::as_array).unwrap() {
            tier.add_point(point_from_value(item));
            observations.push(points_value(&tier.points));
        }
        json!({
            "observations": observations,
            "final": points_value(&tier.points),
        })
    }

    fn run_vlabeler_count(case: &Value) -> Value {
        let mut metric = VlabelerEditsCount::new(
            case.get("move_min_frames")
                .and_then(Value::as_i64)
                .unwrap_or(1),
            case.get("move_max_frames")
                .and_then(Value::as_i64)
                .unwrap_or(2),
        );
        let mut observations = Vec::new();
        for operation in case.get("operations").and_then(Value::as_array).unwrap() {
            match operation.get("op").and_then(Value::as_str).unwrap() {
                "update" => {
                    metric.update(
                        &tier_from_value(operation.get("pred").unwrap()),
                        &tier_from_value(operation.get("target").unwrap()),
                    );
                    observations.push(json!({"op": "update", "compute": metric.compute()}));
                }
                "compute" => {
                    observations.push(json!({"op": "compute", "compute": metric.compute()}));
                }
                "reset" => {
                    metric.reset();
                    observations.push(json!({"op": "reset", "compute": metric.compute()}));
                }
                other => panic!("unknown operation {other:?}"),
            }
        }
        Value::Array(observations)
    }

    fn run_vlabeler_ratio(case: &Value) -> Value {
        let mut metric = VlabelerEditRatio::new(
            case.get("move_min_frames")
                .and_then(Value::as_i64)
                .unwrap_or(1),
            case.get("move_max_frames")
                .and_then(Value::as_i64)
                .unwrap_or(2),
        );
        let mut observations = Vec::new();
        for operation in case.get("operations").and_then(Value::as_array).unwrap() {
            match operation.get("op").and_then(Value::as_str).unwrap() {
                "update" => {
                    metric.update(
                        &tier_from_value(operation.get("pred").unwrap()),
                        &tier_from_value(operation.get("target").unwrap()),
                    );
                    observations.push(json!({
                        "op": "update",
                        "distance": metric.edit_distance.compute(),
                        "total": metric.total,
                        "compute": metric.compute(),
                    }));
                }
                "compute" => {
                    observations.push(json!({"op": "compute", "compute": metric.compute()}));
                }
                "reset" => {
                    metric.reset();
                    observations.push(json!({
                        "op": "reset",
                        "distance": metric.edit_distance.compute(),
                        "total": metric.total,
                        "compute": metric.compute(),
                    }));
                }
                other => panic!("unknown operation {other:?}"),
            }
        }
        Value::Array(observations)
    }

    fn run_iou(case: &Value) -> Value {
        let mut metric = IntersectionOverUnion::new();
        let mut observations = Vec::new();
        for operation in case.get("operations").and_then(Value::as_array).unwrap() {
            match operation.get("op").and_then(Value::as_str).unwrap() {
                "update" => {
                    metric.update(
                        &tier_from_value(operation.get("pred").unwrap()),
                        &tier_from_value(operation.get("target").unwrap()),
                    );
                    observations.push(json!({
                        "op": "update",
                        "intersection": metric_map_value(&metric.intersection),
                        "sum": metric_map_value(&metric.sum),
                    }));
                }
                "compute" => {
                    let request = operation.get("request");
                    let mut object = Map::new();
                    object.insert("op".to_string(), json!("compute"));
                    object.insert(
                        "request".to_string(),
                        request.cloned().unwrap_or(Value::Null),
                    );
                    match request
                        .and_then(Value::as_object)
                        .and_then(|object| object.get("$kind"))
                        .and_then(Value::as_str)
                    {
                        None => {
                            object
                                .insert("ok".to_string(), metric_map_value(&metric.compute_all()));
                        }
                        Some("str") => {
                            let phoneme = request
                                .unwrap()
                                .get("value")
                                .and_then(Value::as_str)
                                .unwrap();
                            if let Value::Object(result) =
                                project_optional_float_result(metric.compute_one(phoneme))
                            {
                                object.extend(result);
                            }
                        }
                        Some("list") => {
                            let items = request
                                .unwrap()
                                .get("items")
                                .and_then(Value::as_array)
                                .unwrap();
                            let mut result = Map::new();
                            let phonemes = items
                                .iter()
                                .map(|item| item.as_str().unwrap().to_string())
                                .collect::<Vec<_>>();
                            for (phoneme, value) in metric.compute_list(&phonemes) {
                                match value {
                                    Ok(Some(value)) => {
                                        result.insert(phoneme, json!(value));
                                    }
                                    Ok(None) => {
                                        result.insert(phoneme, Value::Null);
                                    }
                                    Err(error) => {
                                        panic!("unexpected list-mode IoU error: {error}");
                                    }
                                }
                            }
                            object.insert("ok".to_string(), Value::Object(result));
                        }
                        Some(other) => panic!("unknown compute request {other:?}"),
                    }
                    observations.push(Value::Object(object));
                }
                "reset" => {
                    metric.reset();
                    observations.push(json!({
                        "op": "reset",
                        "intersection": metric_map_value(&metric.intersection),
                        "sum": metric_map_value(&metric.sum),
                    }));
                }
                other => panic!("unknown operation {other:?}"),
            }
        }
        Value::Array(observations)
    }

    fn run_lcs(case: &Value) -> Value {
        let pred = tier_from_value(case.get("pred").unwrap());
        let target = tier_from_value(case.get("target").unwrap());
        let (pred_matched, target_matched) = get_matched_pairs(&pred, &target);
        json!({
            "matches": compute_lcs_matches(&pred, &target)
                .into_iter()
                .map(|(i, j)| json!([i, j]))
                .collect::<Vec<_>>(),
            "pred_matched": points_value(&pred_matched),
            "target_matched": points_value(&target_matched),
        })
    }

    fn run_boundary_distance(case: &Value) -> Value {
        let mut metric = BoundaryEditDistance::new();
        let mut observations = Vec::new();
        for operation in case.get("operations").and_then(Value::as_array).unwrap() {
            match operation.get("op").and_then(Value::as_str).unwrap() {
                "update" => {
                    let ok = metric.update(
                        &tier_from_value(operation.get("pred").unwrap()),
                        &tier_from_value(operation.get("target").unwrap()),
                    );
                    observations.push(json!({
                        "op": "update",
                        "ok": ok,
                        "state": boundary_state_value(&metric),
                        "compute": metric.compute(),
                    }));
                }
                "compute" => {
                    observations.push(json!({"op": "compute", "compute": metric.compute()}));
                }
                "reset" => {
                    metric.reset();
                    observations.push(json!({
                        "op": "reset",
                        "state": boundary_state_value(&metric),
                        "compute": metric.compute(),
                    }));
                }
                other => panic!("unknown operation {other:?}"),
            }
        }
        Value::Array(observations)
    }

    fn run_boundary_ratio(case: &Value) -> Value {
        let mut metric = BoundaryEditRatio::new();
        let mut observations = Vec::new();
        for operation in case.get("operations").and_then(Value::as_array).unwrap() {
            match operation.get("op").and_then(Value::as_str).unwrap() {
                "update" => {
                    let mut object = match project_unit_result(metric.update(
                        &tier_from_value(operation.get("pred").unwrap()),
                        &tier_from_value(operation.get("target").unwrap()),
                    )) {
                        Value::Object(object) => object,
                        _ => unreachable!(),
                    };
                    object.insert("op".to_string(), json!("update"));
                    object.insert("duration".to_string(), json!(metric.duration));
                    object.insert(
                        "distance_state".to_string(),
                        boundary_state_value(&metric.distance_metric),
                    );
                    object.insert("compute".to_string(), json!(metric.compute()));
                    observations.push(Value::Object(object));
                }
                "compute" => {
                    observations.push(json!({"op": "compute", "compute": metric.compute()}));
                }
                "reset" => {
                    let mut object = match project_unit_result(metric.reset()) {
                        Value::Object(object) => object,
                        _ => unreachable!(),
                    };
                    object.insert("op".to_string(), json!("reset"));
                    observations.push(Value::Object(object));
                }
                other => panic!("unknown operation {other:?}"),
            }
        }
        Value::Array(observations)
    }

    fn run_boundary_ratio_weighted(case: &Value) -> Value {
        let mut metric = BoundaryEditRatioWeighted::new();
        let mut observations = Vec::new();
        for operation in case.get("operations").and_then(Value::as_array).unwrap() {
            match operation.get("op").and_then(Value::as_str).unwrap() {
                "update" => {
                    let mut object = match project_unit_result(metric.update(
                        &tier_from_value(operation.get("pred").unwrap()),
                        &tier_from_value(operation.get("target").unwrap()),
                    )) {
                        Value::Object(object) => object,
                        _ => unreachable!(),
                    };
                    object.insert("op".to_string(), json!("update"));
                    object.insert("duration".to_string(), json!(metric.duration));
                    object.insert(
                        "distance_state".to_string(),
                        boundary_state_value(&metric.distance_metric),
                    );
                    object.insert("compute".to_string(), json!(metric.compute()));
                    object.insert("counts".to_string(), json!(metric.counts));
                    object.insert("error".to_string(), json!(metric.error));
                    observations.push(Value::Object(object));
                }
                "compute" => {
                    observations.push(json!({"op": "compute", "compute": metric.compute()}));
                }
                "reset" => {
                    let mut object = match project_unit_result(metric.reset()) {
                        Value::Object(object) => object,
                        _ => unreachable!(),
                    };
                    object.insert("op".to_string(), json!("reset"));
                    observations.push(Value::Object(object));
                }
                other => panic!("unknown operation {other:?}"),
            }
        }
        Value::Array(observations)
    }

    fn run_case(case: &Value) -> Value {
        match case.get("kind").and_then(Value::as_str).unwrap() {
            "custom_point_order" => run_custom_point_order(case),
            "vlabeler_count" => run_vlabeler_count(case),
            "vlabeler_ratio" => run_vlabeler_ratio(case),
            "iou" => run_iou(case),
            "lcs" => run_lcs(case),
            "boundary_distance" => run_boundary_distance(case),
            "boundary_ratio" => run_boundary_ratio(case),
            "boundary_ratio_weighted" => run_boundary_ratio_weighted(case),
            other => panic!("unknown fixture kind {other:?}"),
        }
    }

    fn assert_json_close(case_id: &str, path: &str, actual: &Value, expected: &Value) {
        match (actual, expected) {
            (Value::Number(actual), Value::Number(expected)) => {
                let actual = actual.as_f64().unwrap();
                let expected = expected.as_f64().unwrap();
                assert!(
                    (actual - expected).abs() <= 1e-12,
                    "{case_id} {path}: {actual:?} != {expected:?}"
                );
            }
            (Value::Array(actual), Value::Array(expected)) => {
                assert_eq!(
                    actual.len(),
                    expected.len(),
                    "{case_id} {path}: array lengths differ"
                );
                for (index, (actual_item, expected_item)) in actual.iter().zip(expected).enumerate()
                {
                    assert_json_close(
                        case_id,
                        &format!("{path}[{index}]"),
                        actual_item,
                        expected_item,
                    );
                }
            }
            (Value::Object(actual), Value::Object(expected)) => {
                assert_eq!(
                    actual.len(),
                    expected.len(),
                    "{case_id} {path}: object lengths differ"
                );
                for (key, expected_value) in expected {
                    let actual_value = actual
                        .get(key)
                        .unwrap_or_else(|| panic!("{case_id} {path}: missing key {key:?}"));
                    assert_json_close(
                        case_id,
                        &format!("{path}.{key}"),
                        actual_value,
                        expected_value,
                    );
                }
            }
            _ => {
                assert_eq!(actual, expected, "{case_id} {path}");
            }
        }
    }

    #[test]
    fn hfa_metrics_core_fixture_parity() {
        for line in FIXTURES.lines().filter(|line| !line.is_empty()) {
            let case: Value = serde_json::from_str(line).unwrap();
            assert_json_close(
                case.get("case_id").and_then(Value::as_str).unwrap(),
                "$",
                &run_case(&case),
                case.get("expect").unwrap(),
            );
        }
    }

    #[test]
    fn iou_compute_list_preserves_request_order_and_duplicates() {
        let mut metric = IntersectionOverUnion::new();
        metric.update(
            &PointTier::from_points([
                Point::new(0.0, "a"),
                Point::new(1.0, "b"),
                Point::new(2.0, "a"),
                Point::new(3.0, "end"),
            ]),
            &PointTier::from_points([
                Point::new(0.5, "a"),
                Point::new(1.5, "c"),
                Point::new(2.0, "a"),
                Point::new(3.0, "end"),
            ]),
        );

        let requested = ["c", "a", "missing", "a", "b"]
            .into_iter()
            .map(str::to_string)
            .collect::<Vec<_>>();
        let actual = metric.compute_list(&requested);
        let actual_keys = actual
            .iter()
            .map(|(phoneme, _)| phoneme.as_str())
            .collect::<Vec<_>>();
        assert_eq!(actual_keys, ["c", "a", "missing", "a", "b"]);
        assert_eq!(actual[0].1, Ok(Some(0.0)));
        assert_eq!(actual[1].1, Ok(Some(0.6)));
        assert_eq!(actual[2].1, Ok(None));
        assert_eq!(actual[3].1, Ok(Some(0.6)));
        assert_eq!(actual[4].1, Ok(Some(0.0)));
    }
}
