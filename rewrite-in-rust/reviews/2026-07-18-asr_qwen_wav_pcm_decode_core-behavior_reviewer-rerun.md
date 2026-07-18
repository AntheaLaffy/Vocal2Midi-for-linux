# asr_qwen_wav_pcm_decode_core - stage_behavior_reviewer rerun

Date: 2026-07-18
Decision: fail

## Findings

- Severity: medium
- Location: rust/crates/v2m-core/src/asr_qwen_wav_pcm_decode.rs:199
- Issue: The previous non-byte-aligned sample-width finding is only fixed for the current canonical fixtures. Python `wave.getsampwidth()` derives container byte width from the bits-per-sample field rounded up to bytes, but the Rust parser derives `sample_width` from `block_align / channels` at `rust/crates/v2m-core/src/asr_qwen_wav_pcm_decode.rs:200`. For malformed-but-accepted PCM headers where those fields disagree, Rust decodes a different public output than legacy Python.
- Evidence: Legacy `_load_wav_audio` reads `sample_width = wav_file.getsampwidth()` at `/home/fuurin/code/Vocal2Midi-for-linux/inference/qwen3asr_dml/utils.py:61` and dispatches byte-width decoding at `/home/fuurin/code/Vocal2Midi-for-linux/inference/qwen3asr_dml/utils.py:64`. Inspecting the active uv Python 3.12 `wave.py` showed `wBlockAlign` is read at `/home/fuurin/.local/share/uv/python/cpython-3.12.13-linux-x86_64-gnu/lib/python3.12/wave.py:379`, bits-per-sample is read at `wave.py:385`, and `_sampwidth = (sampwidth + 7) // 8` at `wave.py:404`. An independent same-rate WAV probe with `bits_per_sample=12`, `channels=1`, `block_align=1`, and data bytes `00 01 02 03` produced Python `sample_width=2`, shape `[2]`, values `[0.0078125, 0.02349853515625]`; the current Rust crate returned `[-1.0, -0.9921875, -0.984375, -0.9765625]` by treating the same bytes as unsigned 8-bit samples. The fixture file covers consistent 7/12/20-bit headers at `fixtures/asr_qwen_wav_pcm_decode_core.jsonl:12`, `fixtures/asr_qwen_wav_pcm_decode_core.jsonl:13`, and `fixtures/asr_qwen_wav_pcm_decode_core.jsonl:14`, but not this accepted disagreement boundary.
- Required fix: Derive the Python-compatible sample width from the bits-per-sample field using `(bits_per_sample + 7) / 8`, keep or validate `block_align` only when matching Python-observable behavior, and add a fixture where `bits_per_sample` and `block_align / channels` disagree.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_asr_qwen_wav_pcm_decode_core.py` from `/home/fuurin/code/Vocal2Midi-for-linux`: passed, `asr_qwen_wav_pcm_decode_core fixtures ok: 20 cases`.
- `cargo test --manifest-path rust/Cargo.toml asr_qwen_wav_pcm_decode_core` from `/home/fuurin/code/Vocal2Midi-for-linux/rewrite-in-rust`: passed, 1 targeted Rust fixture test passed.
- `cargo test --manifest-path rust/Cargo.toml` from `/home/fuurin/code/Vocal2Midi-for-linux/rewrite-in-rust`: passed, 128 total tests across lib/bin/doc targets.
- `uv run python -m py_compile inference/qwen3asr_dml/utils.py` from `/home/fuurin/code/Vocal2Midi-for-linux`: passed.
- `rg -n "hound|rodio|symphonia" rust/Cargo.lock rust/crates/v2m-core/Cargo.toml`: no matches, confirming the latest filesystem state is no longer the earlier hound-based implementation.
- `rg -n "asr_qwen_wav_pcm_decode|load_wav_audio_fallback_bytes|load_audio_forced_fallback_bytes" /home/fuurin/code/Vocal2Midi-for-linux --glob '!rewrite-in-rust/rust/target/**' --glob '!rewrite-in-rust/reviews/**'`: matches were limited to rewrite control-plane docs/records and the Rust crate; no production Python caller route was found. `rust/crates/v2m-core/src/lib.rs:3` also states the crate is intentionally not wired into Python runtime.

## Previous Finding Status

- Non-byte-aligned 7/12/20-bit fixture coverage: partially fixed. The current fixtures pass for consistent headers, but the Rust sample-width source still differs from Python `wave.getsampwidth()` on accepted headers where `block_align / channels` disagrees with rounded-up bits-per-sample.
- Non-finite slicing error surface: fixed for the reviewed seam. Rust explicitly maps NaN to `ValueError: cannot convert float NaN to integer` and infinities to `OverflowError: cannot convert float infinity to integer` at `rust/crates/v2m-core/src/asr_qwen_wav_pcm_decode.rs:123`, and fixtures assert start/duration NaN/infinity at `fixtures/asr_qwen_wav_pcm_decode_core.jsonl:15` through `fixtures/asr_qwen_wav_pcm_decode_core.jsonl:18`.
- Source-rate mismatch boundary fixture: fixed. Rust returns the documented same-rate-only boundary error at `rust/crates/v2m-core/src/asr_qwen_wav_pcm_decode.rs:52`, and the Rust-only fixture records it at `fixtures/asr_qwen_wav_pcm_decode_core.jsonl:19`.
- Runtime ownership: acceptable for this role. The Rust module is public inside `v2m-core`, but no Python bridge, subprocess route, HTTP route, or production caller wiring was found.

## Residual Risk

This behavior pass reviewed the same-rate WAV PCM fallback seam, forced-fallback slicing, fixture coverage, and production ownership. It did not review arbitrary malformed RIFF/WAVE errors, WAVE_FORMAT_EXTENSIBLE PCM, SciPy resampling parity, pydub primary decoding, non-WAV codecs, model sessions, or ASR inference.

## Promotion Note

This role blocks promotion. The unit should not be marked `verified` until the remaining sample-width source mismatch is fixed and fixture-backed.
