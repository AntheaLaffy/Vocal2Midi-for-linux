//! SciPy `signal.resample_poly` compatibility for ASR audio.
//!
//! This module implements the fixture-backed default 1D `float32` path used by
//! the legacy Qwen and Romaji ASR loaders. It is not wired into Python runtime
//! loading and does not own audio file IO.

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
/// Python-compatible validation failure for polyphase resampling.
pub struct ResamplePolyError {
    /// The Python-compatible error type.
    pub error_type: &'static str,
    /// The message text.
    pub message: String,
}

impl ResamplePolyError {
    fn value_error(message: impl Into<String>) -> Self {
        Self {
            error_type: "ValueError",
            message: message.into(),
        }
    }
}

impl fmt::Display for ResamplePolyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.error_type, self.message)
    }
}

impl std::error::Error for ResamplePolyError {}

/// Resamples one `f32` signal with SciPy's default polyphase FIR policy.
///
/// # Errors
///
/// Returns [`ResamplePolyError`] when either sample rate is less than one.
pub fn resample_poly_1d_float32(
    input: &[f32],
    target_rate: i64,
    source_rate: i64,
) -> Result<Vec<f32>, ResamplePolyError> {
    if target_rate < 1 || source_rate < 1 {
        return Err(ResamplePolyError::value_error("up and down must be >= 1"));
    }

    let gcd = gcd_i64(target_rate, source_rate);
    let up = (target_rate / gcd) as usize;
    let down = (source_rate / gcd) as usize;

    if up == 1 && down == 1 {
        return Ok(input.to_vec());
    }
    if input.is_empty() {
        return Ok(Vec::new());
    }

    let n_out = ceil_div(input.len() * up, down);
    let max_rate = up.max(down);
    let cutoff = 1.0 / max_rate as f64;
    let half_len = 10 * max_rate;
    let mut h = firwin_lowpass_kaiser(2 * half_len + 1, cutoff);
    for coefficient in &mut h {
        *coefficient *= up as f32;
    }

    let n_pre_pad = down - (half_len % down);
    let mut n_post_pad = 0usize;
    let n_pre_remove = (half_len + n_pre_pad) / down;
    while output_len(h.len() + n_pre_pad + n_post_pad, input.len(), up, down) < n_out + n_pre_remove
    {
        n_post_pad += 1;
    }

    let mut padded_h = Vec::with_capacity(n_pre_pad + h.len() + n_post_pad);
    padded_h.extend(std::iter::repeat_n(0.0, n_pre_pad));
    padded_h.extend(h);
    padded_h.extend(std::iter::repeat_n(0.0, n_post_pad));

    let y = upfirdn_constant_zero(&padded_h, input, up, down);
    let end = n_pre_remove + n_out;
    Ok(y[n_pre_remove..end].to_vec())
}

fn gcd_i64(mut left: i64, mut right: i64) -> i64 {
    while right != 0 {
        let remainder = left % right;
        left = right;
        right = remainder;
    }
    left.abs()
}

fn ceil_div(numerator: usize, denominator: usize) -> usize {
    numerator / denominator + usize::from(numerator % denominator != 0)
}

fn output_len(len_h: usize, in_len: usize, up: usize, down: usize) -> usize {
    (((in_len - 1) * up + len_h) - 1) / down + 1
}

fn firwin_lowpass_kaiser(numtaps: usize, cutoff: f64) -> Vec<f32> {
    let alpha = 0.5 * (numtaps - 1) as f64;
    let mut h = Vec::with_capacity(numtaps);
    for index in 0..numtaps {
        let m = index as f64 - alpha;
        let value = cutoff * sinc(cutoff * m) * kaiser_weight(numtaps, index, 5.0);
        h.push(value);
    }

    let scale = h.iter().copied().sum::<f64>();
    h.into_iter().map(|value| (value / scale) as f32).collect()
}

fn sinc(value: f64) -> f64 {
    if value == 0.0 {
        1.0
    } else {
        let x = std::f64::consts::PI * value;
        x.sin() / x
    }
}

