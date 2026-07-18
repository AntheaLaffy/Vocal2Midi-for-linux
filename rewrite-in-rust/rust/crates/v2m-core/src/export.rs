//! TXT/CSV note export rendering.
//!
//! This module mirrors the deterministic `_save_text` formatting path in
//! `inference/io/note_io.py` without changing the Python runtime owner.

/// One note row accepted by the TXT/CSV formatter.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NoteInfo<'a> {
    /// The onset.
    pub onset: f64,
    /// The offset.
    pub offset: f64,
    /// The pitch.
    pub pitch: f64,
    /// The lyric.
    pub lyric: &'a str,
}

/// Text export container format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextFileFormat {
    /// Represents the Python-compatible txt case.
    Txt,
    /// Represents the Python-compatible csv case.
    Csv,
}

/// Pitch rendering mode used by the Python export path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PitchFormat {
    /// Represents the Python-compatible number case.
    Number,
    /// Represents the Python-compatible name case.
    Name,
}

/// Rendered export content plus the number of invalid notes skipped.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextExport {
    /// The content.
    pub content: String,
    /// The skipped invalid notes.
    pub skipped_invalid_notes: usize,
}

/// Renders valid notes using the TXT/CSV behavior of Python `_save_text`.
pub fn render_text_export(
    notes: &[NoteInfo<'_>],
    file_format: TextFileFormat,
    pitch_format: PitchFormat,
    round_pitch: bool,
) -> TextExport {
    let mut valid_notes = Vec::new();
    let mut skipped_invalid_notes = 0usize;

    for note in notes {
        if is_valid_note(note) {
            valid_notes.push(*note);
        } else {
            skipped_invalid_notes += 1;
        }
    }

    let rows = valid_notes
        .iter()
        .map(|note| RenderedNoteRow {
            onset: format!("{:.3}", note.onset),
            offset: format!("{:.3}", note.offset),
            pitch: format_pitch(note.pitch, pitch_format, round_pitch),
            lyric: note.lyric,
        })
        .collect::<Vec<_>>();

    let has_lyrics = valid_notes.iter().any(|note| !note.lyric.is_empty());
    let content = match file_format {
        TextFileFormat::Txt => render_txt(&rows, has_lyrics),
        TextFileFormat::Csv => render_csv(&rows, has_lyrics),
    };

    TextExport {
        content,
        skipped_invalid_notes,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RenderedNoteRow<'a> {
    onset: String,
    offset: String,
    pitch: String,
    lyric: &'a str,
}

fn is_valid_note(note: &NoteInfo<'_>) -> bool {
    note.onset.is_finite()
        && note.offset.is_finite()
        && note.pitch.is_finite()
        && note.offset > note.onset
}

fn format_pitch(pitch: f64, pitch_format: PitchFormat, round_pitch: bool) -> String {
    let pitch = if round_pitch {
        round_half_even(pitch)
    } else {
        pitch
    };

    match pitch_format {
        PitchFormat::Number => format!("{pitch:.3}"),
        PitchFormat::Name => midi_to_note_ascii(clamp(pitch, 0.0, 127.0), !round_pitch),
    }
}

fn render_txt(rows: &[RenderedNoteRow<'_>], has_lyrics: bool) -> String {
    let mut output = String::new();
    for row in rows {
        if has_lyrics {
            output.push_str(&format!(
                "{}\t{}\t{}\t{}\n",
                row.onset, row.offset, row.pitch, row.lyric
            ));
        } else {
            output.push_str(&format!("{}\t{}\t{}\n", row.onset, row.offset, row.pitch));
        }
    }
    output
}

fn render_csv(rows: &[RenderedNoteRow<'_>], has_lyrics: bool) -> String {
    let mut output = String::new();
    if has_lyrics {
        output.push_str("onset,offset,pitch,lyric\r\n");
    } else {
        output.push_str("onset,offset,pitch\r\n");
    }

    for row in rows {
        output.push_str(&csv_field(&row.onset));
        output.push(',');
        output.push_str(&csv_field(&row.offset));
        output.push(',');
        output.push_str(&csv_field(&row.pitch));
        if has_lyrics {
            output.push(',');
            output.push_str(&csv_field(row.lyric));
        }
        output.push_str("\r\n");
    }

    output
}

fn csv_field(value: &str) -> String {
    if !value.contains([',', '"', '\r', '\n']) {
        return value.to_string();
    }

    let mut escaped = String::with_capacity(value.len() + 2);
    escaped.push('"');
    for char in value.chars() {
        if char == '"' {
            escaped.push('"');
        }
        escaped.push(char);
    }
    escaped.push('"');
    escaped
}

fn midi_to_note_ascii(midi: f64, cents: bool) -> String {
    const NOTE_MAP: [&str; 12] = [
        "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
    ];

    let note_num = round_half_even(midi) as i32;
    let note_cents = (100.0 * round_half_even_to_decimal(midi - f64::from(note_num), 2)) as i32;
    let note = NOTE_MAP[note_num.rem_euclid(12) as usize];
    let octave = note_num / 12 - 1;

    if cents {
        format!("{note}{octave}{note_cents:+}")
    } else {
        format!("{note}{octave}")
    }
}

fn clamp(value: f64, min: f64, max: f64) -> f64 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

fn round_half_even_to_decimal(value: f64, places: i32) -> f64 {
    let factor = 10_f64.powi(places);
    round_half_even(value * factor) / factor
}

fn round_half_even(value: f64) -> f64 {
    if !value.is_finite() {
        return value;
    }

    let floor = value.floor();
    let diff = value - floor;
    if diff < 0.5 {
        floor
    } else if diff > 0.5 {
        floor + 1.0
    } else if (floor as i128).rem_euclid(2) == 0 {
        floor
    } else {
        floor + 1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURES: &str = include_str!("../../../../fixtures/note_text_csv_export_core.tsv");

    fn parse_format(value: &str) -> TextFileFormat {
        match value {
            "txt" => TextFileFormat::Txt,
            "csv" => TextFileFormat::Csv,
            _ => panic!("unknown text file format {value}"),
        }
    }

    fn parse_pitch_format(value: &str) -> PitchFormat {
        match value {
            "number" => PitchFormat::Number,
            "name" => PitchFormat::Name,
            _ => panic!("unknown pitch format {value}"),
        }
    }

    fn parse_bool(value: &str) -> bool {
        match value {
            "true" => true,
            "false" => false,
            _ => panic!("unknown bool {value}"),
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

    fn parse_lyric(value: &str) -> String {
        if value == "__empty__" {
            String::new()
        } else {
            decode_escaped(value)
        }
    }

    fn parse_notes(value: &str) -> Vec<OwnedNoteInfo> {
        if value.is_empty() {
            return Vec::new();
        }

        value
            .split('|')
            .map(|raw_note| {
                let mut fields = raw_note.splitn(4, ',');
                OwnedNoteInfo {
                    onset: parse_number(fields.next().unwrap()),
                    offset: parse_number(fields.next().unwrap()),
                    pitch: parse_number(fields.next().unwrap()),
                    lyric: parse_lyric(fields.next().unwrap()),
                }
            })
            .collect()
    }

    fn decode_escaped(value: &str) -> String {
        let mut result = String::new();
        let mut chars = value.chars();
        while let Some(char) = chars.next() {
            if char != '\\' {
                result.push(char);
                continue;
            }

            match chars.next() {
                Some('t') => result.push('\t'),
                Some('n') => result.push('\n'),
                Some('r') => result.push('\r'),
                Some('\\') => result.push('\\'),
                Some(other) => result.push(other),
                None => result.push('\\'),
            }
        }
        result
    }

    #[derive(Debug, Clone, PartialEq)]
    struct OwnedNoteInfo {
        onset: f64,
        offset: f64,
        pitch: f64,
        lyric: String,
    }

    impl OwnedNoteInfo {
        fn as_note(&self) -> NoteInfo<'_> {
            NoteInfo {
                onset: self.onset,
                offset: self.offset,
                pitch: self.pitch,
                lyric: &self.lyric,
            }
        }
    }

    #[test]
    fn note_text_csv_export_follows_parity_fixture_table() {
        for (line_number, line) in FIXTURES.lines().enumerate() {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let mut fields = line.split('\t');
            let case_id = fields.next().unwrap();
            let file_format = parse_format(fields.next().unwrap());
            let pitch_format = parse_pitch_format(fields.next().unwrap());
            let round_pitch = parse_bool(fields.next().unwrap());
            let owned_notes = parse_notes(fields.next().unwrap());
            let notes = owned_notes
                .iter()
                .map(OwnedNoteInfo::as_note)
                .collect::<Vec<_>>();
            let expected_skipped: usize = fields.next().unwrap().parse().unwrap();
            let expected_content = decode_escaped(fields.next().unwrap());

            let actual = render_text_export(&notes, file_format, pitch_format, round_pitch);
            assert_eq!(
                actual.skipped_invalid_notes,
                expected_skipped,
                "{case_id} fixture line {} skipped count",
                line_number + 1
            );
            assert_eq!(
                actual.content,
                expected_content,
                "{case_id} fixture line {} rendered content",
                line_number + 1
            );
        }
    }

    #[test]
    fn pitch_rounding_matches_python_half_even_edges() {
        assert_eq!(midi_to_note_ascii(60.5, false), "C4");
        assert_eq!(midi_to_note_ascii(61.5, false), "D4");
        assert_eq!(format_pitch(60.5, PitchFormat::Name, true), "C4");
        assert_eq!(format_pitch(61.5, PitchFormat::Number, true), "62.000");
    }

    #[test]
    fn csv_quotes_fields_like_python_csv_writer() {
        assert_eq!(csv_field("plain"), "plain");
        assert_eq!(csv_field("a,b"), "\"a,b\"");
        assert_eq!(csv_field("a\"b"), "\"a\"\"b\"");
        assert_eq!(csv_field("a\nb"), "\"a\nb\"");
    }
}
