//! GAME phoneme-to-word parsing helpers.
//!
//! This module mirrors selected helpers from
//! `inference/game/alignment_utils.py` without changing the Python runtime
//! owner.

/// Validation failure returned by `validate_phones`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PhoneValidationError {
    LengthMismatch {
        phoneme_count: usize,
        duration_count: usize,
    },
    WordSpanMismatch {
        spans: Vec<usize>,
        sum: usize,
        expected: usize,
    },
}

impl PhoneValidationError {
    /// Returns the Python-compatible error message.
    pub fn message(&self) -> String {
        match self {
            Self::LengthMismatch {
                phoneme_count,
                duration_count,
            } => {
                format!("Length mismatch: {phoneme_count} phonemes vs {duration_count} durations.")
            }
            Self::WordSpanMismatch {
                spans,
                sum,
                expected,
            } => {
                format!(
                    "Word span mismatch: sum of {} is {sum}, expected {expected}.",
                    format_usize_list(spans)
                )
            }
        }
    }
}

impl std::fmt::Display for PhoneValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message())
    }
}

impl std::error::Error for PhoneValidationError {}

/// Condition used to mark a parsed word as unvoiced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UvCondition {
    Lead,
    All,
}

/// Validates the phoneme sequence, durations, and word spans.
pub fn validate_phones(
    ph_seq: &[String],
    ph_dur: &[f64],
    ph_num: &[usize],
) -> Result<(), PhoneValidationError> {
    if ph_seq.len() != ph_dur.len() {
        return Err(PhoneValidationError::LengthMismatch {
            phoneme_count: ph_seq.len(),
            duration_count: ph_dur.len(),
        });
    }

    let span_sum: usize = ph_num.iter().sum();
    if span_sum != ph_seq.len() {
        return Err(PhoneValidationError::WordSpanMismatch {
            spans: ph_num.to_vec(),
            sum: span_sum,
            expected: ph_seq.len(),
        });
    }

    Ok(())
}

/// Validates phones and returns the Python-compatible `(is_valid, message)` shape.
pub fn validate_phones_py_shape(
    ph_seq: &[String],
    ph_dur: &[f64],
    ph_num: &[usize],
) -> (bool, Option<String>) {
    match validate_phones(ph_seq, ph_dur, ph_num) {
        Ok(()) => (true, None),
        Err(error) => (false, Some(error.message())),
    }
}

/// Converts phoneme sequence data to word durations and voiced/unvoiced flags.
pub fn parse_words(
    ph_seq: &[String],
    ph_dur: &[f64],
    ph_num: &[usize],
    uv_vocab: Option<&[String]>,
    uv_cond: UvCondition,
    merge_consecutive_uv: bool,
) -> (Vec<f64>, Vec<u8>) {
    let mut word_dur = Vec::new();
    let mut word_vuv = Vec::new();
    let mut idx = 0usize;

    for &num in ph_num {
        let dur_sum: f64 = ph_dur[idx..idx + num].iter().sum();
        word_dur.push(dur_sum);

        let mut vuv = 1u8;
        if let Some(uv_vocab) = uv_vocab {
            match uv_cond {
                UvCondition::Lead => {
                    if contains_phone(uv_vocab, &ph_seq[idx]) {
                        vuv = 0;
                    }
                }
                UvCondition::All => {
                    if ph_seq[idx..idx + num]
                        .iter()
                        .all(|phone| contains_phone(uv_vocab, phone))
                    {
                        vuv = 0;
                    }
                }
            }
        }

        word_vuv.push(vuv);
        idx += num;
    }

    if merge_consecutive_uv {
        merge_consecutive_uv_words(&word_dur, &word_vuv)
    } else {
        (word_dur, word_vuv)
    }
}

/// Merges consecutive unvoiced words into one.
pub fn merge_consecutive_uv_words(word_dur: &[f64], word_vuv: &[u8]) -> (Vec<f64>, Vec<u8>) {
    if word_dur.is_empty() {
        return (Vec::new(), Vec::new());
    }

    let mut merged_dur = vec![word_dur[0]];
    let mut merged_vuv = vec![word_vuv[0]];

    for (&dur, &vuv) in word_dur.iter().skip(1).zip(word_vuv.iter().skip(1)) {
        if vuv == 0 && *merged_vuv.last().unwrap() == 0 {
            *merged_dur.last_mut().unwrap() += dur;
        } else {
            merged_dur.push(dur);
            merged_vuv.push(vuv);
        }
    }

    (merged_dur, merged_vuv)
}

fn contains_phone(uv_vocab: &[String], phone: &str) -> bool {
    uv_vocab.iter().any(|candidate| candidate == phone)
}

