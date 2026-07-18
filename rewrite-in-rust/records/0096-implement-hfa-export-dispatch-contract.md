# 0096 Implement HFA Export Dispatch Contract

## Unit

`hfa_export_dispatch_contract`

## Summary

Added a Rust dispatch module for `Exporter.export` and the default/status slice
of `InferenceBase.export`. The module uses injected sinks so it preserves
legacy Python runtime ownership while making dispatch behavior independently
testable.

## Behavior Captured

- Python membership dispatch over iterable, string, mapping-key, empty, and
  `None` format inputs.
- Case-sensitive checks for lowercase `textgrid` and `htk`.
- Fixed TextGrid-before-HTK call order regardless of caller input order.
- Duplicate and unknown formats are ignored by membership semantics.
- `Exporter.export(None)` raises the Python-compatible `TypeError` before any
  sink call.
- `InferenceBase.export(..., output_format=None)` defaults to `['textgrid']`;
  explicit empty formats stay empty.
- Final status output is returned only after downstream success.
- Downstream TextGrid errors short-circuit HTK and suppress status output.

## Implementation Notes

`rust/crates/v2m-core/src/hfa_export_dispatch.rs` exposes:

- `HfaExportFormats` for Python-shaped membership inputs.
- `HfaExportSink` for injected TextGrid/HTK sinks.
- `export_with_sink` for `Exporter.export` dispatch.
- `inference_export_with_sink` for `InferenceBase.export` default/status policy.
- `HfaPlanningExportSink`, an adapter that composes the existing
  `hfa_textgrid_export` and `hfa_htk_export` planner public APIs without moving
  production Python callers.

No production Python bridge, GUI/API routing, or serializer internals were
changed.

## Verification

- `uv run python rewrite-in-rust/bootstrap/check_hfa_export_dispatch_contract.py`
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_export_dispatch_contract`

## Required Reviews

Keep the unit at `reimplemented` until independent review passes:

- `dependency_bootstrap_reviewer`
- `stage_behavior_reviewer`
- `error_tracing_reviewer`
