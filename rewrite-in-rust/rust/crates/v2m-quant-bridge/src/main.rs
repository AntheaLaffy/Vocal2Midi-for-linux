use std::{
    cmp::Ordering,
    io::{self, Read, Write},
    process::ExitCode,
};

use serde::{Deserialize, Serialize};
use v2m_core::quant::{
    SimpleGridNote, quantize_notes_bayesian, quantize_notes_phrase_dp, quantize_notes_simple,
    quantize_notes_smart, should_apply_quantization,
};

const SUPPORTED_VERSION: u64 = 1;
const MAX_ABS_TICK: f64 = 9_000_000_000_000.0;
const MAX_ABS_STEP: i64 = 1_000_000_000;

#[derive(Debug, Deserialize)]
struct BridgeRequest {
    version: u64,
    mode: Option<String>,
    tempo: f64,
    quantization_step: i64,
    notes: Vec<RequestNote>,
}

#[derive(Debug, Deserialize)]
struct RequestNote {
    index: usize,
    onset: f64,
    offset: f64,
    pitch: f64,
    #[serde(default)]
    lyric: String,
}

#[derive(Debug, Serialize)]
struct BridgeResponse {
    ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    applied: Option<bool>,
    notes: Vec<ResponseNote>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<BridgeError>,
}

#[derive(Debug, Serialize)]
struct ResponseNote {
    index: usize,
    onset: f64,
    offset: f64,
}

#[derive(Debug, Serialize)]
struct BridgeError {
    code: &'static str,
    message: String,
}

impl BridgeResponse {
    fn success(applied: bool, notes: Vec<ResponseNote>) -> Self {
        Self {
            ok: true,
            applied: Some(applied),
            notes,
            error: None,
        }
    }

    fn failure(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            ok: false,
            applied: None,
            notes: Vec::new(),
            error: Some(BridgeError {
                code,
                message: message.into(),
            }),
        }
    }
}

fn main() -> ExitCode {
    let mut input = String::new();
    if let Err(error) = io::stdin().read_to_string(&mut input) {
        write_response(&BridgeResponse::failure(
            "stdin_read_error",
            format!("failed to read stdin: {error}"),
        ));
        return ExitCode::from(2);
    }

    let request = match serde_json::from_str::<BridgeRequest>(&input) {
        Ok(request) => request,
        Err(error) => {
            write_response(&BridgeResponse::failure(
                "invalid_json",
                format!("invalid request JSON: {error}"),
            ));
            return ExitCode::from(2);
        }
    };

    match run_quantization(request) {
        Ok(response) => {
            write_response(&response);
            ExitCode::SUCCESS
        }
        Err(response) => {
            write_response(&response);
            ExitCode::from(1)
        }
    }
}

fn write_response(response: &BridgeResponse) {
    let mut stdout = io::stdout().lock();
    if serde_json::to_writer(&mut stdout, response).is_ok() {
        let _ = stdout.write_all(b"\n");
    }
}

fn run_quantization(request: BridgeRequest) -> Result<BridgeResponse, BridgeResponse> {
    validate_request(&request)?;

    let mode = normalized_mode(request.mode.as_deref());
    let applied = should_apply_quantization(request.mode.as_deref(), request.quantization_step);
    let response_order = if applied {
        sorted_note_positions(&request.notes)
    } else {
        (0..request.notes.len()).collect()
    };

    let mut notes = request
        .notes
        .iter()
        .map(|note| SimpleGridNote {
            onset: note.onset,
            offset: note.offset,
            pitch: note.pitch,
            lyric: &note.lyric,
        })
        .collect::<Vec<_>>();

    if applied {
        match mode.as_str() {
            "smart" => quantize_notes_smart(&mut notes, request.tempo, request.quantization_step),
            "bayes" => {
                quantize_notes_bayesian(&mut notes, request.tempo, request.quantization_step)
            }
            "dp" => quantize_notes_phrase_dp(&mut notes, request.tempo, request.quantization_step),
            _ => quantize_notes_simple(&mut notes, request.tempo, request.quantization_step),
        }
    }

    let response_notes = notes
        .iter()
        .zip(response_order)
        .map(|(note, position)| ResponseNote {
            index: request.notes[position].index,
            onset: note.onset,
            offset: note.offset,
        })
        .collect();

    Ok(BridgeResponse::success(applied, response_notes))
}

