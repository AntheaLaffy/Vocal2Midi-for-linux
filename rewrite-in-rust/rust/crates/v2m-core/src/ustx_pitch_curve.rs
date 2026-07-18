//! USTX pitch-deviation curve rendering.
//!
//! This module mirrors the deterministic `_build_pitd_curve` path in
//! `inference/API/ustx_api.py` for fixture-backed parity only. Python remains
//! the runtime owner for RMVPE model execution, waveform preprocessing, USTX
//! YAML assembly, and filesystem writes.

const U_CURVE_INTERVAL: i64 = 5;

/// One note row accepted by the USTX pitch-curve renderer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UstxPitchNote {
    /// The onset.
    pub onset: f64,
    /// The offset.
    pub offset: f64,
    /// The pitch.
    pub pitch: f64,
}

/// Rendered pitch-deviation curve points.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UstxPitchCurve {
    /// The ordered xs.
    pub xs: Vec<i64>,
    /// The ordered ys.
    pub ys: Vec<i64>,
}

/// Recoverable USTX pitch-curve rendering failures at the Rust boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UstxPitchCurveError {
    /// Represents the Python-compatible invalid tempo case.
    InvalidTempo,
    /// Represents the Python-compatible invalid time step case.
    InvalidTimeStep,
    /// Represents the Python-compatible tick out of range case.
    TickOutOfRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PitchPoint {
    x: i64,
    y: i64,
}

