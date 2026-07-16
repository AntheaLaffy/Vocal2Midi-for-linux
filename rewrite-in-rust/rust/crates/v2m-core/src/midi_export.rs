//! MIDI note export rendering.
//!
//! This module mirrors the deterministic `_save_midi` path in
//! `inference/io/note_io.py` for fixture-backed parity only. Python remains the
//! runtime owner for filesystem writes and export routing.

/// Standard MIDI file type used by `mido.MidiFile()` in the legacy path.
pub const MIDI_FILE_TYPE: u16 = 1;

/// Ticks per beat used by Mido's default `MidiFile`.
pub const TICKS_PER_BEAT: u16 = 480;

const VELOCITY: u8 = 100;
const CHANNEL: u8 = 0;

/// One note row accepted by the MIDI formatter.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MidiNoteInfo<'a> {
    pub onset: f64,
    pub offset: f64,
    pub pitch: f64,
    pub lyric: &'a str,
}

/// MIDI event subset emitted by the legacy `_save_midi` path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MidiEvent<'a> {
    SetTempo {
        delta_ticks: u64,
        tempo: u32,
    },
    Lyrics {
        delta_ticks: u64,
        text: &'a str,
    },
    NoteOn {
        delta_ticks: u64,
        note: u8,
        velocity: u8,
        channel: u8,
    },
    NoteOff {
        delta_ticks: u64,
        note: u8,
        velocity: u8,
        channel: u8,
    },
    EndOfTrack {
        delta_ticks: u64,
    },
}

/// Rendered MIDI data plus the number of invalid input notes skipped.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MidiExport<'a> {
    pub file_type: u16,
    pub ticks_per_beat: u16,
    pub events: Vec<MidiEvent<'a>>,
    pub midi_bytes: Vec<u8>,
    pub skipped_invalid_notes: usize,
}

/// Recoverable MIDI rendering failures at the Rust library boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MidiExportError {
    InvalidTempo,
    TempoOutOfRange,
    TickOutOfRange,
    TrackTooLarge,
}

/// Renders the MIDI event plan and file bytes produced by Python `_save_midi`.
///
/// # Errors
///
/// Returns an error when the tempo is non-finite, non-positive, outside Mido's
/// `set_tempo` meta-message range after conversion, or when tick/file sizes
/// exceed the in-memory encoder bounds.
pub fn render_midi_export<'a>(
    notes: &[MidiNoteInfo<'a>],
    tempo_bpm: f64,
) -> Result<MidiExport<'a>, MidiExportError> {
    let tempo = bpm_to_tempo(tempo_bpm)?;
    let mut skipped_invalid_notes = 0usize;
    let mut valid_notes = Vec::new();

    for note in notes {
        if is_valid_note(note) {
            valid_notes.push(*note);
        } else {
            skipped_invalid_notes += 1;
        }
    }

    valid_notes.sort_by(|left, right| left.onset.partial_cmp(&right.onset).unwrap());

    let mut events = vec![MidiEvent::SetTempo {
        delta_ticks: 0,
        tempo,
    }];
    let mut last_abs_ticks = 0i64;

    for note in valid_notes {
        let mut abs_onset_ticks = round_half_even_i64(note.onset * tempo_bpm * 8.0)?;
        let mut abs_offset_ticks = round_half_even_i64(note.offset * tempo_bpm * 8.0)?;

        if abs_onset_ticks < last_abs_ticks {
            abs_onset_ticks = last_abs_ticks;
        }

        if abs_offset_ticks <= abs_onset_ticks {
            abs_offset_ticks = abs_onset_ticks
                .checked_add(1)
                .ok_or(MidiExportError::TickOutOfRange)?;
        }

        let midi_pitch = clamp_midi_pitch(note.pitch);
        let delta_onset_ticks = u64::try_from(abs_onset_ticks - last_abs_ticks)
            .map_err(|_| MidiExportError::TickOutOfRange)?;
        let note_duration_ticks = u64::try_from(abs_offset_ticks - abs_onset_ticks)
            .map_err(|_| MidiExportError::TickOutOfRange)?;

        if note.lyric.is_empty() {
            events.push(MidiEvent::NoteOn {
                delta_ticks: delta_onset_ticks,
                note: midi_pitch,
                velocity: VELOCITY,
                channel: CHANNEL,
            });
        } else {
            events.push(MidiEvent::Lyrics {
                delta_ticks: delta_onset_ticks,
                text: note.lyric,
            });
            events.push(MidiEvent::NoteOn {
                delta_ticks: 0,
                note: midi_pitch,
                velocity: VELOCITY,
                channel: CHANNEL,
            });
        }

        events.push(MidiEvent::NoteOff {
            delta_ticks: note_duration_ticks,
            note: midi_pitch,
            velocity: VELOCITY,
            channel: CHANNEL,
        });

        last_abs_ticks = abs_offset_ticks;
    }

    events.push(MidiEvent::EndOfTrack { delta_ticks: 0 });
    let midi_bytes = encode_midi_file(&events)?;

    Ok(MidiExport {
        file_type: MIDI_FILE_TYPE,
        ticks_per_beat: TICKS_PER_BEAT,
        events,
        midi_bytes,
        skipped_invalid_notes,
    })
}

