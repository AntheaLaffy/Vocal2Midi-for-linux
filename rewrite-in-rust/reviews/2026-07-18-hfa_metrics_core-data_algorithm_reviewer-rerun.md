# hfa_metrics_core - data_algorithm_reviewer

Date: 2026-07-18
Decision: pass

## Findings

No findings.

## Evidence Reviewed

- Unit and role: reviewed exactly `hfa_metrics_core` as
  `data_algorithm_reviewer`. The unit remains a fixture-backed Rust library
  implementation with legacy Python as runtime owner in `manifest.yaml:1666`.
  I did not edit production Rust/Python code and did not mark the manifest
  verified.
- Fixed rounding finding: the Python reference uses `round(..., 6)` for
  vlabeler ratio, IoU, boundary distance, boundary ratio, and weighted boundary
  ratio at `inference/HubertFA/tools/metrics.py:102`,
  `inference/HubertFA/tools/metrics.py:156`,
  `inference/HubertFA/tools/metrics.py:162`,
  `inference/HubertFA/tools/metrics.py:172`,
  `inference/HubertFA/tools/metrics.py:255`,
  `inference/HubertFA/tools/metrics.py:278`, and
  `inference/HubertFA/tools/metrics.py:305`. Rust now routes those public
  computations through `python_round_6` at
  `rust/crates/v2m-core/src/hfa_metrics.rs:239`,
  `rust/crates/v2m-core/src/hfa_metrics.rs:327`,
  `rust/crates/v2m-core/src/hfa_metrics.rs:348`,
  `rust/crates/v2m-core/src/hfa_metrics.rs:462`,
  `rust/crates/v2m-core/src/hfa_metrics.rs:511`, and
  `rust/crates/v2m-core/src/hfa_metrics.rs:573`; the helper implements
  six-decimal half-even tie handling at
  `rust/crates/v2m-core/src/hfa_metrics.rs:588`. The fixture
  `fixtures/hfa_metrics_core.jsonl:10` exercises the prior failure through
  public `BoundaryEditDistance.compute()` with `5e-07 -> 0.0`.
- Fixed IoU list finding: Python list-mode IoU iterates requested phonemes at
  `inference/HubertFA/tools/metrics.py:169`. Rust now exposes
  `IntersectionOverUnion::compute_list` as an ordered
  `Vec<(String, Result<Option<f64>, HfaMetricError>)>` at
  `rust/crates/v2m-core/src/hfa_metrics.rs:352`, so caller order and repeated
  requests are representable. The direct Rust test at
  `rust/crates/v2m-core/src/hfa_metrics.rs:1057` requests
  `["c", "a", "missing", "a", "b"]` and asserts duplicate-preserving output.
- DP/LCS/IoU/boundary-state algorithms: `VlabelerEditsCount` uses the same
  truncating O(m*n) dynamic-programming table and insertion/delete/move costs
  as Python at `rust/crates/v2m-core/src/hfa_metrics.rs:152` versus
  `inference/HubertFA/tools/metrics.py:35`. LCS uses the same O(m*n) table and
  target-side decrement tie policy at
  `rust/crates/v2m-core/src/hfa_metrics.rs:370` versus
  `inference/HubertFA/tools/metrics.py:187`. IoU span accumulation remains
  linear in adjacent span counts at
  `rust/crates/v2m-core/src/hfa_metrics.rs:263`. Boundary distance and ratio
  state transitions preserve mismatch returns, LCS fallback, empty-target
  errors, weighted penalty, and reset quirks at
  `rust/crates/v2m-core/src/hfa_metrics.rs:436`,
  `rust/crates/v2m-core/src/hfa_metrics.rs:491`, and
  `rust/crates/v2m-core/src/hfa_metrics.rs:542`.
- `ndarray` use: `ndarray::Array1` is imported only for the one-dimensional
  absolute time-difference sum at
  `rust/crates/v2m-core/src/hfa_metrics.rs:8` and
  `rust/crates/v2m-core/src/hfa_metrics.rs:582`. This matches the narrow
  numeric-layer allowance in `records/0099-implement-hfa-metrics-core.md` and
  does not claim broad NumPy compatibility.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_hfa_metrics_core.py`: passed;
  validated 12 `hfa_metrics_core` fixtures.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_metrics`:
  passed; 2 targeted tests passed, including fixture parity and direct
  duplicate-preserving IoU list output.
- `cargo clippy --manifest-path rewrite-in-rust/rust/Cargo.toml -p v2m-core --all-targets -- -D warnings`:
  passed.

## Residual Risk

The vlabeler DP and LCS implementations remain O(m*n) memory/time, matching the
Python algorithms. No benchmark or large-input performance envelope is proven
by this fixture gate, so callers should continue treating this as parity for
fixture-sized or modest metric tiers unless a later optimization unit changes
the algorithm with benchmark evidence.

The half-even rounding helper is fixture-proven for the prior public tie case
and covered through all shared metric callers by the fixture replay test. This
review did not exhaustively fuzz Python `round(value, 6)` over arbitrary f64
inputs.

## Promotion Note

This `data_algorithm_reviewer` rerun does not block promotion. Coordinator state
updates still belong to the coordinator after considering all required
`hfa_metrics_core` reviews; this report does not mark the manifest verified.