fn kaiser_weight(len: usize, index: usize, beta: f64) -> f64 {
    if len <= 1 {
        return 1.0;
    }
    let alpha = (len - 1) as f64 / 2.0;
    let normalized = (index as f64 - alpha) / alpha;
    let argument = beta * (1.0 - normalized * normalized).sqrt();
    bessel_i0(argument) / bessel_i0(beta)
}

fn bessel_i0(value: f64) -> f64 {
    let y = value * value / 4.0;
    let mut sum = 1.0;
    let mut term = 1.0;
    for k in 1..=64 {
        let k = k as f64;
        term *= y / (k * k);
        sum += term;
        if term.abs() <= sum.abs() * 1e-16 {
            break;
        }
    }
    sum
}

fn upfirdn_constant_zero(h: &[f32], x: &[f32], up: usize, down: usize) -> Vec<f32> {
    let h_trans_flip = pad_h_transposed_flipped(h, up);
    let len_out = output_len(h.len(), x.len(), up, down);
    let mut out = vec![0.0f32; len_out];
    apply_upfirdn_constant_zero(x, &h_trans_flip, &mut out, up, down);
    out
}

fn pad_h_transposed_flipped(h: &[f32], up: usize) -> Vec<f32> {
    let h_pad_len = h.len() + ((up - (h.len() % up)) % up);
    let rows = h_pad_len / up;
    let mut result = Vec::with_capacity(h_pad_len);
    for phase in 0..up {
        for row in (0..rows).rev() {
            let index = row * up + phase;
            result.push(h.get(index).copied().unwrap_or(0.0));
        }
    }
    result
}

