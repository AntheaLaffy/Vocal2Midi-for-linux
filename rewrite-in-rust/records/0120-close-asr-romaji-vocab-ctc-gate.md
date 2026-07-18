# 0120 - Close ASR Romaji Vocab CTC Gate

Date: 2026-07-18

## Unit

`asr_romaji_vocab_ctc_decode_core`

## Decision

Mark `asr_romaji_vocab_ctc_decode_core` as `verified`.

The Rust implementation is fixture-backed and independently reviewed, but
runtime ownership remains legacy. Python callers still use
`inference.romaji_asr.common`; no bridge or promotion route was added.

## Review Evidence

- `reviews/2026-07-18-asr_romaji_vocab_ctc_decode_core-behavior_reviewer-rerun.md`
- `reviews/2026-07-18-asr_romaji_vocab_ctc_decode_core-data_algorithm_reviewer-rerun.md`

The first data/algorithm report failed on unsigned large-id wrapping and NaN
argmax behavior. Record 0119 fixed both issues and added 4 Python-generated
fixtures. The behavior and data/algorithm reruns passed without blocking
findings.

## Verification

```bash
uv run python rewrite-in-rust/bootstrap/check_asr_romaji_vocab_ctc_decode_core.py
cargo test --manifest-path rust/Cargo.toml asr_romaji_vocab_ctc_decode_core
uv run python -m py_compile inference/romaji_asr/common.py
uv run python scripts/audit_vendored_sources.py
```

## Remaining Boundary

ONNX Runtime sessions, provider selection, audio loading, resampling,
fake-session metadata, batch padding/mask preparation, model execution, and
runtime promotion remain outside this unit.
