# 0045 - Confirm slicer segment merge boundary

Date: 2026-07-17

## Decision

Confirm `slicer_segment_merge_core` as a pure segment waveform manipulation unit
over synthetic mono/stereo arrays and segment dictionaries.

The unit covers:

- `_concat_waveforms`;
- `_silence_like`;
- `_segment_duration_sec`;
- `_merged_duration_sec`;
- `_merge_segments`;
- `_merge_short_segments`;
- `_merge_tiny_chunks`.

The unit does not reimplement RMS calculation, default silence slicing,
heuristic/grid slicing policies, pitch/RMVPE smart slicing, model execution,
audio IO, multiprocessing, CLI parsing, or filesystem behavior.

## Rationale

The owning module imports heavy audio and inference dependencies, but these
helpers require only deterministic array shape, concatenation, zero-padding,
duration, and list-merge behavior. Keeping this boundary narrow avoids pulling
librosa, Slicer internals, RMVPE, ProcessPoolExecutor behavior, ASR, SoundFile,
FFmpeg, or ONNX Runtime into a small compatibility seam.

The fixtures intentionally preserve legacy edge behavior, including Python
round-to-even gap sample conversion and `_merge_tiny_chunks` using
`len(waveform) / sr` for its tiny-duration test.

## Verification

Legacy fixture check:

```bash
uv run python rewrite-in-rust/bootstrap/check_slicer_segment_merge_core.py
```

Future Rust writer check:

```bash
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml slicer_segment
```

## Rollback

Keep `inference.API.slicer_api` merge helpers as runtime owners until a
promotion record introduces Rust integration.
