# quantization_smart_duration_dp - behavior_reviewer

Date: 2026-07-15
Unit: quantization_smart_duration_dp
Role: behavior_reviewer
Decision: pass

## Findings

No behavior-parity findings.

The reviewed Rust unit stays within the confirmed smart-duration boundary. The
Python source returns without mutation for empty inputs or non-positive
`quantization_step`, sorts by onset when quantization runs, builds duration
candidates, applies the NumPy-backed DP with first-min `argmin`, places rests,
and mutates only note onset/offset at `inference/quant/quantization.py:727`.

The Rust implementation mirrors those public behaviors in the independent
library function: no-op return at
`rewrite-in-rust/rust/crates/v2m-core/src/quant.rs:121`, stable onset sorting at
`rewrite-in-rust/rust/crates/v2m-core/src/quant.rs:126`, half-even tick/raw
duration conversion and candidate construction at
`rewrite-in-rust/rust/crates/v2m-core/src/quant.rs:132`, Vec-backed DP and
first-min scans at `rewrite-in-rust/rust/crates/v2m-core/src/quant.rs:150`,
duration decode and rest placement at
`rewrite-in-rust/rust/crates/v2m-core/src/quant.rs:201`, and final onset/offset
mutation with pitch/lyric preservation at
`rewrite-in-rust/rust/crates/v2m-core/src/quant.rs:229`.

The shared fixture table covers the behavior cases required by the bootstrap:
empty and step-zero no-op, onset sorting with metadata preservation, half-even
tick/grid rounding, zero-duration clamping, preferred/non-preferred duration
penalty behavior, first-min tie cases, rest thresholds, overlap rest clamp, and
non-default tempo/step at
`rewrite-in-rust/fixtures/quantization_smart_duration_dp.tsv:2`. The Python
checker validates the fixture table against legacy `_quantize_notes_smart` at
`rewrite-in-rust/bootstrap/check_quantization_smart_duration_dp.py:83`, and the
Rust tests consume the same table at
`rewrite-in-rust/rust/crates/v2m-core/src/quant.rs:726`.

Rollback and production routing remain legacy-owned. The manifest keeps this
unit in `reimplemented` state with `current_owner: legacy` and rollback to
`inference.quant.quantization._quantize_notes_smart` at
`rewrite-in-rust/manifest.yaml:213`. The bootstrap records no production bridge
for this unit and keeps runtime ownership with legacy Python at
`rewrite-in-rust/bootstrap/quantization_smart_duration_dp.md:64`. A production
route scan found only the legacy Python definition and dispatch call in
`inference/quant/quantization.py:727` and
`inference/quant/quantization.py:796`.

## Checks

- `UV_CACHE_DIR=/tmp/v2m-uv-cache uv run python rewrite-in-rust/bootstrap/check_quantization_smart_duration_dp.py`: passed.
- `CARGO_TARGET_DIR=/tmp/v2m-rust-target cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml smart_duration`: passed; 2 tests passed, 0 failed, 31 filtered out.
- `rg -n "v2m_core|rewrite-in-rust/rust|quantize_notes_smart|_quantize_notes_smart|quantization_smart_duration_dp|from .*rust|import .*rust" application inference gui scripts web_server.py web_task_manager.py --glob '!**/__pycache__/**'`: inspected; only legacy Python `_quantize_notes_smart` definition and `quantize_notes` dispatch matched.

## Evidence

- Boundary record confirms this unit covers only `_quantize_notes_smart` and
  keeps phrase DP, Bayesian quantization, public dispatch, GUI/Web/application
  settings, and runtime promotion separate at
  `rewrite-in-rust/records/0008-confirm-smart-duration-boundary.md:21`.
- Dependency record confirms the narrow Vec-backed implementation, no bridge
  dependencies, required fixture coverage, and legacy-kept runtime routing at
  `rewrite-in-rust/dependencies/quantization_smart_duration_dp.yaml:4`.
- The Rust crate remains an independent test surface and is not wired into the
  Python runtime at `rewrite-in-rust/rust/crates/v2m-core/src/lib.rs:1`.

## Residual Risk

This review proves behavior parity only for the fixture-bound finite-input
surface. Invalid numeric input mapping for NaN, infinities, and non-positive
tempo is explicitly left to future promotion work at
`rewrite-in-rust/dependencies/quantization_smart_duration_dp.yaml:42`.

The Rust unit models the note shape used by the quantization fixtures: onset,
offset, pitch, and lyric. Any future Python/Rust bridge must separately verify
object mapping for additional note attributes before production routing changes.

## Promotion Note

This behavior review does not block coordinator state update for the behavior
gate. The manifest still lists `data_algorithm_reviewer` as a required review
for this unit at `rewrite-in-rust/manifest.yaml:222`, so the coordinator should
only mark the unit verified after the required algorithm review also passes.
