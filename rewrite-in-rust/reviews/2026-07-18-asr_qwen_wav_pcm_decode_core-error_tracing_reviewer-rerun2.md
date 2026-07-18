# asr_qwen_wav_pcm_decode_core - error_tracing_reviewer rerun2

Date: 2026-07-18
Decision: pass

## Scope

- Unit: `asr_qwen_wav_pcm_decode_core`
- Role: `error_tracing_reviewer` only
- Filesystem basis: latest workspace state after `records/0111-fix-asr-qwen-wav-pcm-decode-review-findings.md`
- Writer/reviewer separation: this review did not modify production code.

## Findings

No findings.

## Evidence

- Unsupported sample width remains a structured Python-compatible `ValueError`.
  Rust validates `sample_width` after parsing and returns `Unsupported WAV sample
  width: 5` through `WavPcmDecodeError` at
  `rust/crates/v2m-core/src/asr_qwen_wav_pcm_decode.rs:45`. Fixture line 22
  asserts `{"error_type":"ValueError","message":"Unsupported WAV sample width: 5"}`.
- Non-finite slicing now preserves Python error type and message. NaN maps to
  `ValueError: cannot convert float NaN to integer`; infinities map to
  `OverflowError: cannot convert float infinity to integer` at
  `rust/crates/v2m-core/src/asr_qwen_wav_pcm_decode.rs:123`. Fixture lines 17
  through 20 cover start and duration NaN/infinity, and the Python harness
  captures legacy exception type plus `str(exc)` at
  `bootstrap/check_asr_qwen_wav_pcm_decode_core.py:40`.
- The hand-written parser keeps parser-originated failures inside the structured
  `WavPcmDecodeError { error_type, message }` surface without logging bytes,
  filenames, paths, or model/session state. The parser is limited to RIFF/WAVE,
  `fmt `, channel count, sample rate, sample width, and `data` chunk extraction
  at `rust/crates/v2m-core/src/asr_qwen_wav_pcm_decode.rs:163`.
- Source-rate mismatch is explicit and diagnosable at the current same-rate
  boundary. Rust returns `WAV resampling is owned by asr_resample_poly_contract:
  source_rate=8000, target_rate=16000` at
  `rust/crates/v2m-core/src/asr_qwen_wav_pcm_decode.rs:52`; fixture line 21
  records this as a Rust-only boundary fixture. The manifest also states that
  resampling is owned by `asr_resample_poly_contract` at `manifest.yaml:1792`.
- No sensitive-data leak was found in the scoped Rust module. Its public API
  accepts byte slices, not paths; error messages include only numeric sample
  widths, sample rates, and fixed parser text at
  `rust/crates/v2m-core/src/asr_qwen_wav_pcm_decode.rs:39` and
  `rust/crates/v2m-core/src/asr_qwen_wav_pcm_decode.rs:99`. A logging/path scan
  found no production-path logging, filesystem, command, or path APIs in the
  module; the only `panic!` matches are test-only assertions.
- No accidental production route was found. Exact-symbol search for
  `load_wav_audio_fallback_bytes`, `load_audio_forced_fallback_bytes`,
  `WavPcmDecodeError`, and `asr_qwen_wav_pcm_decode` under production Python,
  application, GUI, scripts, Web, and tests returned no matches. The Rust crate
  still documents that it is intentionally not wired into Python runtime at
  `rust/crates/v2m-core/src/lib.rs:3`.
- Rollback/runtime owner clarity is sufficient for this role. The manifest keeps
  `current_owner: legacy` at `manifest.yaml:1788` and records rollback to
  `inference.qwen3asr_dml.utils.load_audio` plus `_load_wav_audio` at
  `manifest.yaml:1807`. The implementation and fix records repeat that the Rust
  module is not wired into production callers and that Python remains runtime
  owner at `records/0110-implement-asr-qwen-wav-pcm-decode-core.md:14` and
  `records/0111-fix-asr-qwen-wav-pcm-decode-review-findings.md:76`.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_asr_qwen_wav_pcm_decode_core.py`
  from `/home/fuurin/code/Vocal2Midi-for-linux`: passed,
  `asr_qwen_wav_pcm_decode_core fixtures ok: 22 cases`.
- `cargo test --manifest-path rust/Cargo.toml asr_qwen_wav_pcm_decode_core`
  from `/home/fuurin/code/Vocal2Midi-for-linux/rewrite-in-rust`: passed, 1
  targeted Rust fixture test passed.
- `jq -rc '{line:input_line_number,category:.category,call:.call,target_rate:.target_rate,start_second:.start_second,duration:.duration,ok:.expected.ok,error_type:.expected.error_type,message:.expected.message,shape:.expected.shape}' fixtures/asr_qwen_wav_pcm_decode_core.jsonl`:
  confirmed 22 fixture cases, including unsupported sample width, four
  non-finite slicing errors, and the source-rate boundary.
- `rg -n "load_wav_audio_fallback_bytes|load_audio_forced_fallback_bytes|WavPcmDecodeError|asr_qwen_wav_pcm_decode" /home/fuurin/code/Vocal2Midi-for-linux/inference /home/fuurin/code/Vocal2Midi-for-linux/application /home/fuurin/code/Vocal2Midi-for-linux/gui /home/fuurin/code/Vocal2Midi-for-linux/scripts /home/fuurin/code/Vocal2Midi-for-linux/web_server.py /home/fuurin/code/Vocal2Midi-for-linux/web_task_manager.py /home/fuurin/code/Vocal2Midi-for-linux/tests`:
  no matches.
- `rg -n "println!|eprintln!|dbg!|log::|tracing::|env_logger|std::fs|File::|OpenOptions|Command::|PathBuf|Path|panic!" rust/crates/v2m-core/src/asr_qwen_wav_pcm_decode.rs`:
  only test-only `panic!` matches at lines 263 and 268.
- `rg -n 'name = "(hound|rodio|symphonia|symphonia-wav|symphonia-pcm)"|hound|rodio|symphonia' rust/Cargo.lock rust/crates/v2m-core/Cargo.toml`:
  no matches; the removed parser/playback dependencies are not present in the
  scoped Cargo files.

## Residual Risk

Malformed RIFF/WAVE parser error parity is not broadly fixture-claimed. The
current parser maps malformed headers into structured `ValueError` messages,
while legacy Python may surface `wave.Error`, `EOFError`, or lower-level parser
messages for invalid files. This is not blocking for this unit because the
manifested policy is same-rate WAV PCM fallback behavior plus the documented
unsupported-width, non-finite slicing, and source-rate-boundary errors.

## Promotion Note

This role does not block promotion. `asr_qwen_wav_pcm_decode_core` passes the
error/tracing review gate for the current 22-case same-rate WAV PCM fallback
surface. The coordinator should still wait for all required review roles before
updating manifest state.
