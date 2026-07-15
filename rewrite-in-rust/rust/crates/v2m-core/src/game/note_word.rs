//! GAME note-to-word alignment.
//!
//! This module mirrors `inference/game/alignment_utils.py::align_notes_to_words`
//! without changing the Python runtime owner.

const ALIGN_MIN_GAP: f64 = 1e-4;

/// Aligns note sequence data to word durations.
pub fn align_notes_to_words(
    word_dur: &[f64],
    word_vuv: &[u8],
    note_seq: &[String],
    note_dur: &[f64],
    tol: f64,
    apply_word_uv: bool,
) -> (Vec<String>, Vec<f64>, Vec<u8>) {
    if word_dur.is_empty() || note_dur.is_empty() || note_seq.is_empty() {
        return (Vec::new(), Vec::new(), Vec::new());
    }

    let word_boundaries = cumsum_with_zero(word_dur);
    let note_boundaries = cumsum_with_zero(note_dur);
    if note_boundaries.len() < 2 {
        return (Vec::new(), Vec::new(), Vec::new());
    }

    let mut aligned_boundaries = word_boundaries.clone();
    for boundary_idx in 1..word_boundaries.len().saturating_sub(1) {
        let raw_boundary = word_boundaries[boundary_idx];
        let nearest_idx = nearest_boundary_index(&note_boundaries, raw_boundary);
        let nearest_boundary = note_boundaries[nearest_idx];
        let mut candidate = raw_boundary;
        if (nearest_boundary - raw_boundary).abs() <= tol {
            candidate = nearest_boundary;
        }

        let lower = aligned_boundaries[boundary_idx - 1] + ALIGN_MIN_GAP;
        let upper = word_boundaries[boundary_idx + 1] - ALIGN_MIN_GAP;
        if lower < upper {
            aligned_boundaries[boundary_idx] = candidate.clamp(lower, upper);
        } else {
            aligned_boundaries[boundary_idx] = raw_boundary;
        }
    }

    let word_start = &aligned_boundaries[..aligned_boundaries.len() - 1];
    let word_end = &aligned_boundaries[1..];
    let note_start = &note_boundaries[..note_boundaries.len() - 1];
    let note_end = &note_boundaries[1..];
    let mut new_note_seq = Vec::new();
    let mut new_note_dur = Vec::new();
    let mut note_slur = Vec::new();
    let mut note_idx = 0usize;

    for word_idx in 0..word_dur.len() {
        let start = word_start[word_idx];
        let end = word_end[word_idx];
        if end <= start {
            continue;
        }

        while note_idx < note_end.len() && note_end[note_idx] <= start + ALIGN_MIN_GAP {
            note_idx += 1;
        }

        if apply_word_uv && word_vuv[word_idx] == 0 {
            new_note_seq.push("rest".to_string());
            new_note_dur.push(end - start);
            note_slur.push(0);
            while note_idx < note_end.len() && note_end[note_idx] <= end + ALIGN_MIN_GAP {
                note_idx += 1;
            }
            continue;
        }

        let mut word_note_seq: Vec<String> = Vec::new();
        let mut word_note_dur: Vec<f64> = Vec::new();
        let mut scan_idx = note_idx;
        while scan_idx < note_seq.len() && note_start[scan_idx] < end - ALIGN_MIN_GAP {
            let seg_start = start.max(note_start[scan_idx]);
            let seg_end = end.min(note_end[scan_idx]);
            let seg_dur = seg_end - seg_start;
            if seg_dur > ALIGN_MIN_GAP {
                if word_note_seq
                    .last()
                    .is_some_and(|last| last == &note_seq[scan_idx])
                {
                    *word_note_dur.last_mut().unwrap() += seg_dur;
                } else {
                    word_note_seq.push(note_seq[scan_idx].clone());
                    word_note_dur.push(seg_dur);
                }
            }
            if note_end[scan_idx] <= end + ALIGN_MIN_GAP {
                scan_idx += 1;
            } else {
                break;
            }
        }

        if word_note_seq.is_empty() {
            word_note_seq.push("rest".to_string());
            word_note_dur.push(end - start);
        }

        for (idx, seq) in word_note_seq.into_iter().enumerate() {
            new_note_seq.push(seq);
            note_slur.push(if idx == 0 { 0 } else { 1 });
        }
        new_note_dur.extend(word_note_dur);

        while note_idx < note_end.len() && note_end[note_idx] <= end + ALIGN_MIN_GAP {
            note_idx += 1;
        }
    }

    (new_note_seq, new_note_dur, note_slur)
}

