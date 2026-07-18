# 0115 - Implement ASR Resample Poly Contract

Date: 2026-07-18

## Unit

`asr_resample_poly_contract`

## Implementation

Added `rust/crates/v2m-core/src/asr_resample_poly.rs` and exported it from
`v2m-core`.

The Rust helper `resample_poly_1d_float32(input, target_rate, source_rate)`
implements the fixture-backed default SciPy path:

- positive-rate validation with Python-compatible `ValueError` text
- gcd reduction for `up/down`
- identity copy after validation
- SciPy default FIR design with `half_len = 10 * max(up, down)`, cutoff
  `1 / max(up, down)`, Kaiser beta `5.0`, and Bessel I0 series evaluation
- float32 cast before multiplying coefficients by `up`
- SciPy pre-pad/post-pad and `n_pre_remove` trimming
- 1D constant-zero `upfirdn` using SciPy's transposed/flipped phase layout
- empty input, NaN/Inf propagation, output length, and selected long steady-state
  sample behavior covered by fixture tests

No runtime Python route, PyO3 bridge, subprocess bridge, model runtime, or audio
file IO was added.

## Tolerance

The Rust fixture test uses absolute tolerance `2e-7` for finite float32 samples
and `1e-5` for projected finite sums. A stricter `1e-7` run only missed by one
single-precision ulp on a long steady-state tail sample, consistent with
Cython/NumPy versus Rust float32 math-order differences.

## Verification

```bash
cargo test --manifest-path rust/Cargo.toml asr_resample_poly_contract -- --nocapture
```

## State

`asr_resample_poly_contract` is now `reimplemented`. It still requires
independent `stage_behavior_reviewer` and `data_algorithm_reviewer` gates before
it can be marked `verified`.
