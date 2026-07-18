# asr_qwen_wav_pcm_decode_core - stage_behavior_reviewer rerun2

Date: 2026-07-18
Decision: pass

## Scope

- Unit: `asr_qwen_wav_pcm_decode_core`
- Role: `stage_behavior_reviewer` only
- Files reviewed: `manifest.yaml`, records `0110` and `0111`, the 22-case fixture file, Python fixture check, Rust implementation, scoped Cargo files, and legacy `inference/qwen3asr_dml/utils.py`.
- Production code modified: no

## Findings

No findings.

## Evidence

- The manifest keeps the unit `reimplemented`, legacy-owned, and same-rate-only, with resampling explicitly owned by `asr_resample_poly_contract` (`manifest.yaml:1784`, `manifest.yaml:1788`, `manifest.yaml:1792`).
- Legacy Python reads `wave.getsampwidth()` and dispatches by byte sample width before optional SciPy resampling (`/home/fuurin/code/Vocal2Midi-for-linux/inference/qwen3asr_dml/utils.py:57`, `/home/fuurin/code/Vocal2Midi-for-linux/inference/qwen3asr_dml/utils.py:61`, `/home/fuurin/code/Vocal2Midi-for-linux/inference/qwen3asr_dml/utils.py:64`, `/home/fuurin/code/Vocal2Midi-for-linux/inference/qwen3asr_dml/utils.py:85`).
- Rust now derives sample width from `bits_per_sample.div_ceil(8)` at `rust/crates/v2m-core/src/asr_qwen_wav_pcm_decode.rs:199`, matching Python 3.12 `wave.getsampwidth()` behavior for the reviewed fixture surface.
- The 22-case fixture file includes non-byte-aligned 7/12/20-bit headers, block-align disagreement cases, non-finite start/duration slicing, the same-rate boundary, and unsupported sample-width 5 (`fixtures/asr_qwen_wav_pcm_decode_core.jsonl:12`, `fixtures/asr_qwen_wav_pcm_decode_core.jsonl:15`, `fixtures/asr_qwen_wav_pcm_decode_core.jsonl:17`, `fixtures/asr_qwen_wav_pcm_decode_core.jsonl:21`, `fixtures/asr_qwen_wav_pcm_decode_core.jsonl:22`).
- Independent header inspection confirmed the block-align disagreement fixtures use `bits_per_sample=12, block_align=3, ceil=2` and `bits_per_sample=20, block_align=4, ceil=3`.
- Rust maps non-finite slicing to Python-compatible error surfaces at `rust/crates/v2m-core/src/asr_qwen_wav_pcm_decode.rs:123`, with fixture assertions for start and duration NaN/infinity at fixture lines 17-20.
- Rust keeps the documented same-rate-only boundary at `rust/crates/v2m-core/src/asr_qwen_wav_pcm_decode.rs:52`, and fixture line 21 asserts the boundary error.
- The Rust crate remains outside the production Python runtime. `rust/crates/v2m-core/src/lib.rs:1` says the crate is intentionally not wired into Python runtime, and a production-directory `rg` sweep found no callers for the Rust WAV API names.
- `rust/crates/v2m-core/Cargo.toml:12` through `rust/crates/v2m-core/Cargo.toml:17` contain no WAV parser dependency for this unit, and the scoped `Cargo.lock`/`Cargo.toml` scan found no `hound`, `rodio`, or Symphonia entries.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_asr_qwen_wav_pcm_decode_core.py` from `/home/fuurin/code/Vocal2Midi-for-linux`: passed, `asr_qwen_wav_pcm_decode_core fixtures ok: 22 cases`.
- `cargo test --manifest-path rust/Cargo.toml asr_qwen_wav_pcm_decode_core` from `rewrite-in-rust`: passed, 1 targeted Rust fixture test passed.
- `uv run python -m py_compile inference/qwen3asr_dml/utils.py` from `/home/fuurin/code/Vocal2Midi-for-linux`: passed.
- `cargo test --manifest-path rust/Cargo.toml` from `rewrite-in-rust`: passed, 123 `v2m_core` tests, 5 `v2m_quant_bridge` tests, and doc tests passed.
- `jq -r 'select(.category|test("bits|block_align|nan|inf|source_rate|unsupported")) ...' fixtures/asr_qwen_wav_pcm_decode_core.jsonl`: confirmed fixture lines 12-22 cover the prior behavior gaps.
- `uv run python - <<'PY' ...` header inspection: confirmed fixture line 15 uses `block_align=3` with `bits_per_sample=12`, and fixture line 16 uses `block_align=4` with `bits_per_sample=20`.
- `rg -n "asr_qwen_wav_pcm_decode|load_wav_audio_fallback_bytes|load_audio_forced_fallback_bytes|WavPcmDecodeError" inference application gui scripts web_server.py web_task_manager.py tests` from `/home/fuurin/code/Vocal2Midi-for-linux`: no production matches.
- `rg -n "name = \"(hound|rodio|symphonia|symphonia-wav|symphonia-pcm)\"|\\b(hound|rodio|symphonia)\\b" rust/Cargo.lock rust/crates/v2m-core/Cargo.toml`: no matches.

## Residual Risk

This review proves the current fixture-backed public seam for same-rate PCM WAV fallback and forced-fallback slicing. It does not review resampling behavior, malformed/truncated WAV behavior outside the fixture contract, or non-`f64` Python argument coercions; those remain outside this unit's behavior boundary unless a later promotion record expands it.

## Promotion Note

This role does not block promotion. `asr_qwen_wav_pcm_decode_core` passes the rerun2 `stage_behavior_reviewer` gate.
