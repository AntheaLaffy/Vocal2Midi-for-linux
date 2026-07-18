# hfa_metrics_core - dependency_bootstrap_reviewer

Date: 2026-07-18
Decision: pass

## Findings

No findings.

## Boundary Decision

Manifest unit boundary: confirmed.

`hfa_metrics_core` is an appropriate single writer unit for dependency/bootstrap
scope. The manifest constrains the unit to HubertFA metric helpers and keeps
runtime ownership with legacy Python, with no bridge or production caller route
claimed (`manifest.yaml:1666`, `manifest.yaml:1674`, `manifest.yaml:1691`).
The bootstrap record excludes PyO3, subprocess routing, TextGrid file IO, model
execution, filesystem effects, and production caller routing
(`bootstrap/hfa_metrics_core.md:10`). Splitting by metric class would duplicate
the same point-tier, LCS, and boundary-state fixture setup without reducing
dependency risk (`bootstrap/hfa_metrics_core.md:76`).

## Evidence Reviewed

- Source expansion is sufficient for the selected seam. The dependency record
  indexes first-layer NumPy/TextGrid sources and explicitly rejects deeper
  OpenBLAS/TextGrid parser expansion because the public call path only reaches
  `np.array`, `np.abs`, `np.sum`, rounding, and point-tier container behavior
  (`dependencies/hfa_metrics_core.yaml:66`, `dependencies/hfa_metrics_core.yaml:70`).
  The source manifest contains `numpy==1.26.4` and `textgrid==1.6.1`
  (`third_party/sources/manifest.json:473`, `third_party/sources/manifest.json:870`),
  and OpenBLAS is separately indexed as a native source but not reached by this
  fixture seam (`third_party/native_sources/manifest.json:389`).
- TextGrid is correctly kept legacy/general. `CustomPointTier.addPoint` overrides
  upstream validation and only uses `bisect_left` plus direct list insertion
  (`inference/HubertFA/tools/metrics.py:7`), while upstream `PointTier.addPoint`
  includes min/max and duplicate-time validation that this custom override
  intentionally bypasses (`third_party/sources/textgrid-1.6.1/textgrid/textgrid.py:344`).
- NumPy/ndarray scope is acceptable. The Python source only uses NumPy in
  `BoundaryEditDistance.update` for one-dimensional time arrays, absolute
  difference, and sum (`inference/HubertFA/tools/metrics.py:246`). The dependency
  policy permits deferring `ndarray` unless introduced as a narrow shared
  numeric helper with fixture evidence (`dependencies/hfa_metrics_core.yaml:38`,
  `dependencies/hfa_metrics_core.yaml:63`). The writer added `ndarray = "0.17"`
  only to `v2m-core` (`rust/crates/v2m-core/Cargo.toml:15`) and uses `Array1`
  only in `absolute_difference_sum` (`rust/crates/v2m-core/src/hfa_metrics.rs:582`),
  so the implementation does not claim broad NumPy compatibility.
- Fixture coverage is adequate for writer readiness. The bootstrap contract
  requires point insertion ordering, vlabeler count/ratio quirks, IoU modes,
  LCS tie behavior, boundary distance/ratio errors, weighted penalty, and reset
  gaps (`bootstrap/hfa_metrics_core.md:40`). The checker defines 11 cases across
  those categories (`bootstrap/check_hfa_metrics_core.py:261`), and Rust tests
  replay the JSONL fixtures rather than hand-coded expected values
  (`rust/crates/v2m-core/src/hfa_metrics.rs:597`, `rust/crates/v2m-core/src/hfa_metrics.rs:1021`).
- Rollback is documented and narrow: keep `inference.HubertFA.tools.metrics` as
  runtime owner, with rollback limited to removing the independent Rust
  module/tests and checker if the boundary is re-cut
  (`bootstrap/hfa_metrics_core.md:82`, `records/0098-bootstrap-hfa-metrics-core.md:68`).

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_hfa_metrics_core.py`: passed;
  validated 11 `hfa_metrics_core` fixtures.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_metrics_core`:
  passed; 1 targeted Rust fixture-parity test passed, 118 filtered.
- `uv run python scripts/audit_vendored_sources.py`: passed; 135 Python
  packages, 41 native-extension packages, 269 foreign runtime native binaries,
  and 0 `third_party` binary artifacts.

## Residual Risk

This review only covers dependency/bootstrap readiness. It does not assert full
behavior parity, numeric-algorithm quality, Rust style, or promotion readiness.
Those remain with the separate required `stage_behavior_reviewer` and
`data_algorithm_reviewer` roles listed for the unit (`manifest.yaml:1675`).

## Promotion Note

This dependency/bootstrap role does not block promotion. The unit should not be
marked verified or promoted from this report alone; coordinator state updates
still depend on the remaining required review roles.
