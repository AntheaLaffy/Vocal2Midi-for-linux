# ustx_pitch_curve_core - data_algorithm_reviewer

Date: 2026-07-16
Decision: pass-with-followups

## Findings

- Severity: low
- Location: rewrite-in-rust/fixtures/ustx_pitch_curve_core.jsonl:1
- Issue: The current fixture table proves the main numeric path, but it does not directly pin every algorithm-sensitive class named by the boundary: unsorted note sorting, half-even `.5` tick/snap ties, cross-note same-`x` append/replace, and non-finite `time_step_seconds` policy are still mostly covered by source inspection rather than golden rows.
- Evidence: The six durable fixtures cover empty inputs, NaN pitch skipping, edge trimming, cents clipping, duplicate replacement inside one pending note, short-gap interpolation, median/adaptive smoothing, and multi-note pending flush at `rewrite-in-rust/fixtures/ustx_pitch_curve_core.jsonl:1` through `rewrite-in-rust/fixtures/ustx_pitch_curve_core.jsonl:6`. Rust consumes exactly that table at `rewrite-in-rust/rust/crates/v2m-core/src/ustx_pitch_curve.rs:272` through `rewrite-in-rust/rust/crates/v2m-core/src/ustx_pitch_curve.rs:305`. The implementation itself sorts notes, rounds half-even, snaps to `U_CURVE_INTERVAL`, and replaces duplicate output positions at `rewrite-in-rust/rust/crates/v2m-core/src/ustx_pitch_curve.rs:64`, `rewrite-in-rust/rust/crates/v2m-core/src/ustx_pitch_curve.rs:106`, `rewrite-in-rust/rust/crates/v2m-core/src/ustx_pitch_curve.rs:107`, `rewrite-in-rust/rust/crates/v2m-core/src/ustx_pitch_curve.rs:111`, and `rewrite-in-rust/rust/crates/v2m-core/src/ustx_pitch_curve.rs:156`. A targeted Python probe confirmed the legacy cross-note same-`x` replacement shape with `tiny_cross_note_same_x [0] [-100]`.
- Required fix: Before runtime promotion, add focused fixture or unit-test rows for unsorted notes, an exact `.5` tick tie and `.5` snap tie, cross-note same-`x` replacement, and the chosen non-finite time-step policy.

- Severity: low
- Location: rewrite-in-rust/rust/crates/v2m-core/src/ustx_pitch_curve.rs:54
- Issue: Rust rejects non-finite `time_step_seconds` as `InvalidTimeStep`, while the legacy helper returns an empty curve for a NaN time step because every sample time fails the note-window comparison. This is acceptable for the confirmed seam because RMVPE produces a finite constant step, but the narrower Rust contract should be made explicit before promotion.
- Evidence: Rust returns `InvalidTimeStep` at `rewrite-in-rust/rust/crates/v2m-core/src/ustx_pitch_curve.rs:54` through `rewrite-in-rust/rust/crates/v2m-core/src/ustx_pitch_curve.rs:56`, with direct unit coverage at `rewrite-in-rust/rust/crates/v2m-core/src/ustx_pitch_curve.rs:307` through `rewrite-in-rust/rust/crates/v2m-core/src/ustx_pitch_curve.rs:322`. Legacy computes sample time as `i * rmvpe.time_step_seconds` at `inference/API/ustx_api.py:115` and skips samples outside the note window at `inference/API/ustx_api.py:126`. The review probe produced `nan_time_step [] []`. The normal RMVPE path returns `HOP_LENGTH / SAMPLE_RATE` at `inference/API/rmvpe_api.py:99` through `inference/API/rmvpe_api.py:101`, and the bootstrap record keeps this unit on already-computed synthetic RMVPE payloads at `rewrite-in-rust/bootstrap/ustx_pitch_curve_core.md:5` through `rewrite-in-rust/bootstrap/ustx_pitch_curve_core.md:7`.
- Required fix: Document finite `time_step_seconds` as a Rust precondition or add an explicit parity decision and fixture row for NaN/Infinity time steps.