fn is_valid_note(note: &MidiNoteInfo<'_>) -> bool {
    note.onset.is_finite()
        && note.offset.is_finite()
        && note.pitch.is_finite()
        && note.offset > note.onset
}

fn bpm_to_tempo(tempo_bpm: f64) -> Result<u32, MidiExportError> {
    if !tempo_bpm.is_finite() || tempo_bpm <= 0.0 {
        return Err(MidiExportError::InvalidTempo);
    }

    let tempo = round_half_even_i64(60_000_000.0 / tempo_bpm)?;
    if !(0..=0x00ff_ffff).contains(&tempo) {
        return Err(MidiExportError::TempoOutOfRange);
    }
    Ok(tempo as u32)
}

fn clamp_midi_pitch(pitch: f64) -> u8 {
    if pitch <= 0.0 {
        0
    } else if pitch >= 127.0 {
        127
    } else {
        round_half_even_i64(pitch).unwrap_or(0).clamp(0, 127) as u8
    }
}

fn round_half_even_i64(value: f64) -> Result<i64, MidiExportError> {
    if !value.is_finite() || value < i64::MIN as f64 || value > i64::MAX as f64 {
        return Err(MidiExportError::TickOutOfRange);
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

fn encode_midi_file(events: &[MidiEvent<'_>]) -> Result<Vec<u8>, MidiExportError> {
    let mut track_data = Vec::new();
    for event in events {
        encode_event(&mut track_data, event);
    }

    let track_len = u32::try_from(track_data.len()).map_err(|_| MidiExportError::TrackTooLarge)?;

    let mut bytes = Vec::with_capacity(14 + 8 + track_data.len());
    bytes.extend_from_slice(b"MThd");
    bytes.extend_from_slice(&6u32.to_be_bytes());
    bytes.extend_from_slice(&MIDI_FILE_TYPE.to_be_bytes());
    bytes.extend_from_slice(&1u16.to_be_bytes());
    bytes.extend_from_slice(&TICKS_PER_BEAT.to_be_bytes());
    bytes.extend_from_slice(b"MTrk");
    bytes.extend_from_slice(&track_len.to_be_bytes());
    bytes.extend_from_slice(&track_data);
    Ok(bytes)
}

fn encode_event(output: &mut Vec<u8>, event: &MidiEvent<'_>) {
    match event {
        MidiEvent::SetTempo { delta_ticks, tempo } => {
            encode_variable_int(output, *delta_ticks);
            output.extend_from_slice(&[0xff, 0x51, 0x03]);
            output.push((tempo >> 16) as u8);
            output.push((tempo >> 8) as u8);
            output.push(*tempo as u8);
        }
        MidiEvent::Lyrics { delta_ticks, text } => {
            encode_variable_int(output, *delta_ticks);
            output.extend_from_slice(&[0xff, 0x05]);
            encode_variable_int(output, text.len() as u64);
            output.extend_from_slice(text.as_bytes());
        }
        MidiEvent::NoteOn {
            delta_ticks,
            note,
            velocity,
            channel,
        } => {
            encode_variable_int(output, *delta_ticks);
            output.extend_from_slice(&[0x90 | channel, *note, *velocity]);
        }
        MidiEvent::NoteOff {
            delta_ticks,
            note,
            velocity,
            channel,
        } => {
            encode_variable_int(output, *delta_ticks);
            output.extend_from_slice(&[0x80 | channel, *note, *velocity]);
        }
        MidiEvent::EndOfTrack { delta_ticks } => {
            encode_variable_int(output, *delta_ticks);
            output.extend_from_slice(&[0xff, 0x2f, 0x00]);
        }
    }
}

fn encode_variable_int(output: &mut Vec<u8>, mut value: u64) {
    if value == 0 {
        output.push(0);
        return;
    }

    let mut bytes = Vec::new();
    while value > 0 {
        bytes.push((value & 0x7f) as u8);
        value >>= 7;
    }
    bytes.reverse();

    let last_index = bytes.len() - 1;
    for (index, byte) in bytes.iter_mut().enumerate() {
        if index != last_index {
            *byte |= 0x80;
        }
    }
    output.extend(bytes);
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    const FIXTURES: &str = include_str!("../../../../fixtures/midi_export_core.jsonl");

    #[derive(Debug, Clone, PartialEq)]
    struct OwnedNoteInfo {
        onset: f64,
        offset: f64,
        pitch: f64,
        lyric: String,
    }

    impl OwnedNoteInfo {
        fn as_note(&self) -> MidiNoteInfo<'_> {
            MidiNoteInfo {
                onset: self.onset,
                offset: self.offset,
                pitch: self.pitch,
                lyric: &self.lyric,
            }
        }
    }

    fn parse_number(value: &str) -> f64 {
        match value {
            "nan" => f64::NAN,
            "inf" => f64::INFINITY,
            "-inf" => f64::NEG_INFINITY,
            _ => value.parse().unwrap(),
        }
    }

    fn parse_notes(value: &Value) -> Vec<OwnedNoteInfo> {
        value
            .as_array()
            .unwrap()
            .iter()
            .map(|raw_note| OwnedNoteInfo {
                onset: parse_number(raw_note["onset"].as_str().unwrap()),
                offset: parse_number(raw_note["offset"].as_str().unwrap()),
                pitch: parse_number(raw_note["pitch"].as_str().unwrap()),
                lyric: raw_note["lyric"].as_str().unwrap().to_owned(),
            })
            .collect()
    }

    fn event_to_json(event: &MidiEvent<'_>) -> Value {
        match event {
            MidiEvent::SetTempo { delta_ticks, tempo } => {
                json!({"type": "set_tempo", "delta_ticks": delta_ticks, "tempo": tempo})
            }
            MidiEvent::Lyrics { delta_ticks, text } => {
                json!({"type": "lyrics", "delta_ticks": delta_ticks, "text": text})
            }
            MidiEvent::NoteOn {
                delta_ticks,
                note,
                velocity,
                channel,
            } => json!({
                "type": "note_on",
                "delta_ticks": delta_ticks,
                "note": note,
                "velocity": velocity,
                "channel": channel,
            }),
            MidiEvent::NoteOff {
                delta_ticks,
                note,
                velocity,
                channel,
            } => json!({
                "type": "note_off",
                "delta_ticks": delta_ticks,
                "note": note,
                "velocity": velocity,
                "channel": channel,
            }),
            MidiEvent::EndOfTrack { delta_ticks } => {
                json!({"type": "end_of_track", "delta_ticks": delta_ticks})
            }
        }
    }

    fn bytes_to_hex(bytes: &[u8]) -> String {
        let mut output = String::with_capacity(bytes.len() * 2);
        for byte in bytes {
            output.push_str(&format!("{byte:02x}"));
        }
        output
    }

    #[test]
    fn midi_export_follows_parity_fixture_table() {
        for (line_index, line) in FIXTURES.lines().enumerate() {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let case: Value = serde_json::from_str(line).unwrap();
            let case_id = case["case_id"].as_str().unwrap();
            let owned_notes = parse_notes(&case["notes"]);
            let notes = owned_notes
                .iter()
                .map(OwnedNoteInfo::as_note)
                .collect::<Vec<_>>();
            let expected = &case["expect"];

            let actual = render_midi_export(&notes, case["tempo"].as_f64().unwrap()).unwrap();
            let actual_events = actual.events.iter().map(event_to_json).collect::<Vec<_>>();

            assert_eq!(
                actual.skipped_invalid_notes,
                expected["skipped_invalid_notes"].as_u64().unwrap() as usize,
                "{case_id} fixture line {} skipped count",
                line_index + 1
            );
            assert_eq!(
                actual.file_type,
                expected["type"].as_u64().unwrap() as u16,
                "{case_id} fixture line {} MIDI type",
                line_index + 1
            );
            assert_eq!(
                actual.ticks_per_beat,
                expected["ticks_per_beat"].as_u64().unwrap() as u16,
                "{case_id} fixture line {} ticks per beat",
                line_index + 1
            );
            assert_eq!(
                actual_events,
                expected["events"].as_array().unwrap().clone(),
                "{case_id} fixture line {} events",
                line_index + 1
            );
            assert_eq!(
                bytes_to_hex(&actual.midi_bytes),
                expected["midi_hex"].as_str().unwrap(),
                "{case_id} fixture line {} MIDI hex",
                line_index + 1
            );
        }
    }

    #[test]
    fn midi_export_rejects_invalid_tempo_without_panicking() {
        let note = MidiNoteInfo {
            onset: 0.0,
            offset: 0.5,
            pitch: 60.0,
            lyric: "",
        };
        assert_eq!(
            render_midi_export(&[note], 0.0).unwrap_err(),
            MidiExportError::InvalidTempo
        );
        assert_eq!(
            render_midi_export(&[note], f64::NAN).unwrap_err(),
            MidiExportError::InvalidTempo
        );
    }

    #[test]
    fn midi_variable_int_matches_mido_edges() {
        let mut output = Vec::new();
        encode_variable_int(&mut output, 0);
        assert_eq!(output, vec![0x00]);

        output.clear();
        encode_variable_int(&mut output, 127);
        assert_eq!(output, vec![0x7f]);

        output.clear();
        encode_variable_int(&mut output, 128);
        assert_eq!(output, vec![0x81, 0x00]);

        output.clear();
        encode_variable_int(&mut output, 287);
        assert_eq!(output, vec![0x82, 0x1f]);
    }
}
