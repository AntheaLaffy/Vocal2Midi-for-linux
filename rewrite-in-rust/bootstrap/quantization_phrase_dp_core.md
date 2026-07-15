# quantization_phrase_dp_core Bootstrap

## Boundary

`quantization_phrase_dp_core` covers only phrase DP/asymmetric behavior from
`inference/quant/quantization.py`:

- `_local_cost_asym`
- `_segment_split_indices`
- `_center_adjustment`
- `_decode_segment_with_center`
- `_quantize_notes_phrase_hybrid`
- `_quantize_notes_dp_asym`

The compatibility surface is:

- no-op behavior for empty note lists
- in-place sorting by note onset when quantization runs
- fallback to the internal `30` tick grid when the requested
  `quantization_step` is non-positive
- center candidates `(0, 60, 120)` for grid steps up to `30`, and
  `{0, grid_step // 4, grid_step // 2}` for larger grids
- raw note-pair construction with `raw_end = max(onset + 1, offset)` and falsey
  lyrics normalized to `""`
- candidate pair construction from existing start/end candidate helpers with
  `end > start`
- asymmetric local cost terms for start, end, gap, duration, grid120, and
  grid480 distances
- segment splitting by large raw gaps, positive non-tie gaps, and hard maximum
  segment length
- center adjustment penalties and bonuses for `"-"` lyrics, zero raw gaps, long
  raw durations, and centers greater than or equal to `120`
- per-segment decode with Python first-min behavior over insertion-ordered
  candidate pairs
- segment-level center-option DP with switch penalty and second-order shift
  consistency penalty
- overlap repair after all segments decode
- onset/offset mutation only; pitch and lyric metadata stay with the sorted note
  objects

`quantize_notes` public dispatch, simple mode, smart duration DP, Bayesian
quantization, GUI/Web/application defaults, pipeline routing, and runtime
promotion stay legacy-owned or separately planned.

## Dependency Expansion

`inference/quant/quantization.py` imports:

- stdlib: `__future__.annotations`, `typing.Any`
- third party: `numpy`

The selected phrase-DP path itself uses Python scalar arithmetic, lists, dicts,
tuples, `sorted(set(...))`, `round`, `min(..., key=...)`, and helper functions
already covered by earlier quantization units. It does not call NumPy APIs.
Therefore Rust should not add NumPy, ndarray, PyO3, subprocess, CLI, HTTP, or
runtime-router dependencies for this unit.

Dependency evidence:

- `pyproject.toml` and `requirements.txt` include `numpy<2.0.0`.
- `uv.lock` records `numpy==1.26.4`.
- `third_party/sources/manifest.json` records
  `third_party/sources/numpy-1.26.4`.
- `third_party/native_sources/manifest.json` records OpenBLAS native coverage
  for NumPy/SciPy, but this unit does not need BLAS or array kernels.
- `third_party/source_audit.json` reports all foreign runtime native binaries
  covered and no remaining `third_party` binary artifacts.

## Seam

Use the default rewrite seam:

- kind: independent Rust library
- crate: `rewrite-in-rust/rust/crates/v2m-core`
- runtime owner: legacy Python
- bridge dependencies: none

No production Python caller should import Rust output until a later promotion
record chooses and verifies a bridge.

## Fixture Harness

Before writer work, add durable parity tables and a Python checker:

```text
rewrite-in-rust/fixtures/quantization_phrase_dp_helpers.tsv
rewrite-in-rust/fixtures/quantization_phrase_dp_core.tsv
rewrite-in-rust/bootstrap/check_quantization_phrase_dp_core.py
```

The helper table should cover:

- `_local_cost_asym` start early/late, end early/late, predicted/raw gap
  differences, duration error, grid120, and grid480 terms
- `_segment_split_indices` empty input, large raw gap split, positive non-tie
  gap split, `"-"` lyric gap non-split, hard max segment split, and internal
  grid fallback
- `_center_adjustment` positive and zero centers, `"-"` lyric bonus, zero-gap
  bonus, long-duration bonus, and center `>= 120` bonus scaling
- `_decode_segment_with_center` candidate pair ordering, first-min tie behavior,
  previous-end gap cost, and center-specific option choice

The sequence table should use the same note encoding as simple, smart, and
phrase-facing quantization tests:

```text
case_id<TAB>tempo<TAB>step<TAB>input_notes<TAB>expected_notes
```

Each note is encoded as:

```text
onset,offset,pitch,lyric
```

Notes are joined with `|`; `__empty__` represents an empty lyric.

The golden sequence table must cover:

- empty input
- `quantization_step <= 0` using the internal `30` tick grid
- non-default positive tempo and large step using scaled center candidates
- onset sorting and metadata preservation
- segment splitting by large raw gap and by positive non-tie gap
- non-splitting across `"-"` lyric ties
- center switching and second-order consistency penalty across at least three
  segments
- overlap repair where a decoded start is clamped to the previous fixed end
- minimum end extension where `end_tick <= start_tick`
- center adjustment bonuses for tie, zero gap, and long duration

## Repeated-Call Behavior

The function mutates the provided note list in place. For a fixed note sequence,
tempo, and quantization step, repeated calls starting from the same original
input must produce the same sorted and quantized sequence. The function must not
depend on model state, GUI/Web state, filesystem state, export settings, or
global runtime configuration.

Until bridge/promotion work changes the public Rust boundary, this unit assumes
finite note timings. Promotion work must add explicit validation or error
mapping before Rust owns calls that may receive `NaN`, infinities,
overflow-sized ticks, or non-positive tempo.

## Rollback

Rollback is keeping all production imports unchanged:

```text
inference.quant.quantization._quantize_notes_dp_asym
inference.quant.quantization._quantize_notes_phrase_hybrid
```
