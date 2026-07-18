# 0111 - Fix ASR Qwen WAV PCM Decode Review Findings

Date: 2026-07-18

## Unit

`asr_qwen_wav_pcm_decode_core`

## Trigger

The initial behavior review failed promotion for three boundary gaps:

- sample width was derived from `bits_per_sample / 8`, while Python
  `wave.getsampwidth()` follows container byte width
- non-finite `start_second`/`duration` did not preserve Python `int(...)`
  `ValueError`/`OverflowError` messages
- source-rate mismatch was intentionally out of scope but lacked a fixture
  recording the same-rate-only Rust boundary

## Change

Replaced the first `hound`-based implementation with a narrow hand-written
RIFF/WAVE PCM parser for this compatibility unit.

The parser reads:

- `RIFF`/`WAVE`
- `fmt ` format tag, channel count, sample rate, and block alignment
- `data` bytes

It derives Python-compatible `sample_width` from
`ceil(bits_per_sample / 8)`, matching Python 3.12 `wave.getsampwidth()` even
when `wBlockAlign` disagrees with bit depth. It then keeps the existing explicit
normalization, 24-bit sign extension, channel mean, and same-rate boundary
logic.

The slicing adapter now returns Python-compatible non-finite conversion errors:

- `ValueError: cannot convert float NaN to integer`
- `OverflowError: cannot convert float infinity to integer`

## Fixture Evidence

`fixtures/asr_qwen_wav_pcm_decode_core.jsonl` now contains 22 cases. New cases
added after the initial review cover:

- 7-bit header with 1-byte container width
- 12-bit header with 2-byte container width
- 20-bit header with 3-byte container width
- 12-bit and 20-bit headers where `wBlockAlign` disagrees with the bit-depth
  sample width
- `start_second` NaN and infinity
- `duration` NaN and infinity
- source-rate mismatch as a Rust-only same-rate boundary fixture

## Dependency Decision

The writer followed the suggested `rodio` dependency trail and confirmed that
`rodio 0.22.2` uses Symphonia for WAV/PCM decoding. The first implementation
used `hound` as the lower-level WAV parser, but behavior review showed `hound`
rejects non-byte-aligned `bits_per_sample` headers before this compatibility
layer can apply Python byte-container width behavior. This unit therefore does
not keep `hound` as a dependency.

## Verification

Checks run after the fix:

```bash
uv run python rewrite-in-rust/bootstrap/check_asr_qwen_wav_pcm_decode_core.py
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml asr_qwen_wav_pcm_decode_core
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml
cargo clippy --manifest-path rewrite-in-rust/rust/Cargo.toml --all-targets --all-features -- -D warnings
```

## Rollback

Keep `inference.qwen3asr_dml.utils.load_audio` and `_load_wav_audio` as runtime
owners.
