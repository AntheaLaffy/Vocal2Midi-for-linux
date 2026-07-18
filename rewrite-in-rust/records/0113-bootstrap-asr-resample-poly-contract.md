# 0113 - Bootstrap ASR Resample Poly Contract

Date: 2026-07-18

## Unit

`asr_resample_poly_contract`

## Decision

Confirm `asr_resample_poly_contract` as a narrow default-path SciPy
`resample_poly` compatibility unit.

The unit remains `planned` because no Rust implementation has been added yet.
Its inventory is now `confirmed`.

## Boundary

The selected seam starts from pre-decoded 1D `float32` audio and preserves the
default project call:

```python
scipy.signal.resample_poly(audio, sample_rate, sr)
```

Only default options are in scope:

- `axis=0`
- `window=('kaiser', 5.0)`
- `padtype='constant'`
- `cval=None`

This unit does not own WAV decoding, pydub/soundfile/libsndfile IO, arbitrary
codec behavior, model execution, ONNX Runtime sessions, or runtime routing.

## Dependency Evidence

`uv.lock` pins SciPy 1.17.1 and NumPy 1.26.4. The local source corpus contains:

- `third_party/sources/scipy-1.17.1`
- `third_party/sources/numpy-1.26.4`

Project call path evidence:

- `qwen3asr_dml.utils._load_wav_audio` calls `scipy.signal.resample_poly` after
  WAV PCM decode and channel mean.
- `romaji_asr.common.load_audio` calls `scipy.signal.resample_poly` after
  `soundfile.read`, optional channel mean, and `np.asarray(..., dtype=np.float32)`.

SciPy source path evidence:

- `scipy/signal/_signaltools.py::resample_poly`
- `scipy/signal/_fir_filter_design.py::firwin`
- `scipy/signal/windows/_windows.py::get_window`
- `scipy/signal/windows/_windows.py::kaiser`
- `scipy/signal/_upfirdn.py::upfirdn`
- `scipy/signal/_upfirdn_apply.pyx::_output_len`

No deeper source expansion is needed beyond the SciPy first-layer source tree
except the in-tree `firwin -> get_window -> kaiser -> scipy.special.i0` call
path for default Kaiser weights. OpenBLAS/native linear algebra and audio codec
native libraries do not reach this fixture-bound 1D FIR/upfirdn seam.

## Crate Decision

Reject `rubato` and `samplerate` for exact parity. They are useful audio
resampling crates, but they do not own SciPy's default FIR design, pre-padding,
trim, output length, dtype preservation, or error text.

The writer should hand-write the narrow default 1D float32 algorithm against
fixtures.

## Fixtures

Added:

- `fixtures/asr_resample_poly_contract.jsonl`
- `bootstrap/check_asr_resample_poly_contract.py`

The fixture set has 18 cases covering common ASR rates, long steady-state
dual-sine inputs, GCD reduction, upsampling, identity, empty input,
single-sample input, NaN/Inf propagation, and invalid rates including equal
invalid rates.

## Verification

Bootstrap checks:

```bash
uv run python rewrite-in-rust/bootstrap/check_asr_resample_poly_contract.py
uv run python -m py_compile inference/qwen3asr_dml/utils.py inference/romaji_asr/common.py
uv run python scripts/audit_vendored_sources.py
```

## Rollback

Keep `scipy.signal.resample_poly` calls in Python-owned
`qwen3asr_dml.utils` and `romaji_asr.common`.