fn cumsum_with_zero(values: &[f64]) -> Vec<f64> {
    let mut boundaries = Vec::with_capacity(values.len() + 1);
    boundaries.push(0.0);
    let mut total = 0.0;
    for &value in values {
        total += value;
        boundaries.push(total);
    }
    boundaries
}

fn nearest_boundary_index(boundaries: &[f64], raw_boundary: f64) -> usize {
    let mut nearest_idx = 0usize;
    let mut nearest_distance = (boundaries[0] - raw_boundary).abs();
    for (idx, boundary) in boundaries.iter().enumerate().skip(1) {
        let distance = (*boundary - raw_boundary).abs();
        if distance < nearest_distance {
            nearest_idx = idx;
            nearest_distance = distance;
        }
    }
    nearest_idx
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURES: &str = include_str!("../../../../../fixtures/game_note_word_alignment.tsv");
    const FLOAT_TOL: f64 = 1e-12;

    fn parse_str_list(value: &str) -> Vec<String> {
        if value.is_empty() {
            Vec::new()
        } else {
            value.split(',').map(str::to_string).collect()
        }
    }

    fn parse_float_list(value: &str) -> Vec<f64> {
        if value.is_empty() {
            Vec::new()
        } else {
            value.split(',').map(|item| item.parse().unwrap()).collect()
        }
    }

    fn parse_u8_list(value: &str) -> Vec<u8> {
        if value.is_empty() {
            Vec::new()
        } else {
            value.split(',').map(|item| item.parse().unwrap()).collect()
        }
    }

    fn parse_bool(value: &str) -> bool {
        match value {
            "true" => true,
            "false" => false,
            _ => panic!("unknown bool {value}"),
        }
    }

    fn assert_float_lists_close(actual: &[f64], expected: &[f64], case_id: &str) {
        assert_eq!(actual.len(), expected.len(), "{case_id} length mismatch");
        for (index, (actual_value, expected_value)) in actual.iter().zip(expected).enumerate() {
            assert!(
                (actual_value - expected_value).abs() <= FLOAT_TOL,
                "{case_id} float mismatch at {index}: {actual_value:?} != {expected_value:?}"
            );
        }
    }

    #[test]
    fn note_word_alignment_follows_fixture_table() {
        for line in FIXTURES.lines() {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let mut fields = line.split('\t');
            let case_id = fields.next().unwrap();
            let word_dur = parse_float_list(fields.next().unwrap());
            let word_vuv = parse_u8_list(fields.next().unwrap());
            let note_seq = parse_str_list(fields.next().unwrap());
            let note_dur = parse_float_list(fields.next().unwrap());
            let tol = fields.next().unwrap().parse().unwrap();
            let apply_word_uv = parse_bool(fields.next().unwrap());
            let expected_seq = parse_str_list(fields.next().unwrap());
            let expected_dur = parse_float_list(fields.next().unwrap());
            let expected_slur = parse_u8_list(fields.next().unwrap());

            let (actual_seq, actual_dur, actual_slur) = align_notes_to_words(
                &word_dur,
                &word_vuv,
                &note_seq,
                &note_dur,
                tol,
                apply_word_uv,
            );
            assert_eq!(actual_seq, expected_seq, "{case_id} seq mismatch");
            assert_float_lists_close(&actual_dur, &expected_dur, case_id);
            assert_eq!(actual_slur, expected_slur, "{case_id} slur mismatch");
        }
    }

    #[test]
    fn nearest_boundary_index_uses_first_minimum_like_numpy_argmin() {
        let boundaries = [0.0, 0.1, 0.2, 0.3];
        assert_eq!(nearest_boundary_index(&boundaries, 0.15), 1);
    }
}
