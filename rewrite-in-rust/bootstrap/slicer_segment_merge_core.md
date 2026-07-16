# slicer_segment_merge_core Bootstrap

## Boundary

`slicer_segment_merge_core` covers deterministic segment waveform manipulation
helpers in `inference/API/slicer_api.py`:

- `_concat_waveforms`;
- `_silence_like`;
- `_segment_duration_sec`;
- `_merged_duration_sec`;
- `_merge_segments`;
- `_merge_short_segments`;
- `_merge_tiny_chunks`.

The unit preserves mono and channel-major stereo shape behavior, inserted
silence length, Python `round` half-even gap sample conversion, offset
preservation, greedy short-segment merge policy, reverse tail merge, recursive
retry condition, leading/body/tail tiny handling, empty input behavior, and the
legacy `_merge_tiny_chunks` stereo behavior that uses `len(waveform) / sr`
instead of `waveform.shape[-1] / sr` for its tiny-duration test.

The unit explicitly does not cover real silence detection, RMS calculation,
default/heuristic/grid/pitch slicing policies, RMVPE, ASR, librosa time/frame
conversion, SoundFile writes, FFmpeg, argparse, multiprocessing, filesystem
behavior, or production bridge wiring.

## Dependency Expansion

`inference/API/slicer_api.py` imports `numpy`, `librosa`, `itertools`,
`functools`, `ProcessPoolExecutor`, and `inference.slicer.slicer2.Slicer`.

The selected merge helpers use only:

- NumPy array shape inspection;
- NumPy zero array allocation with a copied shape;
- NumPy concatenate on axis `0` for mono and axis `-1` for multi-dimensional
  waveforms;
- Python list/dict copying and sorting already supplied by caller fixtures;
- Python float division, `max`, comparison, and `round` behavior.

Dependency evidence:

- `pyproject.toml` and `requirements.txt` declare `numpy`, `librosa`,
  `soundfile`, `scipy`, and ONNX packages because the owning module and adjacent
  slicing paths perform real audio processing and inference outside this unit.
- `uv.lock` records `numpy`, `librosa`, `soundfile`, `scipy`, and ONNX packages.
- `third_party/sources/manifest.json` records source directories for
  `numpy-1.26.4`, `librosa-0.11.0`, `soundfile-0.14.0`, `scipy-1.17.1`, and an
  upstream fallback for `onnxruntime-1.27.0`.
- `third_party/source_audit.json` records the ONNX Runtime upstream source
  fallback and the vendored source audit covers runtime native binaries.

Only NumPy-like shape/slice/concatenate/zero behavior is required for this
contract. The future Rust writer may use a small hand-written waveform enum or
another minimal local representation instead of adding a broad ndarray/audio
dependency.

## Seam

Use the default rewrite seam:

- kind: independent Rust library
- crate: `rewrite-in-rust/rust/crates/v2m-core`
- suggested module: `slicer_segment`
- runtime owner: legacy Python
- bridge dependencies: none

The future Rust surface should expose:

- waveform concat and silence creation for mono/stereo synthetic payloads;
- segment duration and merged duration helpers;
- segment merge preserving offset and inserting rounded silence gaps;
- short-segment and tiny-segment merge helpers returning ordered segment lists.

It should not call Python, parse CLI arguments, load audio, calculate RMS, run
Slicer, run RMVPE/ASR, spawn processes, write files, or expose a runtime router.

## Fixture Harness

Python/Rust parity uses:

```text
rewrite-in-rust/fixtures/slicer_segment_merge_core.jsonl
```

The legacy Python side is checked by:

```bash
uv run python rewrite-in-rust/bootstrap/check_slicer_segment_merge_core.py
```

The future Rust side should be checked by:

```bash
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml slicer_segment
```

## Repeated-Call Behavior

The selected helpers are deterministic for fixed synthetic segment inputs. They
do not depend on filesystem state, audio decoding, model state, Slicer mutable
state, global slicing method replacement, process pools, or CLI parser state.

## Rollback

Rollback is keeping production imports unchanged:

```text
inference.API.slicer_api._concat_waveforms
inference.API.slicer_api._silence_like
inference.API.slicer_api._segment_duration_sec
inference.API.slicer_api._merged_duration_sec
inference.API.slicer_api._merge_segments
inference.API.slicer_api._merge_short_segments
inference.API.slicer_api._merge_tiny_chunks
```

No production caller should import Rust segment helpers until a promotion record
defines the bridge, waveform payload shape, and error mapping.
