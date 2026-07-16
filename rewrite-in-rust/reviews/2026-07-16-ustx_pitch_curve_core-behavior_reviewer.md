# ustx_pitch_curve_core - behavior_reviewer

Date: 2026-07-16
Decision: pass-with-followups

## Findings

- Severity: low
- Location: rewrite-in-rust/rust/crates/v2m-core/src/ustx_pitch_curve.rs:51
- Issue: Rust intentionally rejects non-finite `tempo` and `time_step_seconds`, but this error behavior is not yet mapped to the legacy `save_ustx` caller path.
- Evidence: The Python `_build_pitd_curve` path consumes `rmvpe.time_step_seconds` directly and computes ticks with Python `round` in `_to_ticks` (`inference/API/ustx_api.py:26`, `inference/API/ustx_api.py:115`, `inference/API/ustx_api.py:137`), while Rust returns structured `InvalidTempo`, `InvalidTimeStep`, or `TickOutOfRange` errors (`rewrite-in-rust/rust/crates/v2m-core/src/ustx_pitch_curve.rs:51`, `rewrite-in-rust/rust/crates/v2m-core/src/ustx_pitch_curve.rs:54`, `rewrite-in-rust/rust/crates/v2m-core/src/ustx_pitch_curve.rs:128`). The fixture parity path covers finite synthetic data and the Rust unit covers non-finite rejection, but no promotion bridge exists yet.
- Required fix: Before runtime promotion, define how these Rust errors map to the Python `save_ustx` behavior and add bridge-level tests for invalid tempo/time-step inputs.

- Severity: low
- Location: rewrite-in-rust/fixtures/ustx_pitch_curve_core.jsonl:1
- Issue: Behavior parity is fixture-proven for the confirmed synthetic seam, but the fixture table is not exhaustive for all possible note ordering and invalid-note edge cases.
- Evidence: The current fixtures cover empty notes, empty `midi_pitch`, NaN skipping, edge trimming, cents clipping, duplicate tick replacement, short-gap interpolation, median/adaptive smoothing, and multi-note pending flush (`rewrite-in-rust/fixtures/ustx_pitch_curve_core.jsonl:1`, `rewrite-in-rust/fixtures/ustx_pitch_curve_core.jsonl:6`). The production `save_ustx` caller filters invalid notes before calling `_build_pitd_curve` (`inference/API/ustx_api.py:369`, `inference/API/ustx_api.py:370`, `inference/API/ustx_api.py:409`), so this does not block the current unit, but promotion should keep the same caller precondition explicit.
- Required fix: Add promotion-level fixtures if Rust becomes callable before Python `_finite_notes`, especially for equal-onset ordering and invalid note fields.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_ustx_pitch_curve_core.py`: pass
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml ustx_pitch_curve`: pass, 2 tests
- `rg -n "onnx|ort|librosa|RmvpeTranscriber|yaml|std::fs|File::|Path|python|pyo3|numpy" rewrite-in-rust/rust/crates/v2m-core/src/ustx_pitch_curve.rs rewrite-in-rust/rust/Cargo.toml`: inspected; no RMVPE/ONNX/audio/YAML/filesystem bridge in the Rust pitch-curve unit
- `git diff --check -- rewrite-in-rust/rust/crates/v2m-core/src/ustx_pitch_curve.rs rewrite-in-rust/bootstrap/check_ustx_pitch_curve_core.py rewrite-in-rust/fixtures/ustx_pitch_curve_core.jsonl rewrite-in-rust/dependencies/ustx_pitch_curve_core.yaml rewrite-in-rust/bootstrap/ustx_pitch_curve_core.md rewrite-in-rust/records/0043-confirm-ustx-pitch-curve-boundary.md`: pass

## Residual Risk

The review proves parity for the committed synthetic `RmvpeResult` fixture table and source-level algorithm structure. It does not prove arbitrary RMVPE runtime output distribution, ONNX/model behavior, waveform preprocessing, USTX YAML insertion, or filesystem integration, all of which are explicitly outside this unit boundary.

## Promotion Note

This behavior role does not block promotion evidence for the current reimplemented unit. The coordinator should still require the separate `data_algorithm_reviewer` role before marking `ustx_pitch_curve_core` verified.
