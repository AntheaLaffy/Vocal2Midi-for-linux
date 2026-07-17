# slicer_grid_search_policy_core Bootstrap

## Boundary

`slicer_grid_search_policy_core` covers the deterministic policy layer in
`inference/API/slicer_api.py::grid_search_slice`.

The unit covers:

- threshold iteration order `[-45, -40, -35, -30, -25, -20]`;
- min-length iteration order `[8000, 6000, 4000, 2500, 1500]`;
- `itertools.product(thresholds, min_lengths_ms)` ordering;
- `Slicer` construction with caller sample rate, candidate threshold,
  candidate min length, caller `min_interval_ms`, and caller
  `max_sil_kept_ms`;
- skipping constructor and slice exceptions;
- skipping empty chunk results;
- scoring chunk durations with legacy `len(chunk["waveform"]) / sr`
  behavior;
- scoring target chunk count with legacy `len(waveform) / sr`;
- short, long, and chunk-count penalty terms;
- strict `score < best_score` update, so equal-score ties keep the first
  candidate;
- returning `[]` when all candidates fail or return empty results.

The unit explicitly does not cover default `Slicer` RMS/silence internals,
`get_rms_db`, `_sliding_window_split`, heuristic slicing, pitch/RMVPE smart
slicing, `librosa.pyin`, `ProcessPoolExecutor`, audio decoding, filesystem
writes, CLI parsing, GUI, Web, or model execution.

## Dependency Expansion

The selected policy depends on local behavior already represented by separate
units:

- `slicer_rms_and_default_core`: default `Slicer` construction and slicing;
- `slice_method_and_bounds_contract`: caller-level method and custom-bound
  dispatch into grid slicing.

The source module imports `librosa`, `numpy`, `itertools`, `functools`, and
`ProcessPoolExecutor`, but this unit only requires `itertools.product` ordering
and list/numeric control flow. Audio, RMS, pitch, multiprocessing, and native
package behavior remain outside this boundary.

Dependency evidence:

- `rewrite-in-rust/records/0047-split-slicer-heuristic-grid-boundary.md`
  split the former heuristic/grid candidate into RMS/window, heuristic policy,
  and grid policy units.
- `rewrite-in-rust/manifest.yaml` marks `slicer_rms_and_default_core` as a
  verified dependency.
- `inference/API/slicer_api.py::grid_search_slice` is local control flow over
  dependency-provided `Slicer.slice` results once `Slicer` is injected.

## Seam

Use the default rewrite seam:

- kind: independent Rust library
- crate: `rewrite-in-rust/rust/crates/v2m-core`
- module: `slicer_grid`
- runtime owner: legacy Python
- bridge dependencies: none

The Rust surface exposes a policy function over fixture-provided Slicer outputs,
plus an actual helper that composes the verified Rust `Slicer` dependency. No
Python runtime bridge is introduced.

## Fixture Harness

Python/Rust parity uses:

```text
rewrite-in-rust/fixtures/slicer_grid_search_policy_core.jsonl
```

The legacy Python side is checked by:

```bash
uv run python rewrite-in-rust/bootstrap/check_slicer_grid_search_policy_core.py
```

The checker replaces `Slicer` with a fake so the grid-search policy can be
proven without re-covering default silence-slicer internals. The fixture table
covers:

- full parameter order and constructor arguments;
- constructor exception skip and slice exception skip;
- empty chunk skip;
- scoring exception skip through a zero-sample-rate fixture;
- short, long, and chunk-count scoring;
- strict first-tie retention;
- all-failing empty output;
- stereo `len()` scoring behavior.

The Rust side is checked by:

```bash
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml slicer_grid
```

## Repeated-Call Behavior

The selected policy is deterministic for fixed waveform length, candidate
outputs, and parameters. It does not depend on filesystem state, model state,
process pools, audio decoder state, CLI parser state, global method wrappers,
or network state.

## Rollback

Rollback is keeping production imports unchanged:

```text
inference.API.slicer_api.grid_search_slice
```

No production caller should import Rust grid helpers until a promotion record
defines waveform payload validation, dependency wiring, logging text, and error
mapping.
