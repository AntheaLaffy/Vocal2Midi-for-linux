# asr_qwen_wav_pcm_decode_core - data_algorithm_reviewer

Date: 2026-07-18
Decision: pass

## Findings

No findings.

## Scope And Boundary

- Unit: `asr_qwen_wav_pcm_decode_core`
- Role: `data_algorithm_reviewer`
- Writer/reviewer separation: preserved for this pass; this review did not modify production code.
- Boundary: confirmed for same-rate WAV PCM fallback decode and forced-fallback slicing. Resampling remains outside this unit and is represented as an explicit Rust-only boundary fixture.

## Evidence

- Legacy numeric source: `/home/fuurin/code/Vocal2Midi-for-linux/inference/qwen3asr_dml/utils.py:57` reads Python `wave` metadata, decodes 8/16/24/32-bit PCM with NumPy, averages channels, resamples only when source and target rates differ, and returns `float32`.
- Legacy slicing source: `/home/fuurin/code/Vocal2Midi-for-linux/inference/qwen3asr_dml/utils.py:114` uses Python `int(...)` truncation and NumPy slicing after fallback decode.
- Rust decode path: `rewrite-in-rust/rust/crates/v2m-core/src/asr_qwen_wav_pcm_decode.rs:59` matches the 8-bit unsigned centering, 16-bit signed normalization by 32768, 24-bit little-endian sign extension and normalization by 8388608, and 32-bit signed normalization by 2147483648.
- Rust channel path: `rewrite-in-rust/rust/crates/v2m-core/src/asr_qwen_wav_pcm_decode.rs:143` averages multichannel frames with `chunks_exact(channels)`, matching the fixture-backed mono/stereo/three-channel behavior.
- Rust sample-width path: `rewrite-in-rust/rust/crates/v2m-core/src/asr_qwen_wav_pcm_decode.rs:199` derives sample width with `bits_per_sample.div_ceil(8)`, matching Python 3.12 `wave.getsampwidth()` for the 7/12/20-bit and block-align-disagreement fixtures.
- Rust slicing path: `rewrite-in-rust/rust/crates/v2m-core/src/asr_qwen_wav_pcm_decode.rs:106` truncates start/duration in Python order, preserves zero-duration falsiness, normalizes negative slice indexes, and returns an empty slice when normalized start is greater than normalized end.
- Rust non-finite path: `rewrite-in-rust/rust/crates/v2m-core/src/asr_qwen_wav_pcm_decode.rs:123` preserves Python `int(...)` `ValueError`/`OverflowError` messages for NaN and infinity.
- Rust same-rate boundary: `rewrite-in-rust/rust/crates/v2m-core/src/asr_qwen_wav_pcm_decode.rs:52` rejects source-rate mismatch with the explicit `asr_resample_poly_contract` ownership message.
- Fixture coverage: `rewrite-in-rust/fixtures/asr_qwen_wav_pcm_decode_core.jsonl:1` through `rewrite-in-rust/fixtures/asr_qwen_wav_pcm_decode_core.jsonl:22` contain 22 cases covering the requested numeric/data areas, including block-align disagreement at lines 15-16, non-finite slicing at lines 17-20, source-rate mismatch at line 21, and unsupported width 5 at line 22.
- Test harness: `rewrite-in-rust/rust/crates/v2m-core/src/asr_qwen_wav_pcm_decode.rs:245` includes the fixture file directly and compares Rust output length, sample values within `1e-7`, error type, and error message.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_asr_qwen_wav_pcm_decode_core.py`: pass, `asr_qwen_wav_pcm_decode_core fixtures ok: 22 cases`
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml asr_qwen_wav_pcm_decode_core`: pass, 1 targeted test passed
- Fixture metadata inspection for non-byte-aligned and block-align-disagreement cases: pass. Python `wave.getsampwidth()` reported `ceil(bits_per_sample/8)` widths for 7-bit, 12-bit, 20-bit, 12-bit/block-align-3, 20-bit/block-align-4, and unsupported 40-bit cases.

## Residual Risk

This pass did not review malformed WAV diagnostics beyond the fixture-covered cases, pydub's primary decode route, or actual resampling parity. Those remain outside the stated same-rate PCM fallback boundary.

## Promotion Note

This data/algorithm role does not block promotion for `asr_qwen_wav_pcm_decode_core`.
