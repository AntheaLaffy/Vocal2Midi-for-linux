# 0112 - Close ASR Qwen WAV PCM Decode Gate

Date: 2026-07-18

## Unit

`asr_qwen_wav_pcm_decode_core`

## Decision

Mark `asr_qwen_wav_pcm_decode_core` as `verified`.

Python remains the runtime owner. This gate verifies the independent Rust
library seam and fixture parity only.

## Review Evidence

Required reviews passed on rerun2:

- `reviews/2026-07-18-asr_qwen_wav_pcm_decode_core-behavior_reviewer-rerun2.md`
- `reviews/2026-07-18-asr_qwen_wav_pcm_decode_core-data_algorithm_reviewer-rerun2.md`
- `reviews/2026-07-18-asr_qwen_wav_pcm_decode_core-error_tracing_reviewer-rerun2.md`

Earlier reviews found sample-width, non-finite slicing, and same-rate boundary
gaps. Record 0111 fixed those with a narrow RIFF/WAVE parser, 22 fixture cases,
and explicit Python-compatible error projection for the reviewed seam.

## Fixture Evidence

`fixtures/asr_qwen_wav_pcm_decode_core.jsonl` contains 22 cases covering:

- unsigned 8-bit, signed 16-bit, signed 24-bit, and signed 32-bit PCM
- 24-bit sign extension
- mono, stereo, and three-channel mean
- Python `wave.getsampwidth()` bit-depth sample-width behavior for 7/12/20-bit
  headers, including block-align disagreement
- positive, zero-duration, negative-start, and non-finite slicing after pydub
  fallback is forced
- unsupported sample width 5 `ValueError`
- Rust same-rate boundary for source-rate mismatch

## Verification

Final commands:

```bash
uv run python rewrite-in-rust/bootstrap/check_asr_qwen_wav_pcm_decode_core.py
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml asr_qwen_wav_pcm_decode_core
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml
cargo clippy --manifest-path rewrite-in-rust/rust/Cargo.toml --all-targets --all-features -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --manifest-path rewrite-in-rust/rust/Cargo.toml --no-deps
uv run python -m py_compile inference/qwen3asr_dml/utils.py
uv run python scripts/audit_vendored_sources.py
```

Additional checks:

```bash
cargo tree --manifest-path rewrite-in-rust/rust/Cargo.toml -p v2m-core --edges normal | rg -n "hound|rodio|symphonia" || true
git -C rewrite-in-rust diff --check
```

## Rollback

Keep `inference.qwen3asr_dml.utils.load_audio` and `_load_wav_audio` as runtime
owners.
