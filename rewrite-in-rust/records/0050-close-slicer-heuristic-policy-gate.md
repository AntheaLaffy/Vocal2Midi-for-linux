# 0050 - Close Slicer Heuristic Policy Gate

Date: 2026-07-17

## Context

`slicer_heuristic_policy_core` was split from the former wide
`slicer_heuristic_grid_core` candidate and reimplemented as an independent
Rust library policy unit.

Required reviews completed:

- dependency/bootstrap review:
  `rewrite-in-rust/reviews/2026-07-17-slicer_heuristic_policy_core-dependency_bootstrap_reviewer.md`
- behavior review:
  `rewrite-in-rust/reviews/2026-07-17-slicer_heuristic_policy_core-behavior_reviewer.md`
- data/algorithm review:
  `rewrite-in-rust/reviews/2026-07-17-slicer_heuristic_policy_core-data_algorithm_reviewer.md`

## Decision

Accept `slicer_heuristic_policy_core` as verified.

The dependency/bootstrap review follow-ups were closed after the initial report:

- an exact `seg_dur == max_len_sec` fixture now proves that the heuristic policy
  only splits when duration is strictly greater than `max_len_sec`;
- a focused Rust `heuristic_slice` smoke test now exercises the public composed
  helper through the verified default slicer/window/merge dependencies.

The remaining risks are promotion-gate only: future runtime wiring must define
waveform payload validation, logging text parity, dependency error mapping,
and inherited numeric policy from `slicer_rms_db_window_split_core`.

## Verification

```bash
uv run python rewrite-in-rust/bootstrap/check_slicer_heuristic_policy_core.py
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml slicer_heuristic
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml
cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check
cargo clippy --manifest-path rewrite-in-rust/rust/Cargo.toml --all-targets --all-features -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --manifest-path rewrite-in-rust/rust/Cargo.toml --no-deps
```

All commands passed during the gate close.

## Rollback

Rollback remains keeping `inference.API.slicer_api.heuristic_slice` as the
runtime owner. No production bridge was introduced.
