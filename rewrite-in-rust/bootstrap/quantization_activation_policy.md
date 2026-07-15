# quantization_activation_policy Bootstrap

## Boundary

`quantization_activation_policy` covers only
`inference/quant/quantization.py::should_apply_quantization`.

The public compatibility surface is:

- `mode = (mode or "simple").lower()`
- exact normalized `dp` returns `True` regardless of `quantization_step`
- all other modes return `quantization_step > 0`
- `None` and empty string mode values behave like `simple`
- case-insensitive mode matching through lowercase conversion
- no whitespace trimming before matching `dp`

`quantize_notes`, simple/smart/dp/bayes note mutation, GUI/Web controls,
application defaults, pipeline routing, and export behavior stay legacy-owned.

## Dependency Expansion

`inference/quant/quantization.py` imports:

- stdlib: `__future__.annotations`, `typing.Any`
- third party: `numpy`

The selected `should_apply_quantization` function uses only scalar string and
integer logic. It does not call NumPy, dynamic programming helpers, note
mutation, model inference, GUI, Web, or export code. Therefore the Rust unit
should not add a NumPy, ndarray, PyO3, subprocess, CLI, HTTP, or runtime-router
dependency.

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
deliberately kept out of this activation-policy unit.

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
rewrite-in-rust/fixtures/quantization_activation_policy.tsv
```

The fixtures must cover:

- `None`, empty string, and simple fallback behavior
- case-insensitive `dp`
- `dp` with negative and zero steps returning true
- non-`dp` modes with negative, zero, and positive steps
- whitespace-padded `dp` not matching because Python does not trim

The legacy Python side of the table is checked by:

```bash
uv run python rewrite-in-rust/bootstrap/check_quantization_activation_policy.py
```

## Repeated-Call Behavior

The predicate is stateless. Repeated calls with the same mode and step must
return the same value and must not depend on note content, model state, GUI/Web
state, or filesystem state.

## Rollback

Rollback is keeping all production imports unchanged:

```text
inference.quant.quantization.should_apply_quantization
```

No production Python caller should import Rust output until a later promotion
record chooses and verifies a bridge.
