# quantization_candidate_primitives Bootstrap

## Boundary

`quantization_candidate_primitives` covers only pure helper behavior from
`inference/quant/quantization.py`:

- `_resolve_dp_grid_step`
- `_resolve_segment_shift_candidates`
- `_nearest_candidate`
- `_mod_distance`
- `_dist_grid`
- `_candidate_values`
- `_build_note_pair`
- `_annotate_pairs_with_gap`
- `_build_candidate_pairs`
- `_build_duration_candidates`
- `_build_gap_candidates`

The compatibility surface is:

- fallback DP grid step `30` when the requested step is missing or non-positive
- default DP segment shifts `(0, 60, 120)` for grid steps up to `30`
- scaled segment shifts `{0, grid_step // 4, grid_step // 2}` for larger grids
- nearest-candidate choice by absolute distance, preserving the first list item
  on ties
- modular and grid distances using Python positive-modulo behavior for positive
  modulo/step values
- Python half-even rounding when raw ticks are converted to candidate centers
- sorted unique candidate tick values
- note-pair raw duration clamped to at least `1`
- missing, empty, or falsey lyrics represented as an empty lyric
- gap annotation from the previous raw end, clamped at `0`
- lexicographically sorted unique candidate start/end pairs with `end > start`
- duration and gap candidate lists from the Python multiplier tables and scalar
  ceil cap calculation

Bayesian candidate filtering, Bayes-specific candidate construction, local cost
functions, metrical penalties, DP decode, note mutation, public dispatch, and
GUI/Web/application promotion stay legacy-owned or separately planned.

## Dependency Expansion

`inference/quant/quantization.py` imports:

- stdlib: `__future__.annotations`, `typing.Any`
- third party: `numpy`

This primitive boundary uses Python scalar arithmetic, list/dict construction,
`round`, `sorted(set(...))`, and `np.ceil` only for scalar cap calculations in
duration/gap candidate lists. Rust should not add NumPy, ndarray, PyO3,
subprocess, CLI, HTTP, or runtime-router dependencies for this unit.

Dependency evidence:

- `pyproject.toml` and `requirements.txt` include `numpy<2.0.0`.
- `uv.lock` records `numpy==1.26.4`.
- `third_party/sources/manifest.json` records
  `third_party/sources/numpy-1.26.4`.
- `third_party/native_sources/manifest.json` records OpenBLAS native coverage
  for NumPy/SciPy, but this unit does not need BLAS or array kernels.
- `third_party/source_audit.json` reports all foreign runtime native binaries
  covered and no remaining `third_party` binary artifacts.

Those sources matter for later smart, phrase DP, and Bayesian quantization
units. They are deliberately kept out of this primitive helper unit.

## Seam

Use the default rewrite seam:

- kind: independent Rust library
- crate: `rewrite-in-rust/rust/crates/v2m-core`
- runtime owner: legacy Python
- bridge dependencies: none

Do not add PyO3, subprocess, CLI, HTTP, runtime-router, NumPy, or ndarray code
for this unit.

## Fixture Harness

Rust tests should consume the durable parity tables at:

```text
rewrite-in-rust/fixtures/quantization_candidate_scalar_primitives.tsv
rewrite-in-rust/fixtures/quantization_candidate_pair_primitives.tsv
```

The legacy Python side of the tables is checked by:

```bash
uv run python rewrite-in-rust/bootstrap/check_quantization_candidate_primitives.py
```

The fixture tables must cover:

- default and positive DP grid resolution
- default and scaled segment shift candidates
- nearest-candidate normal and tie behavior
- modular and grid distances, including negative grid inputs
- half-even candidate value rounding, including positive and negative ties
- negative-radius candidate generation
- duration and gap cap behavior for normal, zero, small, and large raw values
- note-pair lyric fallback and minimum raw duration
- gap annotation for increasing, overlapping, and reversed raw pairs
- candidate-pair sorting and exclusion of `end <= start`

## Repeated-Call Behavior

All selected helpers are deterministic and stateless. For fixed scalar inputs or
pair lists, repeated calls must return the same vectors and structs. The helpers
must not depend on model state, GUI/Web state, filesystem state, export
settings, or global runtime configuration.

Until bridge/promotion work changes the public Rust boundary, these helpers
assume the same positive grid/step/modulo values used by the current quantizer
internals. Invalid-value Python exception mapping belongs to a future bridge or
promotion unit if those private helpers become a public runtime boundary.

## Rollback

Rollback is keeping all production imports unchanged:

```text
inference.quant.quantization helper functions
```

No production Python caller should import Rust output until a later promotion
record chooses and verifies a bridge.
