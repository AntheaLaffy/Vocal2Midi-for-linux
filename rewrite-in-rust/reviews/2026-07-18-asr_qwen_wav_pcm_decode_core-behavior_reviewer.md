# asr_qwen_wav_pcm_decode_core - behavior_reviewer

Date: 2026-07-18
Decision: fail

## Findings

- Severity: medium
- Location: rust/crates/v2m-core/src/asr_qwen_wav_pcm_decode.rs:41
- Issue: Sample-width classification is derived with floor division from WAV bit depth, but Python uses byte sample width from `wave.getsampwidth()`.
- Evidence: Legacy Python reads `sample_width = wav_file.getsampwidth()` at `/home/fuurin/code/Vocal2Midi-for-linux/inference/qwen3asr_dml/utils.py:61`, then dispatches on byte widths at lines 64-80. Rust computes `sample_width = spec.bits_per_sample / 8` at `rust/crates/v2m-core/src/asr_qwen_wav_pcm_decode.rs:41`. For non-byte-aligned PCM headers, Python rounds up to the byte container width; an inline Python probe with a 20-bit PCM WAV header reported `getsampwidth() == 3`. Hound also rejects non-multiple-of-8 `bits_per_sample` while parsing at `/home/fuurin/.cargo/registry/src/rsproxy.cn-e3de039b2554c837/hound-3.5.1/src/read.rs:427`, before this compatibility layer can apply Python's width-3 path. Current fixtures cover 8/16/24/32/40-bit values only at `fixtures/asr_qwen_wav_pcm_decode_core.jsonl:1` through `fixtures/asr_qwen_wav_pcm_decode_core.jsonl:12`.
- Required fix: Either classify and decode sample width the way Python `wave` does for byte-container widths, or explicitly document non-byte-aligned PCM as out of scope for this unit and add a fixture proving the intended Rust boundary. Without that boundary fixture, the sample-width handling claim is broader than the tested behavior.

- Severity: low
- Location: rust/crates/v2m-core/src/asr_qwen_wav_pcm_decode.rs:109
- Issue: Non-finite slicing inputs still do not preserve Python observable errors.
- Evidence: Python `load_audio` uses `int((start_second or 0.0) * sample_rate)` at `/home/fuurin/code/Vocal2Midi-for-linux/inference/qwen3asr_dml/utils.py:114`, so `start_second=float("nan")` raises `ValueError: cannot convert float NaN to integer` and infinities raise `OverflowError`. Rust `python_trunc_to_isize` uses `value.trunc() as isize` at `rust/crates/v2m-core/src/asr_qwen_wav_pcm_decode.rs:109`, which saturates/collapses non-finite floats instead of returning the Python error. The 12 fixtures now cover positive slicing, zero-duration truthiness, negative start, and negative-start-with-duration empty output at `fixtures/asr_qwen_wav_pcm_decode_core.jsonl:7` through `fixtures/asr_qwen_wav_pcm_decode_core.jsonl:11`, but they do not cover non-finite accepted `f64` inputs.
- Required fix: Add fixtures for NaN and infinity, or constrain the Rust public API so non-finite values cannot enter this compatibility seam. If accepted, return Python-equivalent error type/message.

- Severity: low
- Location: rust/crates/v2m-core/src/asr_qwen_wav_pcm_decode.rs:49
- Issue: The same-rate sample-rate boundary is intentionally different from full Python `_load_wav_audio`, but it is not fixture-backed.
- Evidence: The manifest says resampling is owned by `asr_resample_poly_contract` at `manifest.yaml:1792`, while Python `_load_wav_audio` resamples mismatched rates at `/home/fuurin/code/Vocal2Midi-for-linux/inference/qwen3asr_dml/utils.py:85`. Rust returns a Rust-specific `ValueError` for sample-rate mismatch at `rust/crates/v2m-core/src/asr_qwen_wav_pcm_decode.rs:49`. None of the 12 fixtures exercises this boundary.
- Required fix: Add a source-rate-mismatch fixture that records the accepted same-rate-only Rust boundary, or keep this unit out of any public `_load_wav_audio` replacement path until `asr_resample_poly_contract` owns resampling parity.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_asr_qwen_wav_pcm_decode_core.py`: passed, 12 cases.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml asr_qwen_wav_pcm_decode_core`: passed, 1 matching Rust fixture test.
- `uv run python -m py_compile inference/qwen3asr_dml/utils.py`: passed.
- Inline Python probe for signed slicing: confirmed Python negative `start_second` uses NumPy negative slicing and NaN raises `ValueError`. The latest Rust/fixture state now covers the negative-start cases.
- Inline Python probe for 20-bit PCM WAV: confirmed Python `wave.getsampwidth()` reports byte width `3`.
- Local hound 3.5.1 source inspection: `read.rs:427` rejects non-multiple-of-8 `bits_per_sample` while Python `wave` can expose such files through byte sample width.

## Residual Risk

The existing fixtures give useful confidence for same-rate unsigned 8-bit, signed 16-bit, signed 24-bit, signed 32-bit, simple multichannel mean, positive slicing, zero-duration truthiness, negative-start slicing, negative-start-with-duration empty output, and unsupported 40-bit sample-width error text. They do not prove malformed WAV error projection, WAVE_FORMAT_EXTENSIBLE handling, non-byte-aligned sample widths, pydub-success behavior, or SciPy resampling parity. Resampling is intentionally outside this unit, but the Rust API still accepts a target sample rate and currently exposes a non-Python error on mismatch.

## Promotion Note

This behavior role blocks promotion as a Python-parity replacement for `_load_wav_audio`. Keep Python `inference.qwen3asr_dml.utils.load_audio` and `_load_wav_audio` as runtime owners until the sample-width boundary is fixed or explicitly narrowed with fixtures, and until the same-rate sample-rate boundary is recorded as intentional promotion scope.
