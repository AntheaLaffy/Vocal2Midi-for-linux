# slicer_pitch_override_core Bootstrap

## Boundary

`slicer_pitch_override_core` covers supplied-voiced-mask smart slicing in
`inference/API/slicer_api.py`.

The unit covers:

- `get_pitch_curve` override path when `voiced_flag_override` is present and
  `voiced_flag_override_step_sec > 0`;
- global-time mask lookup with `segment_offset_sec`, NumPy round behavior, and
  index clipping;
- `_pitch_based_split` early return for segments already within max length;
- long-segment window frame conversion and clipping;
- midpoint cut in the first longest continuous unvoiced run;
- RMS fallback cut when no unvoiced frame exists;
- `_split_wrapper` passing parent segment offset into the split helper and then
  adding that offset to returned local chunks;
- `pitch_based_slice` fixed pre-slicer parameters, long/short/tiny
  classification, sorting after parallel split results, and handoff to verified
  tiny/short merge helpers.

The unit explicitly does not cover `librosa.pyin`, RMVPE model execution,
default `Slicer` internals, RMS-dB internals, merge-helper internals,
`ProcessPoolExecutor` scheduling semantics, audio decoding, filesystem writes,
CLI parsing, GUI, Web, or production routing.

## Dependency Expansion

The selected behavior depends on local helpers already represented by separate
units:

- `slicer_rms_and_default_core`: default `Slicer` construction and slicing;
- `slicer_rms_db_window_split_core`: RMS-dB fallback calculation;
- `slicer_segment_merge_core`: tiny and short segment merge helpers.

The source module imports `librosa`, `numpy`, `itertools`, `functools`, and
`ProcessPoolExecutor`. This unit uses only deterministic equivalents for
frame/time conversion, grouping, round/clip indexing, and policy control flow.
`librosa.pyin`, RMVPE/model inference, audio IO, multiprocessing mechanics, and
native package behavior remain legacy-owned.

Dependency evidence:

- `rewrite-in-rust/manifest.yaml` scopes this unit to supplied boolean
  voiced-mask fixtures and explicitly says not to call `librosa.pyin` or RMVPE.
- `inference/API/slicer_api.py::get_pitch_curve` bypasses `librosa.pyin` when a
  supplied voiced-mask override and positive step are present.
- `inference/API/slicer_api.py::_pitch_based_split` and `pitch_based_slice`
  are local policy over voiced flags, RMS fallback arrays, and segment
  dictionaries once dependencies are injected.

## Seam

Use the default rewrite seam:

- kind: independent Rust library
- crate: `rewrite-in-rust/rust/crates/v2m-core`
- module: `slicer_pitch`
- runtime owner: legacy Python
- bridge dependencies: none

The Rust surface exposes fixture-bound functions for override resampling,
single-segment pitch split, and outer pitch policy orchestration, plus a
composed helper using verified default slicer and RMS dependencies. No Python
runtime bridge is introduced.

## Fixture Harness

Python/Rust parity uses:

```text
rewrite-in-rust/fixtures/slicer_pitch_override_core.jsonl
```

The legacy Python side is checked by:

```bash
uv run python rewrite-in-rust/bootstrap/check_slicer_pitch_override_core.py
```

The checker:

- calls `get_pitch_curve` with supplied boolean masks for override indexing;
- calls `_pitch_based_split` with supplied masks and a fake `get_rms_db` only
  for RMS fallback policy rows;
- monkeypatches `get_pitch_curve` for direct split-frame clamp rows where the
  reviewed behavior is `_pitch_based_split`'s clipping of already-computed
  voiced/RMS arrays;
- replaces `Slicer`, `_pitch_based_split`, and `ProcessPoolExecutor` with fakes
  for outer `pitch_based_slice` orchestration, while leaving verified merge
  helpers real.

The Rust side is checked by:

```bash
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml slicer_pitch
```

## Repeated-Call Behavior

The selected policy is deterministic for fixed synthetic waveforms, supplied
voiced masks, RMS fallback vectors, split outputs, and parameters. It does not
depend on filesystem state, model state, audio decoder state, CLI parser state,
global method wrappers, network state, or actual process scheduling.

## Rollback

Rollback is keeping production imports unchanged:

```text
inference.API.slicer_api.get_pitch_curve
inference.API.slicer_api._pitch_based_split
inference.API.slicer_api.pitch_based_slice
```

No production caller should import Rust pitch helpers until a promotion record
defines external waveform/mask validation, multiprocessing replacement or
avoidance, logging text, dependency wiring, and Python-facing error mapping.
