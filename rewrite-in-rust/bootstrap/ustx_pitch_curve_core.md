# ustx_pitch_curve_core Bootstrap

## Boundary

`ustx_pitch_curve_core` covers the deterministic `_build_pitd_curve` behavior in
`inference/API/ustx_api.py` when supplied with already-computed
`RmvpeResult.midi_pitch` samples.

The compatibility surface is:

- sorting notes by onset before pitch matching
- returning empty `xs` and `ys` when notes are empty or `midi_pitch` is empty
- sample time calculation from `index * time_step_seconds`
- advancing to the current note as time crosses note offsets
- flushing pending pitch points when the note index changes
- skipping samples outside the current note window
- skipping NaN midi-pitch samples
- trimming note edges by `min(0.025, duration * 0.15)`
- converting sample time to USTX ticks with Python half-even rounding
- snapping ticks to `UCurveInterval == 5`
- clipping cents offset to `-1200..1200`
- replacing the last pending point when two samples snap to the same x
- filling short x gaps up to 12 curve steps
- median filtering with radius 2
- adaptive smoothing with threshold 75 cents and blend 0.7

RMVPE model loading, ONNX Runtime execution, waveform preprocessing, f0-to-midi
interpolation, `voiced_mask` creation, USTX YAML project assembly, and
filesystem writes stay legacy-owned.

## Dependency Expansion

`inference/API/ustx_api.py` imports:

- stdlib: `dataclasses.dataclass`, `pathlib.Path`, `typing.Any`
- third party: `numpy`, `yaml`
- local: `inference.API.rmvpe_api.RmvpeResult`

The selected pitch-curve path uses:

- `numpy.isnan`
- `numpy.clip`
- Python/NumPy half-even rounding through `round`
- `RmvpeResult.time_step_seconds`
- `RmvpeResult.midi_pitch.size` and iterable midi-pitch sample values

Dependency evidence:

- `pyproject.toml` and `requirements.txt` include `numpy<2.0.0`.
- `uv.lock` records `numpy==1.26.4`.
- `third_party/sources/manifest.json` records
  `third_party/sources/numpy-1.26.4`.
- `third_party/native_sources/manifest.json` records OpenBLAS source coverage
  for NumPy/SciPy, but this unit needs only scalar NaN, clip, array size, and
  iteration behavior, not BLAS or ndarray kernels.
- `third_party/source_audit.json` reports all foreign runtime native binaries
  covered and zero `third_party` binary artifacts.

Do not add a broad ndarray, ONNX, librosa, or audio dependency for this unit.
The Rust implementation should take plain note rows, `time_step_seconds`, and a
slice of optional/f64 midi-pitch samples.

## Seam

Use the default rewrite seam:

- kind: independent Rust library
- crate: `rewrite-in-rust/rust/crates/v2m-core`
- suggested module: `ustx_pitch_curve`
- runtime owner: legacy Python
- bridge dependencies: none

The Rust surface should return `xs` and `ys` vectors, plus a structured error
only if non-finite tempo or impossible tick conversion is intentionally rejected.
It should not create USTX YAML, write files, load models, call Python, or expose
a runtime router.

## Fixture Harness

Python/Rust parity uses:

```text
rewrite-in-rust/fixtures/ustx_pitch_curve_core.jsonl
```

Each fixture contains:

- `tempo`
- `time_step_seconds`
- notes with string numeric fields
- `midi_pitch` string values so NaN can be represented
- expected `xs` and `ys`

The legacy Python side is checked by:

```bash
uv run python rewrite-in-rust/bootstrap/check_ustx_pitch_curve_core.py
```

The future Rust side should be checked by:

```bash
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml ustx_pitch_curve
```

## Repeated-Call Behavior

The renderer is deterministic for fixed notes, tempo, time step, and midi-pitch
samples. It must not depend on model state, filesystem state, GUI state, Web
state, ONNX Runtime session state, or global audio backend state.

## Rollback

Rollback is keeping all production imports unchanged:

```text
inference.API.ustx_api._build_pitd_curve
```

No production Python caller should import a Rust pitch-curve renderer until a
later promotion record defines YAML integration and error mapping.
