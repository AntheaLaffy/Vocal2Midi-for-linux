# slicer_rms_db_window_split_core Bootstrap

## Boundary

`slicer_rms_db_window_split_core` covers the deterministic RMS-dB and
sliding-window split behavior from `inference/API/slicer_api.py`.

The unit covers:

- `get_rms_db` mono/stereo averaging;
- librosa-compatible center-padded RMS for synthetic arrays, reused from the
  already verified `slicer_rms_and_default_core` Rust helper;
- `20 * log10(clip(rms, 1e-10, None))` scalar conversion;
- `_sliding_window_split` max-length early return;
- `librosa.time_to_frames` and `librosa.frames_to_time` scalar behavior used by
  the splitter;
- latest threshold-below cut selection;
- first-min local-min fallback when no threshold-safe cut exists;
- sample slicing, offset assignment, tail append, and stereo channel slicing;
- the current legacy `cut_type` unbound-local error when the search window is
  empty after clamping.

The unit explicitly does not cover heuristic policy orchestration,
grid-search parameter scoring, default silence `Slicer` internals, segment
merge internals, pitch/RMVPE smart slicing, `librosa.pyin`, ProcessPoolExecutor
behavior, audio decoding, filesystem writes, CLI parsing, GUI, Web, or model
execution.

## Dependency Expansion

`inference/API/slicer_api.py` imports `librosa`, `numpy`, `itertools`,
`functools`, `ProcessPoolExecutor`, and the local `Slicer`.

For this split unit, the required third-party behavior is limited to:

- `librosa.feature.rms(..., center=True)` for synthetic array RMS;
- `librosa.time_to_frames` and `librosa.frames_to_time` scalar conversions;
- NumPy mean, clip, log10, `where`, and `argmin` ordering semantics.

Dependency evidence:

- `pyproject.toml` and `requirements.txt` declare `librosa` and `numpy<2.0.0`.
- `uv.lock` records `librosa==0.11.0` depending on `numpy`, `scipy`,
  `soundfile`, `soxr`, `numba`, and other broad audio packages.
- `third_party/sources/manifest.json` records source coverage for
  `librosa-0.11.0`, `numpy-1.26.4`, `scipy-1.17.1`, `soundfile-0.14.0`,
  `soxr-1.1.0`, `numba-0.66.0`, and `llvmlite-0.48.0`.
- `third_party/native_sources/manifest.json` records native source coverage for
  the broader audio/native stack.

The Rust implementation is a narrow hand-written replacement. It does not add a
Rust audio crate, ndarray dependency, Python bridge, or runtime router.

Within the Rust workspace, `slicer_window::rms_db` depends on
`slicer_default::get_rms`. That internal dependency is intentional:
`slicer_rms_and_default_core` already verified center-padded RMS framing, while
this unit verifies the `get_rms_db` channel averaging, clip/log conversion, and
the sliding-window policy that consumes RMS dB values.

## Seam

Use the default rewrite seam:

- kind: independent Rust library
- crate: `rewrite-in-rust/rust/crates/v2m-core`
- module: `slicer_window`
- runtime owner: legacy Python
- bridge dependencies: none

The Rust surface accepts synthetic mono or channel-major stereo `Waveform`
payloads shared with `slicer_segment`.

## Fixture Harness

Python/Rust parity uses:

```text
rewrite-in-rust/fixtures/slicer_rms_db_window_split_core.jsonl
```

The legacy Python side is checked by:

```bash
uv run python rewrite-in-rust/bootstrap/check_slicer_rms_db_window_split_core.py
```

The Rust side is checked by:

```bash
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml slicer_window
```

The fixture table includes direct `get_rms_db` coverage for mono, canceling
stereo, non-canceling stereo, no-split sliding-window behavior, threshold and
local-min split behavior, stereo slicing, and the legacy empty-window error.

## Repeated-Call Behavior

The selected helpers are deterministic for fixed synthetic waveform inputs and
parameters. They do not depend on filesystem state, model state, process pools,
audio decoder state, CLI parser state, global method wrappers, or network
state.

## Rollback

Rollback is keeping production imports unchanged:

```text
inference.API.slicer_api.get_rms_db
inference.API.slicer_api._sliding_window_split
```

No production caller should import Rust slicer-window helpers until a promotion
record defines payload validation, numeric tolerance, diagnostics, and rollback
selection.
