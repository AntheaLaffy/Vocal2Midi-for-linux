# 0042 - Confirm USTX Project Export Boundary

## Context

`inference/API/ustx_api.py::save_ustx` combines two deterministic export
surfaces:

- project and note YAML assembly for the OpenUtau USTX project;
- optional pitch-deviation curve generation from `RmvpeResult` arrays.

The pitch-curve path has separate helper logic for NaN skipping, note-boundary
flushes, short-gap interpolation, median filtering, adaptive smoothing,
duplicate tick replacement, and clipping. That logic is already represented by
the separate `ustx_pitch_curve_core` manifest unit.

## Decision

Confirm `ustx_project_export_core` as a fixture-bound library unit for
`save_ustx(..., rmvpe_result=None)`:

- finite-note filtering;
- stable note ordering;
- tick conversion;
- minimum duration;
- tone clamping;
- fallback and UTF-8 lyrics;
- fixed pitch/vibrato note defaults;
- fixed expression descriptors;
- root project metadata, tempo/time-signature metadata, one track, one voice
  part, empty curves, and empty wave parts;
- PyYAML-compatible output for the selected project tree.

Keep legacy-owned or separate:

- `_build_pitd_curve` and all RMVPE-derived `pitd` curve rendering;
- RMVPE model loading and ONNX Runtime inference;
- production filesystem writes, parent directory creation, warning/status
  printing, and runtime export routing;
- broad PyYAML and NumPy package parity beyond the selected USTX tree.

## Consequences

The writer can implement a narrow Rust renderer with synthetic note fixtures and
golden Python YAML output. The runtime Python path remains unchanged, so
filesystem and user-visible warning/error mapping are not promoted in this
unit.

This boundary keeps USTX project shape reviewable while preserving a separate
algorithm review surface for pitch-curve generation.

## Reversal

Rollback is keeping `inference.API.ustx_api.save_ustx` as the runtime owner. No
production bridge is introduced by this record.
