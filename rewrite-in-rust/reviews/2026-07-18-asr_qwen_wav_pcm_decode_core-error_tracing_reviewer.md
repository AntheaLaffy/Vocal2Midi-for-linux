# asr_qwen_wav_pcm_decode_core - error_tracing_reviewer

Date: 2026-07-18
Decision: pass

## Findings

No findings.

The error surface is appropriate for this unpromoted Rust library unit:

- Unsupported sample width keeps Python `ValueError` parity. The legacy fallback
  raises `ValueError(f"Unsupported WAV sample width: {sample_width}")` at
  `/home/fuurin/code/Vocal2Midi-for-linux/inference/qwen3asr_dml/utils.py:79`,
  and Rust returns `WavPcmDecodeError { error_type: "ValueError", message:
  "Unsupported WAV sample width: {sample_width}" }` at
  `rust/crates/v2m-core/src/asr_qwen_wav_pcm_decode.rs:43`. The fixture asserts
  exact `ValueError` text for sample width 5 at
  `fixtures/asr_qwen_wav_pcm_decode_core.jsonl:12`.
- Hound parser and sample-read failures are mapped into the same structured
  Rust error shape at `rust/crates/v2m-core/src/asr_qwen_wav_pcm_decode.rs:38`
  and `rust/crates/v2m-core/src/asr_qwen_wav_pcm_decode.rs:82`. The mapping is
  path-safe because the public functions accept bytes, use `Cursor`, and do not
  log or include caller filesystem paths at
  `rust/crates/v2m-core/src/asr_qwen_wav_pcm_decode.rs:33`.
- Sample-rate mismatch is explicitly non-owned by this unit. The manifest
  assigns resampling to `asr_resample_poly_contract` at `manifest.yaml:1792`,
  and Rust returns a diagnosable `ValueError` naming that owner plus
  source/target rates at `rust/crates/v2m-core/src/asr_qwen_wav_pcm_decode.rs:49`.
- The implementation emits no logs and handles slicing without panic for the
  fixture-covered negative-start and empty-slice cases at
  `rust/crates/v2m-core/src/asr_qwen_wav_pcm_decode.rs:94` and
  `rust/crates/v2m-core/src/asr_qwen_wav_pcm_decode.rs:103`.
- Rollback and runtime ownership are clear: manifest state is `reimplemented`
  with `current_owner: legacy` at `manifest.yaml:1786`, and rollback keeps
  `inference.qwen3asr_dml.utils.load_audio` plus `_load_wav_audio` as owners at
  `manifest.yaml:1805`. Record 0110 repeats that the Rust module is not wired
  into production Python callers at `records/0110-implement-asr-qwen-wav-pcm-decode-core.md:14`
  and names the same rollback at `records/0110-implement-asr-qwen-wav-pcm-decode-core.md:59`.
- No accidental production route was found. The Rust module is exported only
  from the independent `v2m-core` crate at `rust/crates/v2m-core/src/lib.rs:10`,
  with `hound` scoped to `v2m-core` dependencies at
  `rust/crates/v2m-core/Cargo.toml:15` and locked as `hound 3.5.1` at
  `rust/Cargo.lock:83`.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_asr_qwen_wav_pcm_decode_core.py`:
  passed, 12 fixture cases.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml asr_qwen_wav_pcm_decode_core`:
  passed, 1 matching Rust fixture test.
- `rg -n "asr_qwen_wav_pcm_decode|load_wav_audio_fallback_bytes|load_audio_forced_fallback_bytes|WavPcmDecodeError" /home/fuurin/code/Vocal2Midi-for-linux --glob '!rewrite-in-rust/reviews/**' --glob '!rewrite-in-rust/target/**'`:
  inspected; matches were limited to rewrite control-plane artifacts, the Rust
  module, and the `v2m-core` module export.
- `rg -n "hound|asr_qwen_wav_pcm_decode|load_wav_audio_fallback_bytes|load_audio_forced_fallback_bytes|WavPcmDecodeError" rust/Cargo.lock rust/crates/v2m-core/src rust/crates/v2m-core/Cargo.toml`:
  inspected; `hound` is present only as the WAV parser dependency for this
  independent Rust crate, and no logging/bridge surface was introduced.

## Residual Risk

This review does not prove future bridge or path-based caller redaction because
the current public Rust seam accepts in-memory WAV bytes and is not connected to
Python runtime routing. If a later promotion introduces file paths, subprocess
JSON, PyO3, HTTP, or logging, that promotion needs a fresh error/tracing review.

## Promotion Note

This role does not block promotion. The unit is ready for coordinator state
update after the remaining required review roles pass.
