# 0099 - Implement HFA Metrics Core

Date: 2026-07-18

## Change

Reimplemented the fixture-bound HubertFA metric helpers in Rust under
`v2m-core::hfa_metrics`:

- synthetic `Point` and `PointTier` with `CustomPointTier.addPoint`
  `bisect_left` ordering, duplicate-time insertion before existing points, and
  no min/max or duplicate validation
- `VlabelerEditsCount` and `VlabelerEditRatio` dynamic-programming behavior,
  unequal-length truncation, repeated-target insertion penalty, denominator,
  rounding, empty defaults, and reset
- `IntersectionOverUnion` span accumulation, one-point tiers, list/string/dict
  compute behavior, missing phonemes, and zero-union quirks
- LCS match-pair computation with legacy repeated-label tie behavior
- `BoundaryEditDistance`, `BoundaryEditRatio`, and
  `BoundaryEditRatioWeighted`, including mismatch returns, LCS fallback,
  error-phoneme accumulation, duration/count/error accumulation, weighted
  penalty, default `1.0` cases, empty-target `IndexError`, and inherited reset
  gaps

The fixture checker at `bootstrap/check_hfa_metrics_core.py` writes and
validates Python-generated JSONL fixtures at
`fixtures/hfa_metrics_core.jsonl`. Rust tests replay the same JSONL cases.

## Dependency Note

Added `ndarray = "0.17"` to `v2m-core` after Cargo resolved it as
`ndarray 0.17.2`. Usage is intentionally narrow: the metrics module uses
`Array1` only for the one-dimensional absolute time-difference sum in
`BoundaryEditDistance`, matching the accepted small numeric layer guidance
without introducing broad NumPy compatibility.

## Verification

Passed:

- `uv run python rewrite-in-rust/bootstrap/check_hfa_metrics_core.py --write`
- `uv run python rewrite-in-rust/bootstrap/check_hfa_metrics_core.py`
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_metrics_core`
- `cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check`
- `cargo clippy --manifest-path rewrite-in-rust/rust/Cargo.toml --all-targets --all-features -- -D warnings`

## State

`hfa_metrics_core` is `reimplemented`. Python remains the runtime owner; no
bridge, caller route, TextGrid IO, NumPy compatibility layer, or production
Python behavior was changed.

## Required Reviews

- `dependency_bootstrap_reviewer`
- `stage_behavior_reviewer`
- `data_algorithm_reviewer`
