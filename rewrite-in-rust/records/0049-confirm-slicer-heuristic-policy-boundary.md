# 0049 - Confirm Slicer Heuristic Policy Boundary

Date: 2026-07-17

## Context

After `slicer_rms_db_window_split_core` was verified, the next manifest unit is
`slicer_heuristic_policy_core`, split from the former wide
`slicer_heuristic_grid_core` candidate.

`heuristic_slice` combines deterministic orchestration with dependencies that
are already or separately verified:

- default silence slicing through `Slicer`;
- long-segment splitting through `_sliding_window_split`;
- tiny and short segment merging through merge helpers;
- grid-search scoring and pitch/RMVPE smart slicing in separate units.

## Decision

Confirm `slicer_heuristic_policy_core` as a policy orchestration unit.

The unit covers `heuristic_slice` stage ordering, fixed dependency parameters,
long-segment split dispatch, parent-offset adjustment, retention of exact-min /
short / ultra-short segments, offset sorting before merge, and merge-helper
handoff.

The unit does not re-cover default `Slicer`, RMS-dB/window splitting, merge
helper internals, grid search, pitch/RMVPE smart slicing, ProcessPoolExecutor,
audio IO, CLI parsing, GUI, Web, or model execution.

Use fake `Slicer` and fake `_sliding_window_split` in the Python checker so the
policy surface can be verified without re-testing already accepted dependencies.

## Consequences

The Rust implementation can stay a narrow library module that composes verified
Rust helpers and exposes a dependency-injected policy function for fixture
tests. Runtime production ownership stays with Python.

The next unit after review is still `slicer_grid_search_policy_core` unless
review findings require a follow-up pass.

## Rollback

Rollback is keeping `inference.API.slicer_api.heuristic_slice` as the runtime
owner. No production bridge is introduced by this record.
