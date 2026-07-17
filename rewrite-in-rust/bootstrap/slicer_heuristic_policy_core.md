# slicer_heuristic_policy_core Bootstrap

## Boundary

`slicer_heuristic_policy_core` covers the deterministic policy layer in
`inference/API/slicer_api.py::heuristic_slice`.

The unit covers:

- pre-slicer construction parameters: caller threshold, caller
  `min_silence_len_ms`, fixed `min_interval=200`, and fixed
  `max_sil_kept=100`;
- long segment detection using legacy `len(segment["waveform"]) / sr`
  behavior;
- calls into the already verified sliding-window split dependency with
  `frame_length=2048`, `hop_length=512`, caller min/max bounds, and caller
  split threshold;
- adding the parent segment offset to each sliding-window subchunk;
- retaining exact-min, short, and ultra-short segments before merge;
- sorting chunks by offset before merge;
- handing chunks to the already verified tiny and short merge helpers.

The unit explicitly does not cover `Slicer` RMS/default internals,
`get_rms_db`, `_sliding_window_split` internals, `_merge_tiny_chunks`,
`_merge_short_segments`, `grid_search_slice`, pitch/RMVPE smart slicing,
`librosa.pyin`, ProcessPoolExecutor behavior, audio decoding, filesystem
writes, CLI parsing, GUI, Web, or model execution.

## Dependency Expansion

The selected policy code depends on local helpers already represented by
separate units:

- `slicer_rms_and_default_core`: default `Slicer` construction and slicing;
- `slicer_rms_db_window_split_core`: RMS-dB and sliding-window splitting;
- `slicer_segment_merge_core`: tiny and short segment merge helpers.

The source module imports `librosa`, `numpy`, `itertools`, `functools`, and
`ProcessPoolExecutor`, but this unit does not need package-level audio or native
dependency parity. Those imports belong to adjacent units or model/audio paths.

Dependency evidence:

- `rewrite-in-rust/records/0047-split-slicer-heuristic-grid-boundary.md`
  split the former heuristic/grid candidate into RMS/window, heuristic policy,
  and grid policy units.
- `rewrite-in-rust/manifest.yaml` marks
  `slicer_rms_db_window_split_core`, `slicer_rms_and_default_core`, and
  `slicer_segment_merge_core` as verified dependencies.
- `inference/API/slicer_api.py::heuristic_slice` is local control flow over
  segment dictionaries once the helper dependencies are injected.

## Seam

Use the default rewrite seam:

- kind: independent Rust library
- crate: `rewrite-in-rust/rust/crates/v2m-core`
- module: `slicer_heuristic`
- runtime owner: legacy Python
- bridge dependencies: none

The Rust surface exposes a policy function over fixture-provided pre-sliced
segments and split outputs, plus an actual helper that composes the verified
Slicer and sliding-window Rust dependencies. No Python runtime bridge is
introduced.

## Fixture Harness

Python/Rust parity uses:

```text
rewrite-in-rust/fixtures/slicer_heuristic_policy_core.jsonl
```

The legacy Python side is checked by:

```bash
uv run python rewrite-in-rust/bootstrap/check_slicer_heuristic_policy_core.py
```

The checker replaces `Slicer` and `_sliding_window_split` with fakes so the
policy layer can be proven without re-covering the verified dependencies.
Merge helpers remain real because their exact handoff and ordering affect the
observable heuristic output.

The Rust tests also include a focused `heuristic_slice` smoke case using real
verified dependencies so the public composed helper is compile- and
runtime-proven before review.

The Rust side is checked by:

```bash
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml slicer_heuristic
```

## Repeated-Call Behavior

The selected policy is deterministic for fixed synthetic segments, split
outputs, and parameters. It does not depend on filesystem state, model state,
process pools, audio decoder state, CLI parser state, global method wrappers,
or network state.

## Rollback

Rollback is keeping production imports unchanged:

```text
inference.API.slicer_api.heuristic_slice
```

No production caller should import Rust heuristic helpers until a promotion
record defines waveform payload validation, dependency wiring, logging text, and
error mapping.
