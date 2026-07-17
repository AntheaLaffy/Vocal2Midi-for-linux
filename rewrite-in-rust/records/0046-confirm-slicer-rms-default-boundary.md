# 0046 - Confirm slicer RMS/default boundary

Date: 2026-07-17

## Decision

Confirm `slicer_rms_and_default_core` as a pure RMS/default silence slicing
unit over synthetic mono and channel-major stereo arrays.

The unit covers:

- `get_rms`;
- `Slicer.__init__`;
- `Slicer._apply_slice`;
- `Slicer.slice`;
- `inference.API.slicer_api.default_slice` caller parameters.

The unit does not reimplement heuristic/grid slicing, pitch/RMVPE smart slicing,
`get_rms_db`, `_sliding_window_split`, `librosa.pyin`, model execution, audio
IO, multiprocessing, CLI parsing, or filesystem behavior.

## Rationale

The owning modules sit near heavy audio and inference dependencies, but the
default slicer itself is deterministic array framing, scalar conversion, a
silence-tag state machine, and waveform slicing. The project already contains a
local `get_rms` copy, so a narrow Rust implementation can be fixture-bound
without adding package-level librosa or ndarray parity.

Heuristic/grid and pitch/RMVPE slicing have separate manifest units because
they add different dependencies and policies: dB RMS helpers, librosa frame/time
conversion, grid scoring, unvoiced-mask splitting, process pools, and pitch
fallback decisions.

## Verification

Legacy fixture check:

```bash
uv run python rewrite-in-rust/bootstrap/check_slicer_rms_and_default_core.py
```

Future Rust writer check:

```bash
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml slicer_default
```

## Rollback

Keep `inference.slicer.slicer2` and `inference.API.slicer_api.default_slice` as
runtime owners until a promotion record introduces Rust integration.