fn validate_request(request: &BridgeRequest) -> Result<(), BridgeResponse> {
    if request.version != SUPPORTED_VERSION {
        return Err(BridgeResponse::failure(
            "unsupported_version",
            format!(
                "unsupported version {}; expected {SUPPORTED_VERSION}",
                request.version
            ),
        ));
    }

    if !request.tempo.is_finite() || request.tempo <= 0.0 {
        return Err(BridgeResponse::failure(
            "invalid_tempo",
            "tempo must be a positive finite number",
        ));
    }

    if request.quantization_step.unsigned_abs() > MAX_ABS_STEP as u64 {
        return Err(BridgeResponse::failure(
            "tick_overflow",
            format!("quantization_step magnitude exceeds bridge limit {MAX_ABS_STEP}"),
        ));
    }

    for note in &request.notes {
        validate_note_number(note.index, "onset", note.onset, request.tempo)?;
        validate_note_number(note.index, "offset", note.offset, request.tempo)?;
        if !note.pitch.is_finite() {
            return Err(BridgeResponse::failure(
                "invalid_note",
                format!("note {} pitch must be finite", note.index),
            ));
        }
    }

    Ok(())
}

fn validate_note_number(
    index: usize,
    name: &'static str,
    value: f64,
    tempo: f64,
) -> Result<(), BridgeResponse> {
    if !value.is_finite() {
        return Err(BridgeResponse::failure(
            "invalid_note",
            format!("note {index} {name} must be finite"),
        ));
    }

    let ticks = value * tempo * 8.0;
    if !ticks.is_finite() || ticks.abs() > MAX_ABS_TICK {
        return Err(BridgeResponse::failure(
            "tick_overflow",
            format!("note {index} {name} produces an out-of-range tick value"),
        ));
    }

    Ok(())
}

fn normalized_mode(mode: Option<&str>) -> String {
    match mode {
        Some(value) if !value.is_empty() => value.to_lowercase(),
        _ => "simple".to_string(),
    }
}

fn sorted_note_positions(notes: &[RequestNote]) -> Vec<usize> {
    let mut positions = (0..notes.len()).collect::<Vec<_>>();
    positions.sort_by(|&left, &right| {
        notes[left]
            .onset
            .partial_cmp(&notes[right].onset)
            .unwrap_or(Ordering::Equal)
    });
    positions
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(mode: Option<&str>, step: i64) -> BridgeRequest {
        BridgeRequest {
            version: 1,
            mode: mode.map(str::to_string),
            tempo: 120.0,
            quantization_step: step,
            notes: vec![
                RequestNote {
                    index: 10,
                    onset: 0.2,
                    offset: 0.31,
                    pitch: 62.0,
                    lyric: "late".to_string(),
                },
                RequestNote {
                    index: 5,
                    onset: 0.02,
                    offset: 0.1,
                    pitch: 60.0,
                    lyric: "early".to_string(),
                },
            ],
        }
    }

    #[test]
    fn unknown_positive_mode_uses_simple_fallback_and_original_indexes() {
        let response = run_quantization(request(Some("mystery"), 60)).unwrap();
        assert!(response.ok);
        assert_eq!(response.applied, Some(true));
        assert_eq!(response.notes[0].index, 5);
        assert_eq!(response.notes[1].index, 10);
        assert!((response.notes[0].onset - 0.0).abs() <= 1e-12);
        assert!((response.notes[1].onset - 0.1875).abs() <= 1e-12);
    }

    #[test]
    fn disabled_non_dp_mode_preserves_order_and_values() {
        let response = run_quantization(request(Some("simple"), 0)).unwrap();
        assert_eq!(response.applied, Some(false));
        assert_eq!(response.notes[0].index, 10);
        assert_eq!(response.notes[1].index, 5);
        assert!((response.notes[0].onset - 0.2).abs() <= 1e-12);
        assert!((response.notes[1].offset - 0.1).abs() <= 1e-12);
    }

    #[test]
    fn dp_mode_with_step_zero_still_quantizes() {
        let response = run_quantization(request(Some("dp"), 0)).unwrap();
        assert_eq!(response.applied, Some(true));
        assert_eq!(response.notes[0].index, 5);
        assert_eq!(response.notes[1].index, 10);
    }

    #[test]
    fn padded_dp_with_step_zero_is_disabled_like_python_activation() {
        let response = run_quantization(request(Some(" dp "), 0)).unwrap();
        assert_eq!(response.applied, Some(false));
        assert_eq!(response.notes[0].index, 10);
        assert_eq!(response.notes[1].index, 5);
    }

    #[test]
    fn invalid_numbers_are_rejected_before_quantization() {
        let mut request = request(Some("simple"), 60);
        request.notes[0].onset = f64::INFINITY;
        let response = run_quantization(request).unwrap_err();
        assert_eq!(response.error.unwrap().code, "invalid_note");
    }
}
