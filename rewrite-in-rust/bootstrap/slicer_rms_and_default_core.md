# slicer_rms_and_default_core Bootstrap

## Boundary

`slicer_rms_and_default_core` covers the deterministic RMS/default silence
slicer behavior in `inference/slicer/slicer2.py` and the default-slice caller
shape in `inference/API/slicer_api.py`.

The unit covers:

- `get_rms` center padding, frame construction, hop downsampling, mean-square,
  and square-root behavior;
- `Slicer.__init__` validation, threshold conversion, and millisecond-to-frame
  conversion;
- mono and channel-major stereo input handling;
- short-input and no-silence whole-waveform behavior;
- leading, middle, and trailing silence tag selection;
- `_apply_slice` offset calculation and waveform slicing.

The unit explicitly does not cover heuristic/grid slicing, `get_rms_db`,
`_sliding_window_split`, pitch/RMVPE smart slicing, `librosa.pyin`, model
execution, audio decoding, SoundFile/libsndfile, FFmpeg, multiprocessing, CLI
parsing, filesystem behavior, or production bridge wiring.

## Dependency Expansion

`inference/slicer/slicer2.py` imports only `numpy`. The local `get_rms`
implementation notes that it was obtained from librosa, but the selected
behavior is already present in project source and does not need package-level
librosa parity.

`inference/API/slicer_api.py` imports heavier slicing and audio dependencies
because adjacent methods use `librosa`, `ProcessPoolExecutor`, RMVPE helpers,
SoundFile, and CLI/audio IO behavior. Those capabilities stay out of this unit.

Dependency evidence:

- `pyproject.toml` and `requirements.txt` declare `librosa`, `numpy`, `scipy`,
  `soundfile`, and ONNX packages for the broader application.
- `uv.lock` records `librosa` and its dependencies including `audioread`,
  `numba`, `numpy`, `scipy`, `soundfile`, and `soxr`.
- `third_party/sources/manifest.json` records sources for `numpy-1.26.4`,
  `librosa-0.11.0`, `scipy-1.17.1`, `soundfile-0.14.0`, `soxr-1.1.0`,
  `numba-0.66.0`, and `llvmlite-0.48.0`.
- `third_party/native_sources/manifest.json` records native coverage for
  OpenBLAS, libsndfile, codec dependencies, and llvmlite's LLVM dependency.

The Rust implementation uses a small fixture-bound waveform and RMS model
instead of adding broad ndarray/audio dependency parity.

## Seam

Use the default rewrite seam:

- kind: independent Rust library
- crate: `rewrite-in-rust/rust/crates/v2m-core`
- suggested module: `slicer_default`
- runtime owner: legacy Python
- bridge dependencies: none

The Rust surface exposes:

- `get_rms`-equivalent output for finite mono fixture samples;
- constructor validation and converted slicer parameters;
- default silence slicing over mono and channel-major stereo fixture payloads.

It should not call Python, import librosa, parse CLI arguments, load audio,
write files, run RMVPE/ASR, spawn processes, or expose a runtime router.

## Fixture Harness

Python/Rust parity uses:

```text
rewrite-in-rust/fixtures/slicer_rms_and_default_core.jsonl
```

The legacy Python side is checked by:

```bash
uv run python rewrite-in-rust/bootstrap/check_slicer_rms_and_default_core.py
```

The fixture table includes direct `default_slice` caller coverage, trailing
silence with EOF-clipped RMS search bounds, and a non-identical stereo-channel
case that proves channel averaging affects silence detection while output
slicing preserves each channel.

The Rust side is checked by a targeted `slicer_default` test:

```bash
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml slicer_default
```

## Repeated-Call Behavior

The selected helpers are deterministic for fixed synthetic waveform inputs and
constructor parameters. They do not depend on filesystem state, audio decoding,
model state, process pools, CLI parser state, or global slicing-method
replacement.

## Rollback

Rollback is keeping production imports unchanged:

```text
inference.slicer.slicer2.get_rms
inference.slicer.slicer2.Slicer
inference.API.slicer_api.default_slice
```

No production caller should import Rust default-slicer helpers until a
promotion record defines waveform payload validation, numeric tolerance, and
error mapping.