/// Renders Python-compatible `pitd` curve vectors from synthetic RMVPE output.
///
/// # Errors
///
/// Returns an error when `tempo`, `time_step_seconds`, or an intermediate tick
/// conversion cannot be represented as a finite integer.
pub fn render_ustx_pitch_curve(
    notes: &[UstxPitchNote],
    midi_pitch: &[f64],
    time_step_seconds: f64,
    tempo: f64,
) -> Result<UstxPitchCurve, UstxPitchCurveError> {
    if !tempo.is_finite() {
        return Err(UstxPitchCurveError::InvalidTempo);
    }
    if !time_step_seconds.is_finite() {
        return Err(UstxPitchCurveError::InvalidTimeStep);
    }
    if notes.is_empty() || midi_pitch.is_empty() {
        return Ok(UstxPitchCurve {
            xs: Vec::new(),
            ys: Vec::new(),
        });
    }

    let mut sorted_notes = notes.to_vec();
    sorted_notes.sort_by(|left, right| {
        left.onset
            .partial_cmp(&right.onset)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut xs = Vec::new();
    let mut ys = Vec::new();
    let mut pending = Vec::new();
    let mut pending_note_idx = None;
    let mut note_idx = 0usize;

    for (index, midi_pitch) in midi_pitch.iter().enumerate() {
        let t = index as f64 * time_step_seconds;
        while note_idx + 1 < sorted_notes.len() && sorted_notes[note_idx].offset <= t {
            note_idx += 1;
        }

        let note = &sorted_notes[note_idx];
        if !pending.is_empty() && pending_note_idx != Some(note_idx) {
            append_smoothed_points(&mut xs, &mut ys, &pending);
            pending.clear();
            pending_note_idx = None;
        }

        if !(note.onset <= t && t < note.offset) {
            continue;
        }
        if midi_pitch.is_nan() {
            continue;
        }

        let duration = 0.0_f64.max(note.offset - note.onset);
        let note_offset = t - note.onset;
        let edge_trim = 0.025_f64.min(duration * 0.15);
        if duration > edge_trim * 2.0
            && (note_offset < edge_trim || duration - note_offset <= edge_trim)
        {
            continue;
        }

        let tick = to_ticks(t, tempo)?;
        let x = round_half_even_i64(tick as f64 / U_CURVE_INTERVAL as f64)? * U_CURVE_INTERVAL;
        let y = round_half_even_i64(((*midi_pitch - note.pitch) * 100.0).clamp(-1200.0, 1200.0))?;
        let point = PitchPoint { x, y };
        pending_note_idx = Some(note_idx);
        if let Some(last) = pending.last_mut()
            && last.x == point.x
        {
            *last = point;
            continue;
        }
        pending.push(point);
    }

    append_smoothed_points(&mut xs, &mut ys, &pending);
    Ok(UstxPitchCurve { xs, ys })
}

fn to_ticks(seconds: f64, tempo: f64) -> Result<i64, UstxPitchCurveError> {
    round_half_even_i64(seconds * tempo * 8.0)
}

fn round_half_even_i64(value: f64) -> Result<i64, UstxPitchCurveError> {
    if !value.is_finite() || value < i64::MIN as f64 || value > i64::MAX as f64 {
        return Err(UstxPitchCurveError::TickOutOfRange);
    }

    let floor = value.floor();
    let diff = value - floor;
    let rounded = if diff < 0.5 {
        floor
    } else if diff > 0.5 {
        floor + 1.0
    } else if (floor as i128).rem_euclid(2) == 0 {
        floor
    } else {
        floor + 1.0
    };
    Ok(rounded as i64)
}

fn append_smoothed_points(xs: &mut Vec<i64>, ys: &mut Vec<i64>, points: &[PitchPoint]) {
    if points.is_empty() {
        return;
    }

    let processed = fill_short_gaps(points, 12);
    let values = processed.iter().map(|point| point.y).collect::<Vec<_>>();
    let smoothed = adaptive_smooth(&median_filter(&values, 2), 75.0, 0.7);
    for (point, y) in processed.iter().zip(smoothed) {
        if xs.last() == Some(&point.x) {
            if let Some(last_y) = ys.last_mut() {
                *last_y = y;
            }
        } else {
            xs.push(point.x);
            ys.push(y);
        }
    }
}

fn fill_short_gaps(points: &[PitchPoint], max_gap_steps: i64) -> Vec<PitchPoint> {
    if points.is_empty() {
        return Vec::new();
    }

    let mut expanded = vec![points[0]];
    for point in points.iter().skip(1) {
        let prev = *expanded.last().unwrap();
        let gap_steps = 0.max((point.x - prev.x).div_euclid(U_CURVE_INTERVAL) - 1);
        if 0 < gap_steps && gap_steps <= max_gap_steps {
            for step in 1..=gap_steps {
                let ratio = step as f64 / (gap_steps + 1) as f64;
                let y = round_half_even_i64(prev.y as f64 + (point.y - prev.y) as f64 * ratio)
                    .unwrap_or(prev.y);
                expanded.push(PitchPoint {
                    x: prev.x + step * U_CURVE_INTERVAL,
                    y,
                });
            }
        }
        expanded.push(*point);
    }
    expanded
}

fn median_filter(values: &[i64], radius: usize) -> Vec<i64> {
    if values.is_empty() {
        return Vec::new();
    }

    let mut result = Vec::with_capacity(values.len());
    for index in 0..values.len() {
        let left = index.saturating_sub(radius);
        let right = values.len().min(index + radius + 1);
        let mut window = values[left..right].to_vec();
        window.sort_unstable();
        result.push(window[window.len() / 2]);
    }
    result
}

fn adaptive_smooth(values: &[i64], threshold_cents: f64, blend: f64) -> Vec<i64> {
    if values.len() <= 2 {
        return values.to_vec();
    }

    let mut output = values.to_vec();
    for index in 1..output.len() - 1 {
        let neighbor_avg = (output[index - 1] + output[index + 1]) as f64 / 2.0;
        let delta = output[index] as f64 - neighbor_avg;
        if delta.abs() > threshold_cents {
            output[index] =
                round_half_even_i64(output[index] as f64 - delta * blend).unwrap_or(output[index]);
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    const FIXTURES: &str = include_str!("../../../../fixtures/ustx_pitch_curve_core.jsonl");

    fn parse_number(value: &str) -> f64 {
        match value {
            "nan" => f64::NAN,
            "inf" => f64::INFINITY,
            "-inf" => f64::NEG_INFINITY,
            _ => value.parse().unwrap(),
        }
    }

    fn parse_notes(value: &Value) -> Vec<UstxPitchNote> {
        value
            .as_array()
            .unwrap()
            .iter()
            .map(|raw_note| UstxPitchNote {
                onset: parse_number(raw_note["onset"].as_str().unwrap()),
                offset: parse_number(raw_note["offset"].as_str().unwrap()),
                pitch: parse_number(raw_note["pitch"].as_str().unwrap()),
            })
            .collect()
    }

    fn parse_pitch(value: &Value) -> Vec<f64> {
        value
            .as_array()
            .unwrap()
            .iter()
            .map(|raw| parse_number(raw.as_str().unwrap()))
            .collect()
    }

    fn parse_i64_array(value: &Value) -> Vec<i64> {
        value
            .as_array()
            .unwrap()
            .iter()
            .map(|raw| raw.as_i64().unwrap())
            .collect()
    }

    #[test]
    fn ustx_pitch_curve_follows_parity_fixture_table() {
        for (line_index, line) in FIXTURES.lines().enumerate() {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let case: Value = serde_json::from_str(line).unwrap();
            let case_id = case["case_id"].as_str().unwrap();
            let notes = parse_notes(&case["notes"]);
            let midi_pitch = parse_pitch(&case["midi_pitch"]);

            let actual = render_ustx_pitch_curve(
                &notes,
                &midi_pitch,
                case["time_step_seconds"].as_f64().unwrap(),
                case["tempo"].as_f64().unwrap(),
            )
            .unwrap();

            assert_eq!(
                actual.xs,
                parse_i64_array(&case["expected_xs"]),
                "{case_id} fixture line {} xs",
                line_index + 1
            );
            assert_eq!(
                actual.ys,
                parse_i64_array(&case["expected_ys"]),
                "{case_id} fixture line {} ys",
                line_index + 1
            );
        }
    }

    #[test]
    fn ustx_pitch_curve_rejects_non_finite_tempo_and_time_step() {
        let notes = [UstxPitchNote {
            onset: 0.0,
            offset: 1.0,
            pitch: 60.0,
        }];
        assert_eq!(
            render_ustx_pitch_curve(&notes, &[60.0], 0.01, f64::NAN).unwrap_err(),
            UstxPitchCurveError::InvalidTempo
        );
        assert_eq!(
            render_ustx_pitch_curve(&notes, &[60.0], f64::INFINITY, 120.0).unwrap_err(),
            UstxPitchCurveError::InvalidTimeStep
        );
    }
}
