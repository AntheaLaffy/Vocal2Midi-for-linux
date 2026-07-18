# asr_qwen_wav_pcm_decode_core - data_algorithm_reviewer

Date: 2026-07-18
Decision: fail

## Findings

- Severity: medium
- Location: rust/crates/v2m-core/src/asr_qwen_wav_pcm_decode.rs:200
- Issue: The narrow RIFF/WAVE parser derives `sample_width` from `block_align / channels`, but the legacy path obtains `sample_width` from Python `wave.Wave_read.getsampwidth()`. Python 3.12 computes that width as `ceil(bits_per_sample / 8)` and does not use `wBlockAlign` for this value. The two formulas match the current 7/12/20-bit fixtures because their headers keep `block_align == channels * ceil(bits_per_sample / 8)`, but they diverge for Python-accepted WAV headers where `wBlockAlign` and `bits_per_sample` disagree. That can change 8/16/24/32 conversion selection, unsupported-width errors, frame counts, and decoded float values.
- Evidence: Legacy `inference/qwen3asr_dml/utils.py:61` calls `wav_file.getsampwidth()`. `uv run python` inspection of `/home/fuurin/.local/share/uv/python/cpython-3.12.13-linux-x86_64-gnu/lib/python3.12/wave.py:379` and `wave.py:404` showed `wBlockAlign` is read but `_sampwidth = (sampwidth + 7) // 8`, where `sampwidth` is the bits-per-sample field. A probe using the legacy `_load_wav_audio` accepted a same-rate PCM header with `block_align=5`, `bits_per_sample=16` and decoded it as 16-bit samples; the Rust source would compute `sample_width=5` at `asr_qwen_wav_pcm_decode.rs:200` and return `Unsupported WAV sample width: 5` at `asr_qwen_wav_pcm_decode.rs:45`.
- Required fix: Parse and store the `bits_per_sample` field from the `fmt ` chunk, derive Python-compatible sample width as `(bits_per_sample + 7) / 8`, and add at least one fixture where `block_align / channels` differs from `ceil(bits_per_sample / 8)` so this compatibility assumption is locked down. Keep the existing 7/12/20-bit fixtures as valid non-byte-aligned coverage.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_asr_qwen_wav_pcm_decode_core.py` from `/home/fuurin/code/Vocal2Midi-for-linux`: passed, `asr_qwen_wav_pcm_decode_core fixtures ok: 20 cases`.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml asr_qwen_wav_pcm_decode_core` from `/home/fuurin/code/Vocal2Midi-for-linux`: passed, 1 targeted Rust fixture test passed.
- `cargo tree --manifest-path rust/Cargo.toml -p v2m-core | rg "hound|symphonia|rodio"` from `rewrite-in-rust/`: no matches, confirming this crate no longer depends on `hound`, `rodio`, or Symphonia for the unit.
- `rg -n "name = \"hound\"|hound|symphonia|rodio" rust/Cargo.lock rust/crates/v2m-core/Cargo.toml` from `rewrite-in-rust/`: no matches, confirming the scoped Cargo files do not retain those dependencies.
- `jq -r '[input_line_number, .category, .call, .target_rate, .expected.ok, (.expected.error_type // ""), (.expected.message // ""), (.expected.values|length? // "")] | @tsv' fixtures/asr_qwen_wav_pcm_decode_core.jsonl`: confirmed 20 fixture cases, including unsigned 8-bit, signed 16/24/32-bit, stereo and three-channel mean, non-byte-aligned 7/12/20-bit headers, zero/negative/non-finite slicing, unsupported sample width 5, and a Rust-only source-rate mismatch boundary.
- `uv run python` probe against legacy `_load_wav_audio`: same-rate `block_align=5`, `bits_per_sample=16` decoded successfully as 16-bit float32 output, proving the sample-width formula difference reaches public fallback output.

## Residual Risk

The current fixtures prove normal same-rate PCM data conversion, 24-bit sign extension, channel means for up to three channels, Python-compatible NaN/infinity slicing messages, and the intended source-rate mismatch boundary. They do not prove large-channel float32 mean accumulation order, negative infinity slicing, WAVE_FORMAT_EXTENSIBLE PCM headers, or malformed headers beyond the sample-width mismatch described above.

## Promotion Note

This role blocks promotion until the sample-width derivation is aligned with Python `wave.getsampwidth()` or the unit contract is explicitly narrowed with fixtures and records documenting that malformed `wBlockAlign`/bits-per-sample disagreement is out of scope.