fn format_usize_list(values: &[usize]) -> String {
    let body = values
        .iter()
        .map(usize::to_string)
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{body}]")
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALIDATION_FIXTURES: &str =
        include_str!("../../../../../fixtures/game_phone_word_validation.tsv");
    const PARSE_FIXTURES: &str = include_str!("../../../../../fixtures/game_parse_words.tsv");
    const MERGE_FIXTURES: &str = include_str!("../../../../../fixtures/game_merge_uv.tsv");
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

    fn parse_usize_list(value: &str) -> Vec<usize> {
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

    fn parse_uv_vocab(value: &str) -> Option<Vec<String>> {
        if value == "__none__" {
            None
        } else {
            Some(parse_str_list(value))
        }
    }

    fn parse_uv_condition(value: &str) -> UvCondition {
        match value {
            "lead" => UvCondition::Lead,
            "all" => UvCondition::All,
            _ => panic!("unknown uv condition {value}"),
        }
    }

    fn assert_float_lists_close(actual: &[f64], expected: &[f64], line_number: usize) {
        assert_eq!(
            actual.len(),
            expected.len(),
            "fixture line {line_number} length mismatch"
        );
        for (index, (actual_value, expected_value)) in actual.iter().zip(expected).enumerate() {
            assert!(
                (actual_value - expected_value).abs() <= FLOAT_TOL,
                "fixture line {line_number} float mismatch at {index}: {actual_value:?} != {expected_value:?}"
            );
        }
    }

    #[test]
    fn validate_phones_follows_fixture_table() {
        for (line_number, line) in VALIDATION_FIXTURES.lines().enumerate() {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let mut fields = line.split('\t');
            let ph_seq = parse_str_list(fields.next().unwrap());
            let ph_dur = parse_float_list(fields.next().unwrap());
            let ph_num = parse_usize_list(fields.next().unwrap());
            let expected_valid = parse_bool(fields.next().unwrap());
            let expected_message = fields.next().unwrap_or("");

            let expected_shape = if expected_valid {
                (true, None)
            } else {
                (false, Some(expected_message.to_string()))
            };
            assert_eq!(
                validate_phones_py_shape(&ph_seq, &ph_dur, &ph_num),
                expected_shape,
                "line {}",
                line_number + 1
            );

            match (expected_valid, validate_phones(&ph_seq, &ph_dur, &ph_num)) {
                (true, Ok(())) => {}
                (false, Err(error)) => {
                    assert_eq!(
                        error.message(),
                        expected_message,
                        "line {}",
                        line_number + 1
                    );
                }
                (true, Err(error)) => {
                    panic!("line {} failed unexpectedly: {}", line_number + 1, error);
                }
                (false, Ok(())) => panic!("line {} passed unexpectedly", line_number + 1),
            }
        }
    }

    #[test]
    fn parse_words_follows_fixture_table() {
        for (line_number, line) in PARSE_FIXTURES.lines().enumerate() {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let mut fields = line.split('\t');
            let ph_seq = parse_str_list(fields.next().unwrap());
            let ph_dur = parse_float_list(fields.next().unwrap());
            let ph_num = parse_usize_list(fields.next().unwrap());
            let uv_vocab = parse_uv_vocab(fields.next().unwrap());
            let uv_cond = parse_uv_condition(fields.next().unwrap());
            let merge = parse_bool(fields.next().unwrap());
            let expected_dur = parse_float_list(fields.next().unwrap());
            let expected_vuv = parse_u8_list(fields.next().unwrap());

            let (actual_dur, actual_vuv) = parse_words(
                &ph_seq,
                &ph_dur,
                &ph_num,
                uv_vocab.as_deref(),
                uv_cond,
                merge,
            );
            assert_float_lists_close(&actual_dur, &expected_dur, line_number + 1);
            assert_eq!(actual_vuv, expected_vuv, "line {}", line_number + 1);
        }
    }

    #[test]
    fn merge_consecutive_uv_words_follows_fixture_table() {
        for (line_number, line) in MERGE_FIXTURES.lines().enumerate() {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let mut fields = line.split('\t');
            let word_dur = parse_float_list(fields.next().unwrap());
            let word_vuv = parse_u8_list(fields.next().unwrap());
            let expected_dur = parse_float_list(fields.next().unwrap());
            let expected_vuv = parse_u8_list(fields.next().unwrap());

            let (actual_dur, actual_vuv) = merge_consecutive_uv_words(&word_dur, &word_vuv);
            assert_float_lists_close(&actual_dur, &expected_dur, line_number + 1);
            assert_eq!(actual_vuv, expected_vuv, "line {}", line_number + 1);
        }
    }

    #[test]
    fn validate_phones_error_display_matches_message() {
        let ph_seq = parse_str_list("a,b,c");
        let ph_dur = parse_float_list("0.1,0.2,0.3");
        let ph_num = parse_usize_list("1,3");
        let error = validate_phones(&ph_seq, &ph_dur, &ph_num).unwrap_err();
        assert_eq!(
            error.to_string(),
            "Word span mismatch: sum of [1, 3] is 4, expected 3."
        );
    }
}
