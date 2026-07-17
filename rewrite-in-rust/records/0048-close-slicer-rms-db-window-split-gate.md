# 0048 - Close Slicer RMS-dB Window Split Gate

Date: 2026-07-17

## Context

`slicer_rms_db_window_split_core` was split from the former wide
`slicer_heuristic_grid_core` candidate and reimplemented as an independent
Rust library unit.

Required reviews completed:

- dependency/bootstrap review:
  `rewrite-in-rust/reviews/2026-07-17-slicer_rms_db_window_split_core-dependency_bootstrap_reviewer.md`
- behavior review:
  `rewrite-in-rust/reviews/2026-07-17-slicer_rms_db_window_split_core-behavior_reviewer.md`
- data/algorithm review:
  `rewrite-in-rust/reviews/2026-07-17-slicer_rms_db_window_split_core-data_algorithm_reviewer.md`

## Decision

Accept `slicer_rms_db_window_split_core` as verified.

Two review follow-ups were closed in the fixture/bootstrap evidence:

- the internal dependency on `slicer_rms_and_default_core` /
  `slicer_default::get_rms` is now recorded in the dependency and bootstrap
  documents;
- a non-identical, non-canceling stereo `get_rms_db` fixture now proves channel
  averaging outside the silence-floor case.

The remaining data/algorithm note is promotion-gate only: before any runtime
bridge feeds threshold decisions from external waveform payloads, define whether
Rust should intentionally preserve current `f64` comparison behavior or cast /
round RMS-dB values to legacy librosa `float32` behavior for near-threshold
strict `<` cuts.

## Verification

```bash
uv run python rewrite-in-rust/bootstrap/check_slicer_rms_db_window_split_core.py
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml slicer_window
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml
cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check
cargo clippy --manifest-path rewrite-in-rust/rust/Cargo.toml --all-targets --all-features -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --manifest-path rewrite-in-rust/rust/Cargo.toml --no-deps
```

All commands passed during the gate close.

## Rollback

Rollback remains keeping `inference.API.slicer_api.get_rms_db` and
`inference.API.slicer_api._sliding_window_split` as runtime owners. No
production bridge was introduced.
