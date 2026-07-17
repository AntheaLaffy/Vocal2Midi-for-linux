# 0051 - Confirm Slicer Grid Search Boundary

Date: 2026-07-17

## Context

After `slicer_rms_db_window_split_core` and `slicer_heuristic_policy_core` were
verified, the next manifest unit is `slicer_grid_search_policy_core`, split
from the former wide `slicer_heuristic_grid_core` candidate by record 0047.

Dependency expansion of `inference/API/slicer_api.py::grid_search_slice` showed
that the remaining grid behavior is a local policy layer:

- candidate threshold/min-length product order;
- construction of `Slicer` with candidate and caller parameters;
- skip-on-exception and skip-empty behavior;
- short/long/count scoring;
- strict best-score update and first-tie retention;
- empty output when no candidate succeeds.

The default silence `Slicer` internals are already covered by
`slicer_rms_and_default_core`.

## Decision

Confirm `slicer_grid_search_policy_core` as a grid policy unit.

Use fake `Slicer` outputs in the Python checker so the fixture harness proves
grid policy without re-covering default RMS/silence slicing. Keep a composed
Rust helper that calls the verified Rust `Slicer` dependency as a smoke path,
but do not introduce a production Python/Rust bridge.

## Consequences

The unit should write:

- `rewrite-in-rust/fixtures/slicer_grid_search_policy_core.jsonl`;
- `rewrite-in-rust/bootstrap/check_slicer_grid_search_policy_core.py`;
- `rewrite-in-rust/dependencies/slicer_grid_search_policy_core.yaml`;
- `rewrite-in-rust/bootstrap/slicer_grid_search_policy_core.md`;
- `rewrite-in-rust/rust/crates/v2m-core/src/slicer_grid.rs`.

The next unit after review remains `slicer_pitch_override_core` unless
dependency discovery re-cuts the remaining slicer smart-slicing boundary.

## Reversal

Rollback is keeping `inference.API.slicer_api.grid_search_slice` as the runtime
owner. No production bridge is introduced.
