# slicer_pitch_override_core - behavior_reviewer rerun2

Date: 2026-07-17
Role: behavior_reviewer
Unit: slicer_pitch_override_core

## Findings

- Severity: none
- Location: `rewrite-in-rust/rust/crates/v2m-core/src/slicer_pitch.rs:180`, `rewrite-in-rust/rust/crates/v2m-core/src/slicer_pitch.rs:181`, `rewrite-in-rust/rust/crates/v2m-core/src/slicer_pitch.rs:188`, `rewrite-in-rust/fixtures/slicer_pitch_override_core.jsonl:5`, `rewrite-in-rust/bootstrap/check_slicer_pitch_override_core.py:44`, `rewrite-in-rust/rust/crates/v2m-core/src/slicer_pitch.rs:675`
- Issue: Prior medium mismatch is closed. `pitch_based_split_with_override` now preserves Python's short-segment early return before requiring a supplied override or pyin-owned path, and both Python and Rust fixture paths cover `voiced_flag_override: null`.
- Evidence: Rust now computes `total_sec` and returns the original segment at `rewrite-in-rust/rust/crates/v2m-core/src/slicer_pitch.rs:180` through `rewrite-in-rust/rust/crates/v2m-core/src/slicer_pitch.rs:186`, before resolving `voiced_flag_override` at line 188. Fixture `pitch_split_short_no_override_early_return` at `rewrite-in-rust/fixtures/slicer_pitch_override_core.jsonl:5` uses `voiced_flag_override:null` and `voiced_flag_override_step_sec:null`. The Python checker preserves null through `optional_bool_array` at `rewrite-in-rust/bootstrap/check_slicer_pitch_override_core.py:44`, and Rust uses `parse_optional_bool_vec` for pitch split requests at `rewrite-in-rust/rust/crates/v2m-core/src/slicer_pitch.rs:675`. A direct Rust probe for the prior failing short/no-override case returned `Ok([Segment { offset: 0.0, waveform: Mono([0.0, 1.0, 2.0]) }])`.
- Required fix: none.

- Severity: none
- Location: `rewrite-in-rust/fixtures/slicer_pitch_override_core.jsonl:8`, `inference/API/slicer_api.py:503`, `inference/API/slicer_api.py:507`, `rewrite-in-rust/rust/crates/v2m-core/src/slicer_pitch.rs:247`, `rewrite-in-rust/rust/crates/v2m-core/src/slicer_pitch.rs:249`, `rewrite-in-rust/rust/crates/v2m-core/src/slicer_pitch.rs:691`
- Issue: Prior low voiced-frame clamp coverage finding is closed.
- Evidence: The direct voiced clamp row now has `sr=1`, `hop_length=1`, `min_len_sec=2.0`, `max_len_sec=4.0`, and `direct_voiced_flag` length `3`, so the first split window computes `end_frame=4` and must clamp to `3`. Python clamps at `inference/API/slicer_api.py:507`; Rust mirrors the clamp at `rewrite-in-rust/rust/crates/v2m-core/src/slicer_pitch.rs:249`. The Rust fixture harness uses the already-computed-voiced-flags seam for rows with `direct_voiced_flag` at `rewrite-in-rust/rust/crates/v2m-core/src/slicer_pitch.rs:691`.
- Required fix: none.

## Decision

pass

Prior medium mismatch: closed.
Prior low clamp coverage finding: closed.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_slicer_pitch_override_core.py`: pass.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml slicer_pitch`: pass; 3 focused `slicer_pitch` tests passed, 89 filtered in `v2m_core`, and 0 matching bridge tests ran.
- `git diff --check`: pass.
- Direct Rust probe for the prior short/no-override mismatch: pass; returned the original segment instead of `PyinUnsupported`.

## Residual Risk

This rerun only reviewed the coordinator fix after the failing rerun report. Broader promotion risks remain the same fixture-bound exclusions: `librosa.pyin`, RMVPE/model execution, real process scheduling, external payload validation, GUI/Web/CLI routing, and production bridge error mapping are still out of scope.

## Promotion Note

This behavior rerun does not block closing the prior follow-up. Coordinator state still needs the other required review roles and normal promotion discipline before marking runtime ownership changes.
