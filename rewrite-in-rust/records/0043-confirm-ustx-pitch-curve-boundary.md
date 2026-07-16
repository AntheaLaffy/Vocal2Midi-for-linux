# 0043 - Confirm USTX pitch curve boundary

Date: 2026-07-16

## Decision

Confirm `ustx_pitch_curve_core` as a deterministic algorithm unit over synthetic
RMVPE output:

```text
notes + RmvpeResult(time_step_seconds, midi_pitch) + tempo -> pitd xs/ys
```

Keep RMVPE model loading, ONNX Runtime execution, waveform preprocessing,
f0-to-midi interpolation, `voiced_mask` creation, USTX YAML assembly, and
filesystem writes legacy-owned.

## Rationale

The selected `_build_pitd_curve` path is scalar/list processing. It uses NumPy
only for NaN checks, clipping, and array size/iteration behavior. Pulling in a
Rust ndarray, ONNX Runtime, or audio stack would broaden the unit beyond the
behavior needed for parity.

This boundary also keeps pitch-curve rendering separate from the already
verified `ustx_project_export_core` unit, which covers `save_ustx(...,
rmvpe_result=None)`.

## Verification

Legacy fixture check:

```bash
uv run python rewrite-in-rust/bootstrap/check_ustx_pitch_curve_core.py
```

Future Rust writer check:

```bash
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml ustx_pitch_curve
```

## Rollback

Keep `inference.API.ustx_api._build_pitd_curve` and
`inference.API.ustx_api.save_ustx` as runtime owners until a promotion record
introduces Rust integration.
