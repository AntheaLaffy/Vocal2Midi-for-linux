# slicer_rms_db_window_split_core - dependency_bootstrap_reviewer

Date: 2026-07-17
Decision: pass-with-followups

## Findings

- Severity: low
- Location: rewrite-in-rust/rust/crates/v2m-core/src/slicer_window.rs:8
- Issue: The Rust implementation depends on `slicer_default::get_rms`, but the unit's dependency/bootstrap records do not name `slicer_rms_and_default_core` or `slicer_default::get_rms` as an internal prerequisite.
- Evidence: `slicer_window` imports `PadMode` and `get_rms` from `slicer_default` at rewrite-in-rust/rust/crates/v2m-core/src/slicer_window.rs:8, while the unit dependency record declares no bridge dependencies at rewrite-in-rust/dependencies/slicer_rms_db_window_split_core.yaml:12 and describes the hand-written replacement as a direct librosa/numpy subset at rewrite-in-rust/dependencies/slicer_rms_db_window_split_core.yaml:22. The bootstrap record only names shared `Waveform` payloads from `slicer_segment` at rewrite-in-rust/bootstrap/slicer_rms_db_window_split_core.md:63.
- Required fix: Before coordinator promotion, record `slicer_rms_and_default_core` / `slicer_default::get_rms` as an internal Rust prerequisite for this unit, or state explicitly why reusing that verified helper is part of this unit's accepted seam.

- Severity: low
- Location: rewrite-in-rust/fixtures/slicer_rms_db_window_split_core.jsonl:2
- Issue: The fixture set claims stereo averaging coverage for `get_rms_db`, but the only stereo RMS-dB fixture cancels the channels to zero and primarily proves clipping to the -200 dB floor.
- Evidence: The public policy requires preserving channel averaging at rewrite-in-rust/manifest.yaml:1044, and Python averages channels before RMS at inference/API/slicer_api.py:234. The Rust implementation averages channel-major samples at rewrite-in-rust/rust/crates/v2m-core/src/slicer_window.rs:145. Current stereo coverage is the cancel-to-floor RMS case at rewrite-in-rust/fixtures/slicer_rms_db_window_split_core.jsonl:2 plus an identical-channel sliding split case at rewrite-in-rust/fixtures/slicer_rms_db_window_split_core.jsonl:6, so a non-identical, non-canceling average is not fixture-proven for this unit.
- Required fix: Add one `rms_db` fixture with non-identical, non-canceling stereo channels before promotion, or explicitly delegate that exact averaging proof to the already verified `slicer_rms_and_default_core` evidence.

## Boundary Decision

The manifest unit boundary is confirmed with followups. The split from the wider heuristic/grid candidate is justified by record 0047: `get_rms_db` and `_sliding_window_split` are the shared deterministic dependency, while heuristic orchestration and grid scoring are separate policy units at rewrite-in-rust/records/0047-split-slicer-heuristic-grid-boundary.md:22. The seam stays an independent `v2m-core` library with legacy Python as runtime owner, no PyO3/router/subprocess bridge, and no broad audio crate at rewrite-in-rust/bootstrap/slicer_rms_db_window_split_core.md:50.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_slicer_rms_db_window_split_core.py`: passed.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml slicer_window`: passed, 2 tests passed and 0 failed.
- `uv run python scripts/audit_vendored_sources.py`: passed, reporting 135 Python packages, 41 native-extension packages, 269 foreign runtime native binaries, and 0 third_party binary artifacts.

## Residual Risk

Behavior and data/algorithm review are still required for numeric edge cases beyond the JSONL fixture table. This review did not approve runtime promotion, external waveform validation, logging text parity beyond the captured legacy error path, or later heuristic/grid policy behavior.

## Promotion Note

This dependency/bootstrap role does not block promotion if the two low-severity followups are resolved or explicitly accepted by the coordinator. The unit still needs the remaining required review roles before any coordinator state update to `verified`.
