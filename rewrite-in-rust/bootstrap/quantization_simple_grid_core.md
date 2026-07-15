# quantization_simple_grid_core Bootstrap

## Boundary

`quantization_simple_grid_core` covers only:

- `inference/quant/quantization.py::_ticks_from_sec`
- `inference/quant/quantization.py::_quantize_notes_simple`

The public compatibility surface is:

- no-op behavior for empty note lists or `quantization_step <= 0`
- in-place sorting by note onset when quantization runs
- Python half-even rounding when converting seconds to ticks
- Python half-even rounding when snapping ticks to the grid
- strictly increasing quantized onsets, bumping by one step when needed
- offset snapping from original offsets
- touching-note glue when `abs(orig_offset[i] - orig_onset[i + 1]) < 1e-3`
- minimum duration of at least one quantization step
- clipping an offset to the next quantized onset if it would cross it
- preservation of non-timing note metadata with the sorted note objects
- finite note timings and positive finite tempo as pre-promotion inputs; a
  future bridge must either validate those inputs before calling Rust or map
  invalid numeric inputs to Python-compatible errors

`quantize_notes` dispatch, activation policy, smart duration DP, phrase DP,
Bayesian quantization, GUI/Web/application defaults, pipeline routing, and
runtime promotion stay legacy-owned or separately planned.

## Dependency Expansion

`inference/quant/quantization.py` imports:

- stdlib: `__future__.annotations`, `typing.Any`
- third party: `numpy`

The selected `_quantize_notes_simple` path uses only Python scalar arithmetic,
list sorting, and the local `_ticks_from_sec` helper. It does not call NumPy,
candidate builders, dynamic programming helpers, model inference, GUI, Web, or
export code. Therefore the Rust unit should not add a NumPy, ndarray, PyO3,
subprocess, CLI, HTTP, or runtime-router dependency.

Dependency evidence:

- `pyproject.toml` and `requirements.txt` include `numpy<2.0.0`.
- `uv.lock` records `numpy==1.26.4`.
- `third_party/sources/manifest.json` records
  `third_party/sources/numpy-1.26.4`.
- `third_party/native_sources/manifest.json` records OpenBLAS native coverage
  for NumPy/SciPy, but this unit does not need BLAS or array kernels.
- `third_party/source_audit.json` reports all foreign runtime native binaries
  covered and zero `third_party` binary artifacts.

Those sources matter for later smart/DP/Bayesian quantization units. They are
deliberately kept out of this scalar simple-grid unit.

## Seam

Use the default rewrite seam:

- kind: independent Rust library
- crate: `rewrite-in-rust/rust/crates/v2m-core`
- runtime owner: legacy Python
- bridge dependencies: none

Do not add PyO3, subprocess, CLI, HTTP, runtime-router, NumPy, or ndarray code
for this unit.

## Fixture Harness

Rust tests should consume the durable parity table at:

```text
rewrite-in-rust/fixtures/quantization_simple_grid_core.tsv
```

The fixtures must cover:

- empty input
- `quantization_step <= 0` no-op behavior and original order preservation
- half-even seconds-to-ticks conversion and grid rounding
- a non-default positive tempo and quantization step
- sort-by-onset and monotonic onset bumping
- touching-note offset glue
- minimum duration when the offset snaps to or before onset
- offset clipping to the next onset
- pitch and lyric metadata preservation

The legacy Python side of the table is checked by:

```bash
uv run python rewrite-in-rust/bootstrap/check_quantization_simple_grid_core.py
```

## Repeated-Call Behavior

The function mutates the provided note list in place. For a fixed note sequence,
tempo, and quantization step, repeated calls starting from the same original
input must produce the same sorted and quantized sequence. The function must not
depend on model state, GUI/Web state, filesystem state, or export settings.

Until bridge/promotion work changes the public Rust boundary, this unit assumes
the same finite timing values and positive finite tempo used by the pipeline's
normal quantization path. Promotion work must add explicit validation or error
mapping before Rust owns calls that may receive `NaN`, infinities, or
non-positive tempo.

## Rollback

Rollback is keeping all production imports unchanged:

```text
inference.quant.quantization._quantize_notes_simple
```

No production Python caller should import Rust output until a later promotion
record chooses and verifies a bridge.