fn apply_upfirdn_constant_zero(
    x: &[f32],
    h_trans_flip: &[f32],
    out: &mut [f32],
    up: usize,
    down: usize,
) {
    let h_per_phase = h_trans_flip.len() / up;
    let padded_len = x.len() + h_per_phase - 1;
    let mut x_idx = 0usize;
    let mut y_idx = 0usize;
    let mut phase = 0usize;

    if out.is_empty() {
        return;
    }

    while x_idx < x.len() {
        let mut h_idx = phase * h_per_phase;
        let x_conv_idx = x_idx as isize - h_per_phase as isize + 1;
        let start = if x_conv_idx < 0 {
            h_idx += (-x_conv_idx) as usize;
            0
        } else {
            x_conv_idx as usize
        };
        for (sample, coefficient) in x
            .iter()
            .take(x_idx + 1)
            .skip(start)
            .zip(h_trans_flip.iter().skip(h_idx))
        {
            out[y_idx] += *sample * *coefficient;
        }

        y_idx += 1;
        if y_idx >= out.len() {
            return;
        }
        phase += down;
        x_idx += phase / up;
        phase %= up;
    }

    while x_idx < padded_len {
        let x_conv_idx = x_idx as isize - h_per_phase as isize + 1;
        for (h_idx, sample_index) in (phase * h_per_phase..).zip(x_conv_idx..=x_idx as isize) {
            if (0..x.len() as isize).contains(&sample_index) {
                out[y_idx] += x[sample_index as usize] * h_trans_flip[h_idx];
            }
        }
        y_idx += 1;
        if y_idx >= out.len() {
            return;
        }
        phase += down;
        x_idx += phase / up;
        phase %= up;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    const FIXTURES: &str = include_str!("../../../../fixtures/asr_resample_poly_contract.jsonl");
    const VALUE_TOLERANCE: f64 = 2e-7;
    const SUM_TOLERANCE: f64 = 1e-5;

    #[test]
    fn asr_resample_poly_contract_matches_fixtures() {
        for (index, line) in FIXTURES.lines().filter(|line| !line.is_empty()).enumerate() {
            let case: Value = serde_json::from_str(line).unwrap();
            let input = fixture_input(&case);
            let target_rate = case["target_rate"].as_i64().unwrap();
            let source_rate = case["source_rate"].as_i64().unwrap();
            let actual = resample_poly_1d_float32(&input, target_rate, source_rate);
            let expected = &case["expected"];

            if expected["ok"].as_bool().unwrap() {
                let actual = actual.unwrap_or_else(|err| panic!("case {index} failed: {err}"));
                assert_eq!(
                    actual.len(),
                    expected["shape"][0].as_u64().unwrap() as usize,
                    "case {index}"
                );
                if let Some(values) = expected["values"].as_array() {
                    for (sample_index, (actual, expected)) in
                        actual.iter().zip(values.iter()).enumerate()
                    {
                        assert_float_close(
                            *actual as f64,
                            json_float(expected),
                            VALUE_TOLERANCE,
                            &format!("case {index} sample {sample_index}"),
                        );
                    }
                } else {
                    for selected in expected["selected_values"].as_array().unwrap() {
                        let sample_index = selected[0].as_u64().unwrap() as usize;
                        assert_float_close(
                            actual[sample_index] as f64,
                            json_float(&selected[1]),
                            VALUE_TOLERANCE,
                            &format!("case {index} sample {sample_index}"),
                        );
                    }
                    let finite_sum = actual
                        .iter()
                        .copied()
                        .filter(|value| value.is_finite())
                        .map(f64::from)
                        .sum::<f64>();
                    let finite_abs_sum = actual
                        .iter()
                        .copied()
                        .filter(|value| value.is_finite())
                        .map(|value| f64::from(value.abs()))
                        .sum::<f64>();
                    assert_float_close(
                        finite_sum,
                        json_float(&expected["finite_sum"]),
                        SUM_TOLERANCE,
                        &format!("case {index} finite_sum"),
                    );
                    assert_float_close(
                        finite_abs_sum,
                        json_float(&expected["finite_abs_sum"]),
                        SUM_TOLERANCE,
                        &format!("case {index} finite_abs_sum"),
                    );
                }
            } else {
                let err = actual.expect_err("expected fixture error");
                assert_eq!(err.error_type, expected["error_type"].as_str().unwrap());
                assert_eq!(err.message, expected["message"].as_str().unwrap());
            }
        }
    }

    fn fixture_input(case: &Value) -> Vec<f32> {
        if let Some(values) = case.get("input").and_then(Value::as_array) {
            return values
                .iter()
                .map(json_float)
                .map(|value| value as f32)
                .collect();
        }

        let spec = &case["input_spec"];
        assert_eq!(spec["kind"].as_str().unwrap(), "dual_sine");
        let len = spec["length"].as_u64().unwrap() as usize;
        let source_rate = spec["source_rate"].as_f64().unwrap() as f32;
        let mut input = vec![0.0f32; len];
        for component in spec["components"].as_array().unwrap() {
            let phase_step =
                (2.0 * std::f64::consts::PI * component["frequency"].as_f64().unwrap()) as f32;
            let amplitude = component["amplitude"].as_f64().unwrap() as f32;
            for (sample_index, sample) in input.iter_mut().enumerate() {
                let t = sample_index as f32 / source_rate;
                let angle = phase_step * t;
                *sample += amplitude * (angle as f64).sin() as f32;
            }
        }
        input
    }

    fn json_float(value: &Value) -> f64 {
        match value {
            Value::String(value) if value == "nan" => f64::NAN,
            Value::String(value) if value == "inf" => f64::INFINITY,
            Value::String(value) if value == "-inf" => f64::NEG_INFINITY,
            value => value.as_f64().unwrap(),
        }
    }

    fn assert_float_close(actual: f64, expected: f64, tolerance: f64, label: &str) {
        if actual.is_nan() || expected.is_nan() {
            assert!(
                actual.is_nan() && expected.is_nan(),
                "{label}: actual {actual:?}, expected {expected:?}"
            );
            return;
        }
        if actual.is_infinite() || expected.is_infinite() {
            assert_eq!(actual, expected, "{label}");
            return;
        }
        assert!(
            (actual - expected).abs() <= tolerance,
            "{label}: actual {actual:?}, expected {expected:?}, tolerance {tolerance}"
        );
    }
}
