# asr_qwen_wav_pcm_decode_core - error_tracing_reviewer

Date: 2026-07-18
Decision: pass

## Findings

No findings.

Error surface and traceability are acceptable for this unpromoted Rust library
unit:

- Unsupported sample width keeps the legacy `ValueError` projection. Python
  raises `ValueError(f"Unsupported WAV sample width: {sample_width}")` in
  `/home/fuurin/code/Vocal2Midi-for-linux/inference/qwen3asr_dml/utils.py:80`;
  Rust returns `WavPcmDecodeError { error_type: "ValueError", message:
  "Unsupported WAV sample width: {sample_width}" }` at
  `rust/crates/v2m-core/src/asr_qwen_wav_pcm_decode.rs:45`. Fixture line 20
  asserts exact `ValueError` text for sample width 5.
- Non-finite slicing now preserves Python exception type and message. Rust maps
  NaN to `ValueError: cannot convert float NaN to integer` and infinities to
  `OverflowError: cannot convert float infinity to integer` at
  `rust/crates/v2m-core/src/asr_qwen_wav_pcm_decode.rs:123`. Fixture lines 15
  through 18 cover both `start_second` and `duration` cases, and the Python
  checker captures legacy exception type plus message at
  `bootstrap/check_asr_qwen_wav_pcm_decode_core.py:40`.
- Parser errors are path-safe and diagnosable for the owned byte-oriented seam.
  The Rust parser accepts in-memory bytes, not file paths, and maps malformed
  RIFF/WAVE structure into structured `WavPcmDecodeError` values without
  logging caller data at `rust/crates/v2m-core/src/asr_qwen_wav_pcm_decode.rs:163`.
  Exact malformed-container parity with Python `wave.Error` is not claimed by
  the manifest-backed fixture set; malformed parser errors should be rechecked
  if a later promotion exposes this parser as a user-facing file loader.
- The source-rate boundary is explicit and names the runtime owner that still
  owns resampling. The manifest assigns resampling to
  `asr_resample_poly_contract` at `manifest.yaml:1792`, and Rust returns a
  structured `ValueError` naming that owner plus source and target rates at
  `rust/crates/v2m-core/src/asr_qwen_wav_pcm_decode.rs:52`. Fixture line 19
  records the Rust-only same-rate boundary.
- No sensitive data is emitted. The implementation has no logging, filesystem,
  subprocess, or path-handling calls in
  `rust/crates/v2m-core/src/asr_qwen_wav_pcm_decode.rs`; the public functions
  accept WAV bytes and numeric parameters at
  `rust/crates/v2m-core/src/asr_qwen_wav_pcm_decode.rs:39` and
  `rust/crates/v2m-core/src/asr_qwen_wav_pcm_decode.rs:99`.
- No accidental production route was found. Production directories have no
  references to `asr_qwen_wav_pcm_decode`, `load_wav_audio_fallback_bytes`,
  `load_audio_forced_fallback_bytes`, or `WavPcmDecodeError`; the Rust module
  is only exported from the independent `v2m-core` crate at
  `rust/crates/v2m-core/src/lib.rs:10`.
- Rollback and runtime ownership remain clear. The unit is `reimplemented`,
  `current_owner: legacy`, with rollback to Python `load_audio` and
  `_load_wav_audio` at `manifest.yaml:1786`, `manifest.yaml:1788`, and
  `manifest.yaml:1807`. Record
  0110 says the Rust module is not wired into production Python callers at
  `records/0110-implement-asr-qwen-wav-pcm-decode-core.md:14`, and record 0111
  documents the post-review fix that removed `hound` and added explicit
  non-finite slicing errors at
  `records/0111-fix-asr-qwen-wav-pcm-decode-review-findings.md:22` and
  `records/0111-fix-asr-qwen-wav-pcm-decode-review-findings.md:36`.
- The latest implementation no longer depends on `hound`, `rodio`, or
  Symphonia. `rust/crates/v2m-core/Cargo.toml:12` lists only
  `encoding_rs`, `md-5`, `ndarray`, `saphyr-parser`, and `serde_json`, and a
  focused lockfile/dependency search found no `hound`, `rodio`, or `symphonia`
  entries.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_asr_qwen_wav_pcm_decode_core.py`:
  passed, `asr_qwen_wav_pcm_decode_core fixtures ok: 20 cases`.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml asr_qwen_wav_pcm_decode_core`:
  passed, 1 matching Rust fixture test.
- `jq -rc '{line:input_line_number,category:.category,call:.call,target_rate:.target_rate,start_second:.start_second,duration:.duration,ok:.expected.ok,error_type:.expected.error_type,message:.expected.message,shape:.expected.shape}' fixtures/asr_qwen_wav_pcm_decode_core.jsonl`:
  inspected all 20 fixture rows, including unsupported sample width, four
  non-finite slicing errors, and source-rate mismatch.
- `rg -n "asr_qwen_wav_pcm_decode|load_wav_audio_fallback_bytes|load_audio_forced_fallback_bytes|WavPcmDecodeError" /home/fuurin/code/Vocal2Midi-for-linux/inference /home/fuurin/code/Vocal2Midi-for-linux/application /home/fuurin/code/Vocal2Midi-for-linux/gui /home/fuurin/code/Vocal2Midi-for-linux/scripts /home/fuurin/code/Vocal2Midi-for-linux/web_server.py /home/fuurin/code/Vocal2Midi-for-linux/web_task_manager.py /home/fuurin/code/Vocal2Midi-for-linux/tests`:
  passed with no matches, confirming no production route in the checked Python
  runtime surfaces.
- `rg -n "println!|eprintln!|dbg!|log::|tracing::|env_logger|std::fs|File::|OpenOptions|Command::|PathBuf|Path" rust/crates/v2m-core/src/asr_qwen_wav_pcm_decode.rs`:
  passed with no matches, confirming no logging, path, subprocess, or
  filesystem surface in the Rust module.
- `rg -n "name = \"hound\"|hound|name = \"rodio\"|rodio|symphonia" rust/Cargo.lock rust/Cargo.toml rust/crates/v2m-core/Cargo.toml`:
  passed with no matches.

## Residual Risk

Malformed-container parser errors are not fixture-backed against Python
`wave.Error` type/message parity. That is acceptable for this same-rate PCM
decode unit because the manifest-backed public policy covers valid WAV PCM
fallback behavior, unsupported sample widths, slicing, and the explicit
same-rate boundary, and because no production route currently exposes the Rust
parser as a user-facing file loader.

Future promotion through file paths, subprocess JSON, PyO3, HTTP, logging, or a
Python runtime bridge needs a fresh error/tracing review for path redaction,
bridge error serialization, and malformed-file diagnostics.

## Promotion Note

This role does not block promotion. `asr_qwen_wav_pcm_decode_core` passes the
error/tracing rerun gate after record 0111.
