# ASR Resample Poly Contract Bootstrap

Date: 2026-07-18

## Unit

`asr_resample_poly_contract`

## Public Boundary

This unit owns the shared resampling helper behavior reached by:

- `inference/qwen3asr_dml/utils.py::_load_wav_audio`
- `inference/romaji_asr/common.py::load_audio`

Both call sites use `scipy.signal.resample_poly(audio, sample_rate, sr)` only
after audio has already become a mono 1D `np.float32` array. The selected Rust
seam is therefore a library helper equivalent to:

```python
resample_poly(audio.astype(np.float32), target_rate, source_rate).astype(np.float32, copy=False)
```

for the default SciPy options:

- `axis=0`
- `window=('kaiser', 5.0)`
- `padtype='constant'`
- `cval=None`

Identity `target_rate == source_rate` may be a Rust short-circuit, but it must
come after SciPy-compatible positive-rate validation and return the same
float32 values and shape as the Python caller path.

## Source Evidence

Project call sites:

- `inference/qwen3asr_dml/utils.py`
- `inference/romaji_asr/common.py`

First-layer dependency source:

- `third_party/sources/scipy-1.17.1/scipy/signal/_signaltools.py::resample_poly`
- `third_party/sources/scipy-1.17.1/scipy/signal/_fir_filter_design.py::firwin`
- `third_party/sources/scipy-1.17.1/scipy/signal/windows/_windows.py::get_window`
- `third_party/sources/scipy-1.17.1/scipy/signal/windows/_windows.py::kaiser`
- `third_party/sources/scipy-1.17.1/scipy/signal/_upfirdn.py::upfirdn`
- `third_party/sources/scipy-1.17.1/scipy/signal/_upfirdn_apply.pyx::_output_len`

`uv.lock` pins SciPy 1.17.1 and NumPy 1.26.4. The source manifest records both
source snapshots under `third_party/sources/`.

## Dependency Decision

Generic audio resampler crates are not selected for exact parity:

- `rubato 4.0.0` is a maintained audio resampling crate, but its public contract
  is quality-oriented audio conversion, not SciPy's default `firwin + upfirdn`
  filter construction, pre-padding, trimming, output length, and error text.
- `samplerate 0.2.4` wraps libsamplerate and would introduce a native/FFI
  algorithm with different public outputs.

The writer should hand-write the narrow default path:

1. validate integer positive `target_rate`/`source_rate`
2. reduce `up/down` by `gcd`
3. build the default low-pass FIR coefficients with
   `half_len = 10 * max(up, down)`, cutoff `1 / max(up, down)`, and
   Kaiser beta `5.0`; the window path is
   `_fir_filter_design.py::firwin -> windows/_windows.py::get_window -> kaiser`,
   where Kaiser weights use `scipy.special.i0`
4. multiply coefficients by `up`
5. prepend SciPy's `n_pre_pad = down - half_len % down`
6. run 1D constant-zero `upfirdn`
7. keep `n_pre_remove..n_pre_remove+n_out`
8. return `Vec<f32>` values matching the fixture tolerance

The implementation should not add a production route, PyO3 bridge, subprocess
bridge, or model runtime dependency.

## Fixture Harness

`fixtures/asr_resample_poly_contract.jsonl` contains 18 Python golden cases:

- identity
- 44100 -> 16000
- 48000 -> 16000
- 22050 -> 16000
- long steady-state 44100 -> 16000, 48000 -> 16000, 22050 -> 16000, and
  8000 -> 16000 dual-sine inputs with sampled head/mid/tail values plus finite
  sum checks
- 32000 -> 16000 GCD reduction
- 8000 -> 16000 upsampling
- single-sample resampling
- empty input
- NaN/Inf propagation
- zero and negative rate errors, including equal invalid rates that prove the
  checker does not bypass SciPy validation

`bootstrap/check_asr_resample_poly_contract.py` validates these fixtures against
the current uv Python environment and always calls SciPy `resample_poly` for the
contract cases.

Accepted numeric tolerance for Rust fixture comparison: absolute error `<= 2e-7`
for finite float32 samples and `<= 1e-5` for projected finite sums, with string
sentinels for `nan`, `inf`, and `-inf`. The looser finite-sample Rust tolerance
covers 1-ulp-level differences from Cython/NumPy float32 math order while keeping
the SciPy golden fixture checker exact to the Python environment.

## Kept Legacy

The following remain Python-owned:

- non-default `resample_poly` options (`axis`, custom `window`, `padtype`,
  `cval`)
- multidimensional arrays and complex dtypes
- soundfile/pydub/libsndfile file IO and codecs
- model sessions, Qwen/Romaji inference, and ONNX Runtime

## Writer Readiness

This unit is writer-ready after dependency/bootstrap review. The expected Rust
module should live in `v2m-core`, expose an independent library helper, and stay
unwired from production Python callers.
