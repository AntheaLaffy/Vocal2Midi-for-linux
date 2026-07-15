//! Slice duration bounds validation.
//!
//! This module mirrors `application/config.py::validate_slice_bounds` without
//! changing the Python runtime owner.

/// Smallest slice duration accepted by the Python application boundary.
pub const SLICE_DURATION_MIN_SEC: f64 = 0.0;

/// Largest slice duration accepted by the Python application boundary.
pub const SLICE_DURATION_MAX_SEC: f64 = 60.0;

/// Default minimum slice duration used by Python callers.
pub const DEFAULT_SLICE_MIN_SEC: f64 = 5.0;

/// Default maximum slice duration used by Python callers.
pub const DEFAULT_SLICE_MAX_SEC: f64 = 10.0;

/// Validation failure that maps to Python `ValueError` on a future bridge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SliceBoundsError {
    SliceMinOutOfRange,
    SliceMaxOutOfRange,
    SliceMaxNotPositive,
    SliceMinGreaterThanMax,
}

impl SliceBoundsError {
    /// Returns the Python-compatible error message.
    pub const fn message(self) -> &'static str {
        match self {
            Self::SliceMinOutOfRange => "slice_min_sec must be within 0-60 seconds",
            Self::SliceMaxOutOfRange => "slice_max_sec must be within 0-60 seconds",
            Self::SliceMaxNotPositive => "slice_max_sec must be greater than 0 seconds",
            Self::SliceMinGreaterThanMax => {
                "slice_min_sec must be less than or equal to slice_max_sec"
            }
        }
    }
}

impl std::fmt::Display for SliceBoundsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.message())
    }
}

impl std::error::Error for SliceBoundsError {}

/// Validates user-facing slice duration settings.
pub fn validate_slice_bounds(
    slice_min_sec: f64,
    slice_max_sec: f64,
) -> Result<(), SliceBoundsError> {
    if !(SLICE_DURATION_MIN_SEC..=SLICE_DURATION_MAX_SEC).contains(&slice_min_sec) {
        return Err(SliceBoundsError::SliceMinOutOfRange);
    }
    if !(SLICE_DURATION_MIN_SEC..=SLICE_DURATION_MAX_SEC).contains(&slice_max_sec) {
        return Err(SliceBoundsError::SliceMaxOutOfRange);
    }
    if slice_max_sec <= 0.0 {
        return Err(SliceBoundsError::SliceMaxNotPositive);
    }
    if slice_min_sec > slice_max_sec {
        return Err(SliceBoundsError::SliceMinGreaterThanMax);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURES: &str = include_str!("../../../../fixtures/slice_bounds_validation.tsv");

    fn parse_fixture_number(value: &str) -> f64 {
        match value {
            "nan" => f64::NAN,
            "inf" => f64::INFINITY,
            "-inf" => f64::NEG_INFINITY,
            _ => value.parse().unwrap(),
        }
    }

    #[test]
    fn slice_bounds_follow_parity_fixture_table() {
        for (line_number, line) in FIXTURES.lines().enumerate() {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let mut fields = line.split('\t');
            let slice_min_sec = parse_fixture_number(fields.next().unwrap());
            let slice_max_sec = parse_fixture_number(fields.next().unwrap());
            let outcome = fields.next().unwrap();
            let expected_message = fields.next().unwrap_or("");

            match (outcome, validate_slice_bounds(slice_min_sec, slice_max_sec)) {
                ("ok", Ok(())) => {}
                ("err", Err(error)) => assert_eq!(
                    error.message(),
                    expected_message,
                    "fixture line {}",
                    line_number + 1
                ),
                ("ok", Err(error)) => {
                    panic!(
                        "fixture line {} failed unexpectedly: {}",
                        line_number + 1,
                        error
                    )
                }
                ("err", Ok(())) => {
                    panic!("fixture line {} passed unexpectedly", line_number + 1)
                }
                _ => panic!("fixture line {} has unknown outcome", line_number + 1),
            }
        }
    }

    #[test]
    fn slice_bounds_accept_valid_pairs() {
        for (slice_min_sec, slice_max_sec) in [(0.0, 0.1), (5.0, 10.0), (60.0, 60.0)] {
            assert_eq!(validate_slice_bounds(slice_min_sec, slice_max_sec), Ok(()));
        }
    }

    #[test]
    fn slice_bounds_reject_invalid_pairs_with_python_messages() {
        let cases = [
            (
                -0.5,
                10.0,
                SliceBoundsError::SliceMinOutOfRange,
                "slice_min_sec must be within 0-60 seconds",
            ),
            (
                60.5,
                60.0,
                SliceBoundsError::SliceMinOutOfRange,
                "slice_min_sec must be within 0-60 seconds",
            ),
            (
                5.0,
                -0.5,
                SliceBoundsError::SliceMaxOutOfRange,
                "slice_max_sec must be within 0-60 seconds",
            ),
            (
                5.0,
                60.5,
                SliceBoundsError::SliceMaxOutOfRange,
                "slice_max_sec must be within 0-60 seconds",
            ),
            (
                0.0,
                0.0,
                SliceBoundsError::SliceMaxNotPositive,
                "slice_max_sec must be greater than 0 seconds",
            ),
            (
                1.0,
                0.0,
                SliceBoundsError::SliceMaxNotPositive,
                "slice_max_sec must be greater than 0 seconds",
            ),
            (
                2.0,
                1.0,
                SliceBoundsError::SliceMinGreaterThanMax,
                "slice_min_sec must be less than or equal to slice_max_sec",
            ),
        ];

        for (slice_min_sec, slice_max_sec, expected_error, expected_message) in cases {
            let error = validate_slice_bounds(slice_min_sec, slice_max_sec).unwrap_err();
            assert_eq!(error, expected_error);
            assert_eq!(error.message(), expected_message);
            assert_eq!(error.to_string(), expected_message);
        }
    }

    #[test]
    fn slice_bounds_preserve_python_check_order() {
        assert_eq!(
            validate_slice_bounds(-1.0, -1.0),
            Err(SliceBoundsError::SliceMinOutOfRange)
        );
        assert_eq!(
            validate_slice_bounds(1.0, 0.0),
            Err(SliceBoundsError::SliceMaxNotPositive)
        );
        assert_eq!(
            validate_slice_bounds(f64::NAN, f64::NAN),
            Err(SliceBoundsError::SliceMinOutOfRange)
        );
        assert_eq!(
            validate_slice_bounds(1.0, f64::NAN),
            Err(SliceBoundsError::SliceMaxOutOfRange)
        );
    }

    #[test]
    fn slice_bounds_reject_non_finite_values_like_python_range_checks() {
        let cases = [
            (f64::INFINITY, 10.0, SliceBoundsError::SliceMinOutOfRange),
            (
                f64::NEG_INFINITY,
                10.0,
                SliceBoundsError::SliceMinOutOfRange,
            ),
            (10.0, f64::INFINITY, SliceBoundsError::SliceMaxOutOfRange),
            (
                10.0,
                f64::NEG_INFINITY,
                SliceBoundsError::SliceMaxOutOfRange,
            ),
            (f64::NAN, 10.0, SliceBoundsError::SliceMinOutOfRange),
            (10.0, f64::NAN, SliceBoundsError::SliceMaxOutOfRange),
        ];

        for (slice_min_sec, slice_max_sec, expected_error) in cases {
            assert_eq!(
                validate_slice_bounds(slice_min_sec, slice_max_sec),
                Err(expected_error)
            );
        }
    }
}