## Accepted Data/Algorithm Evidence

- Note ordering and window advancement match the legacy loop. Python sorts by `note.onset`, advances `note_idx` on `offset <= t`, flushes pending points when the note index changes, and skips samples outside the current note window at `inference/API/ustx_api.py:108` through `inference/API/ustx_api.py:127`. Rust mirrors that sequence at `rewrite-in-rust/rust/crates/v2m-core/src/ustx_pitch_curve.rs:64` through `rewrite-in-rust/rust/crates/v2m-core/src/ustx_pitch_curve.rs:91`.
- Edge trimming, NaN skipping, tick conversion, UCurveInterval snapping, cents clipping, and duplicate pending replacement match the source algorithm at `inference/API/ustx_api.py:128` through `inference/API/ustx_api.py:144` and `rewrite-in-rust/rust/crates/v2m-core/src/ustx_pitch_curve.rs:93` through `rewrite-in-rust/rust/crates/v2m-core/src/ustx_pitch_curve.rs:117`.
- Post-processing order matches legacy: fill short gaps, median filter with radius 2, then adaptive smoothing with in-place mutation order. Python implements this at `inference/API/ustx_api.py:47` through `inference/API/ustx_api.py:101`; Rust mirrors it at `rewrite-in-rust/rust/crates/v2m-core/src/ustx_pitch_curve.rs:147` through `rewrite-in-rust/rust/crates/v2m-core/src/ustx_pitch_curve.rs:223`.
- Output shape is correct for the narrowed seam: Python returns `tuple[list[int], list[int]]` at `inference/API/ustx_api.py:104` through `inference/API/ustx_api.py:147`; Rust returns `UstxPitchCurve { xs: Vec<i64>, ys: Vec<i64> }` at `rewrite-in-rust/rust/crates/v2m-core/src/ustx_pitch_curve.rs:18` through `rewrite-in-rust/rust/crates/v2m-core/src/ustx_pitch_curve.rs:23` and `rewrite-in-rust/rust/crates/v2m-core/src/ustx_pitch_curve.rs:120` through `rewrite-in-rust/rust/crates/v2m-core/src/ustx_pitch_curve.rs:121`.
- Complexity is acceptable: sorting notes is `O(n log n)`, iterating pitch samples is `O(m)`, duplicate replacement is constant time, short-gap fill is linear in emitted points plus bounded interpolation, and median filtering sorts windows of at most five values, so it remains effectively linear in generated curve points.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_ustx_pitch_curve_core.py`: pass.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml ustx_pitch_curve`: pass; 2 tests passed.
- `uv run python -c "... targeted _build_pitd_curve probes ..."`: pass; confirmed legacy outputs for `inf_clip`, `tiny_cross_note_same_x`, and `nan_time_step`.
- Source and fixture inspection: checked `inference/API/ustx_api.py`, `inference/API/rmvpe_api.py`, `rewrite-in-rust/rust/crates/v2m-core/src/ustx_pitch_curve.rs`, `rewrite-in-rust/fixtures/ustx_pitch_curve_core.jsonl`, dependency/bootstrap records, the boundary record, and manifest entry.

## Residual Risk

The Rust implementation is fixture-compatible for the current synthetic RMVPE seam, but fixture coverage is not yet exhaustive for every numeric resolver edge. The remaining risk is promotion-time drift if a future bridge feeds unsorted, overlapping, or non-finite synthetic rows without adding the focused tests listed above.

## Promotion Note

This data/algorithm role does not block coordinator verification for `ustx_pitch_curve_core`. The coordinator should still require the remaining required review roles before changing the unit state. The listed follow-ups should be completed before runtime promotion or before treating arbitrary synthetic pitch-curve payloads as supported public input.
