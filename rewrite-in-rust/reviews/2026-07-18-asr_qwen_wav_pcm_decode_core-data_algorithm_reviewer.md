# asr_qwen_wav_pcm_decode_core - data_algorithm_reviewer

Date: 2026-07-18
Decision: pass

## Findings

No findings.

## Data and Algorithm Review

- Unit and role: reviewed `asr_qwen_wav_pcm_decode_core` as `data_algorithm_reviewer` only.
- Boundary: confirmed the manifest keeps this child unit to same-rate WAV PCM fallback decoding, final forced-fallback slicing, and excludes resampling to `asr_resample_poly_contract` (`manifest.yaml:1784`, `manifest.yaml:1792`).
- Writer/reviewer separation: this report is review-only and does not modify production code.
- Legacy source: `_load_wav_audio` uses Python `wave`, NumPy integer conversion, explicit denominators for 8/16/24/32-bit PCM, channel mean, optional SciPy resampling, and final `float32` projection (`/home/fuurin/code/Vocal2Midi-for-linux/inference/qwen3asr_dml/utils.py:57`). `load_audio` applies final Python slice semantics after fallback (`/home/fuurin/code/Vocal2Midi-for-linux/inference/qwen3asr_dml/utils.py:114`).
- WAV PCM conversion: Rust uses `hound::WavReader`, rejects non-1..4 byte sample widths, rejects non-same-rate input for this unit, and rejects non-integer sample formats before conversion (`rust/crates/v2m-core/src/asr_qwen_wav_pcm_decode.rs:33`). The conversion denominators match Python: 8-bit `/128`, 16-bit `/32768`, 24-bit `/8388608`, and 32-bit `/2147483648` (`rust/crates/v2m-core/src/asr_qwen_wav_pcm_decode.rs:63`).
- 8-bit unsigned centering: the selected `hound` read path explicitly converts unsigned WAV 8-bit samples to signed values by subtracting 128 before exposing signed samples (`/home/fuurin/.cargo/registry/src/rsproxy.cn-e3de039b2554c837/hound-3.5.1/src/lib.rs:98`, `/home/fuurin/.cargo/registry/src/rsproxy.cn-e3de039b2554c837/hound-3.5.1/src/lib.rs:201`). That matches Python `(uint8.astype(float32) - 128.0) / 128.0`.
- 24-bit sign extension: Python sign-extends little-endian 24-bit samples with `np.where(signed & 0x800000, signed - 0x1000000, signed)` (`/home/fuurin/code/Vocal2Midi-for-linux/inference/qwen3asr_dml/utils.py:68`). `hound` sign-extends 24-bit reads before returning `i32` samples (`/home/fuurin/.cargo/registry/src/rsproxy.cn-e3de039b2554c837/hound-3.5.1/src/read.rs:143`, `/home/fuurin/.cargo/registry/src/rsproxy.cn-e3de039b2554c837/hound-3.5.1/src/lib.rs:274`).
- Mean and float32 behavior: Rust performs per-sample `f32` conversion before channel averaging and returns `Vec<f32>` (`rust/crates/v2m-core/src/asr_qwen_wav_pcm_decode.rs:63`, `rust/crates/v2m-core/src/asr_qwen_wav_pcm_decode.rs:119`). The 12 Python golden fixtures cover mono, stereo, three-channel mean, 32-bit max rounding to `1.0`, and float32-valued outputs (`fixtures/asr_qwen_wav_pcm_decode_core.jsonl:1`).
- Slicing semantics: Rust now computes slice indices with signed truncation toward zero, normalizes negative Python slice indexes relative to length, clamps to `[0, len]`, and returns empty when normalized start exceeds normalized end (`rust/crates/v2m-core/src/asr_qwen_wav_pcm_decode.rs:87`). Fixtures cover positive slicing, zero-duration truthiness, negative start-only slicing, and negative start plus duration producing an empty slice (`fixtures/asr_qwen_wav_pcm_decode_core.jsonl:7`).
- Dependency choice: `hound 3.5` is a narrow WAV parser/sample decoder dependency in `v2m-core`, with no `rodio` or Symphonia playback/codec chain pulled in (`rust/crates/v2m-core/Cargo.toml:15`, `rust/Cargo.lock:84`). This matches the dependency record's capability-level choice for WAV container parsing and integer PCM extraction, with Python-specific math kept local (`dependencies/asr_text_postprocess_contract.yaml:70`).
- Fixture coverage: current fixtures contain 12 Python 3.12 golden cases for unsigned 8-bit, signed 16-bit, signed 24-bit sign extension, signed 32-bit rounding, stereo and three-channel mean, forced-fallback positive/zero-duration/negative-start slicing, and unsupported sample width 5 (`records/0110-implement-asr-qwen-wav-pcm-decode-core.md:20`, `fixtures/asr_qwen_wav_pcm_decode_core.jsonl:1`).

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_asr_qwen_wav_pcm_decode_core.py`: passed, `asr_qwen_wav_pcm_decode_core fixtures ok: 12 cases`.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml asr_qwen_wav_pcm_decode_core`: passed, 1 test passed.
- `cargo tree --manifest-path rewrite-in-rust/rust/Cargo.toml -p v2m-core --edges normal`: confirmed direct `hound v3.5.1`; no `rodio`/Symphonia dependency chain.
- `uv run python -m py_compile inference/qwen3asr_dml/utils.py`: passed.

## Residual Risk

This review does not prove SciPy `resample_poly` parity for mismatched source and target rates; that remains explicitly owned by `asr_resample_poly_contract`. It also does not claim arbitrary pydub/ffmpeg/audio-codec parity, IEEE float WAV parity, or all malformed WAV error-text parity.

The fixture set is strong for the claimed PCM math and slicing edges. It does not cover non-finite `start_second`/`duration`, 24-bit samples stored in 4-byte containers, or incomplete trailing channel frames; those are residual edge risks outside the current golden cases.

## Promotion Note

This role does not block promotion. `asr_qwen_wav_pcm_decode_core` passes the data/algorithm review gate.
