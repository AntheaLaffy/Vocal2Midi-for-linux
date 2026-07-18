# 0116 - Close ASR Resample Poly Gate

Date: 2026-07-18

## Unit

`asr_resample_poly_contract`

## Decision

Mark `asr_resample_poly_contract` as `verified`.

The Rust implementation is fixture-backed and independently reviewed, but
runtime ownership remains legacy. Python callers still use
`scipy.signal.resample_poly`; no bridge or promotion route was added.

## Review Evidence

- `reviews/2026-07-18-asr_resample_poly_contract-dependency_bootstrap_reviewer-rerun.md`
- `reviews/2026-07-18-asr_resample_poly_contract-behavior_reviewer.md`
- `reviews/2026-07-18-asr_resample_poly_contract-data_algorithm_reviewer.md`

All three required gates passed without blocking findings. The project uses
`behavior_reviewer` reports to satisfy the manifest's stage-behavior gate, as in
the prior ASR unit records.

## Verification

```bash
cargo test --manifest-path rust/Cargo.toml asr_resample_poly_contract
uv run python rewrite-in-rust/bootstrap/check_asr_resample_poly_contract.py
uv run python -m py_compile inference/qwen3asr_dml/utils.py inference/romaji_asr/common.py
uv run python scripts/audit_vendored_sources.py
```

## Remaining Boundary

Non-default SciPy options, multidimensional arrays, complex dtypes, file IO,
Qwen/Romaji model execution, ONNX Runtime sessions, and runtime promotion remain
outside this unit.
