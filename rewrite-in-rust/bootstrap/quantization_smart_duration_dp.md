# quantization_smart_duration_dp Bootstrap

## Boundary

`quantization_smart_duration_dp` covers only
`inference/quant/quantization.py::_quantize_notes_smart`.

The compatibility surface is:

- no-op behavior for empty note lists or `quantization_step <= 0`
- in-place sorting by note onset when quantization runs
- Python half-even rounding through `_ticks_from_sec`
- raw duration calculation as `max(1, offset_tick - onset_tick)`
- duration candidates from `_build_duration_candidates`
- preferred duration multipliers `{1, 2, 4, 8, 16}` with `0.08 * step`
  penalty for other candidates
- transition cost `abs(previous_candidate - current_candidate) * 0.08`
- NumPy `argmin` first-min tie behavior for transition and final decode
- first quantized onset snapped to the requested grid with Python half-even
  rounding
- offset placement as `onset + max(step, chosen_duration)`
- rest placement as `0` when raw rest is below half a step, otherwise rounded
  to the requested step with Python half-even rounding
- overlap rest clamping through `max(0, next_raw_onset - current_raw_offset)`
- onset/offset mutation only; pitch and lyric metadata are preserved with the
  sorted note objects

`quantize_notes` public dispatch, simple mode, phrase DP/asymmetric mode,
Bayesian mode, GUI/Web/application defaults, pipeline routing, and runtime
promotion stay legacy-owned or separately planned.

## Dependency Expansion

`inference/quant/quantization.py` imports:

- stdlib: `__future__.annotations`, `typing.Any`
- third party: `numpy`

The selected smart path uses NumPy only for small local DP arrays:

- `np.inf`
- `np.full`
- `np.array(..., dtype=np.float64)`
- vectorized absolute values for transition cost
- `np.argmin` first-min tie behavior

The same behavior can be covered by Rust `Vec<f64>` and `Vec<isize>` tables. Do
not add NumPy, ndarray, PyO3, subprocess, CLI, HTTP, or runtime-router
dependencies for this unit.

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

Rust tests should consume the durable parity table at:

```text
rewrite-in-rust/fixtures/quantization_smart_duration_dp.tsv
```

The table uses the same note-sequence shape as simple grid quantization:

```text
case_id<TAB>tempo<TAB>step<TAB>input_notes<TAB>expected_notes
```

Each note is encoded as:

```text
onset,offset,pitch,lyric
```

Notes are joined with `|`; `__empty__` represents an empty lyric.

The legacy Python side of the table is checked by:

```bash
uv run python rewrite-in-rust/bootstrap/check_quantization_smart_duration_dp.py
```

The fixture table must cover:

- empty input
- `quantization_step <= 0` no-op behavior and original order preservation
- onset sorting and metadata preservation
- half-even seconds-to-ticks conversion
- first-onset grid half-even snapping
- zero-duration raw notes clamped to at least one step
- preferred and non-preferred duration penalties
- transition and final `argmin` first-min tie behavior
- a three-note traceback path with two nontrivial backpointers
- rest values below half a step, just above half a step, and at 1.5 steps
- overlapping raw notes with rest clamped to zero
- non-default positive tempo and quantization step

## Repeated-Call Behavior

The function mutates the provided note list in place. For a fixed note sequence,
tempo, and quantization step, repeated calls starting from the same original
input must produce the same sorted and quantized sequence. The function must not
depend on model state, GUI/Web state, filesystem state, export settings, or
global runtime configuration.

Until bridge/promotion work changes the public Rust boundary, this unit assumes
finite note timings and positive finite tempo when quantization runs. Promotion
work must add explicit validation or error mapping before Rust owns calls that
may receive `NaN`, infinities, or non-positive tempo.

## Rollback

Rollback is keeping all production imports unchanged:

```text
inference.quant.quantization._quantize_notes_smart
```
