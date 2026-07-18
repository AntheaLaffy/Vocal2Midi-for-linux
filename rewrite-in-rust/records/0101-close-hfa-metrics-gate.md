# 0101 - Close HFA Metrics Gate

Date: 2026-07-18

## Decision

`hfa_metrics_core` is verified.

The Rust implementation remains an independent `v2m-core::hfa_metrics` library
seam. Python `inference.HubertFA.tools.metrics` remains the runtime owner. No
bridge, production caller route, TextGrid IO, model execution, or broad NumPy
compatibility layer changed in this gate.

## Evidence

Final fixture state:

- `rewrite-in-rust/fixtures/hfa_metrics_core.jsonl` contains 12 Python 3.12
  synthetic point-tier cases covering `CustomPointTier` ordering,
  `VlabelerEditsCount`, `VlabelerEditRatio`, `IntersectionOverUnion`, LCS
  helpers, `BoundaryEditDistance`, `BoundaryEditRatio`, and
  `BoundaryEditRatioWeighted`.
- The final fixture set includes a Python half-even rounding tie through public
  `BoundaryEditDistance.compute()` where `0.0000005` rounds to `0.0`.
- Rust also has a direct public API test proving `IntersectionOverUnion` list
  mode preserves caller order and repeated phoneme requests.

Required review reports:

- `rewrite-in-rust/reviews/2026-07-18-hfa_metrics_core-dependency_bootstrap_reviewer.md`
- `rewrite-in-rust/reviews/2026-07-18-hfa_metrics_core-behavior_reviewer-rerun.md`
- `rewrite-in-rust/reviews/2026-07-18-hfa_metrics_core-data_algorithm_reviewer-rerun.md`

The failed initial behavior and data/algorithm reports remain durable audit
evidence:

- `rewrite-in-rust/reviews/2026-07-18-hfa_metrics_core-behavior_reviewer.md`
- `rewrite-in-rust/reviews/2026-07-18-hfa_metrics_core-data_algorithm_reviewer.md`

Those initial reports found Python half-even rounding drift and IoU list-mode
ordering drift. Record 0100 fixed both; the reruns passed without findings.

## Dependency Note

`ndarray 0.17.2` is now in `v2m-core` and is used narrowly for the
one-dimensional absolute time-difference sum in `BoundaryEditDistance`. This
starts the accepted Rust numeric-array layer for later model-adjacent work
without claiming general NumPy replacement in this unit.

## Verification

Coordinator checks run before closeout:

```bash
uv run python rewrite-in-rust/bootstrap/check_hfa_metrics_core.py
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_metrics
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml
cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check
cargo clippy --manifest-path rewrite-in-rust/rust/Cargo.toml --all-targets --all-features -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --manifest-path rewrite-in-rust/rust/Cargo.toml --no-deps
uv run python -m py_compile inference/HubertFA/tools/metrics.py rewrite-in-rust/bootstrap/check_hfa_metrics_core.py
uv run python scripts/audit_vendored_sources.py
git diff --check
```

All passed.

## Reversal

Rollback remains keeping Python `inference.HubertFA.tools.metrics` as runtime
owner. Because no production route changed, reversal is removing the
independent Rust module, fixture/checker, `ndarray` dependency if no later unit
uses it, and manifest verification entries if this seam is later re-cut.
