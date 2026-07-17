# 0047 - Split Slicer Heuristic/Grid Boundary

Date: 2026-07-17

## Context

The provisional `slicer_heuristic_grid_core` unit grouped several behaviors from
`inference/API/slicer_api.py`:

- `get_rms_db`;
- `_sliding_window_split`;
- `heuristic_slice`;
- `grid_search_slice`.

Dependency discovery showed that this grouped unit mixes a reusable RMS/window
split dependency with two policy orchestration layers. The policy layers also
depend on already verified units: `slicer_rms_and_default_core` for the default
`Slicer` behavior and `slicer_segment_merge_core` for tiny/short segment merges.

## Decision

Replace `slicer_heuristic_grid_core` with three smaller units:

- `slicer_rms_db_window_split_core`;
- `slicer_heuristic_policy_core`;
- `slicer_grid_search_policy_core`.

Start implementation with `slicer_rms_db_window_split_core`, because both later
policy units can reuse its RMS-dB and sliding-window behavior.

Keep the Rust seam as an independent library in `v2m-core`. Do not add PyO3,
CLI/subprocess routing, HTTP, a runtime router, ndarray, or a broad audio crate
for this split.

## Consequences

The first split unit can be fixture-bound with synthetic waveform payloads and a
small JSONL checker. Later heuristic and grid units should treat
`slicer_rms_db_window_split_core`, `slicer_rms_and_default_core`, and
`slicer_segment_merge_core` as dependencies rather than re-covering their
internal behavior.

The original wide unit name should not be used as a promotion or review target.
Review evidence should name the smaller unit being reviewed.

## Reversal

Rollback is keeping `inference/API/slicer_api.py` as the runtime owner. No
production bridge is introduced by this split.
