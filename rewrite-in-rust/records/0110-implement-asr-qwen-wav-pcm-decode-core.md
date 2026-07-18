# 0110 - Implement ASR Qwen WAV PCM Decode Core

Date: 2026-07-18

## Unit

`asr_qwen_wav_pcm_decode_core`

## Change

Added an independent Rust implementation for the same-rate WAV PCM fallback
contract from `inference/qwen3asr_dml/utils.py`.

The Rust module is
`rewrite-in-rust/rust/crates/v2m-core/src/asr_qwen_wav_pcm_decode.rs`. It is not
wired into production Python callers.

## Fixture Evidence

`rewrite-in-rust/fixtures/asr_qwen_wav_pcm_decode_core.jsonl` contains 20 cases
covering:

- unsigned 8-bit PCM centering by 128
- signed 16-bit PCM normalization by 32768
- signed 24-bit little-endian sign extension and normalization by 8388608
- signed 32-bit PCM normalization by 2147483648
- mono, stereo, and three-channel mean behavior
- same-rate float32 output
- final `load_audio` positive, zero-duration, and negative-start slicing after
  pydub fallback is forced
- Python `ValueError`/`OverflowError` text for NaN and infinity slicing inputs
- Python `wave.getsampwidth()` byte-container behavior for non-byte-aligned
  7/12/20-bit PCM headers
- unsupported sample width 5 `ValueError` message
- Rust-only same-rate boundary behavior for source-rate mismatch, with
  resampling deferred to `asr_resample_poly_contract`

`rewrite-in-rust/bootstrap/check_asr_qwen_wav_pcm_decode_core.py` verifies the
fixture file against the current Python implementation.

## Dependency Boundary

The writer checked the Rust audio route suggested by the project context:
`rodio 0.22.2` enables WAV support through `symphonia-wav` and
`symphonia-pcm`. For this compatibility seam, the lower-level requirement is
WAV spec visibility plus integer PCM sample extraction, not playback or
arbitrary codec decoding.

The first implementation used `hound 3.5`, which exposes WAV sample format, bit
depth, channel count, sample rate, 8-bit unsigned-to-signed PCM projection, and
24-bit little-endian sign extension. Behavior review then showed that `hound`
rejects non-byte-aligned `bits_per_sample` headers before this compatibility
layer can apply Python `wave.getsampwidth()` byte-container semantics. The
final implementation therefore removes `hound` and hand-writes the narrow
RIFF/WAVE PCM parser for this unit.

The Python-specific normalization denominators, channel mean, same-rate
boundary, unsupported sample-width message, non-finite slicing errors, and
slicing order are hand-written and fixture-backed.

This unit does not use `rodio`, Symphonia, `hound`, NumPy, SciPy, pydub runtime
decoding, ONNX Runtime, model sessions, `ndarray`, PyO3, subprocess bridges,
HTTP, or production routing. Resampling remains owned by
`asr_resample_poly_contract`.

## Manifest State

`asr_qwen_wav_pcm_decode_core` is now `reimplemented`. It requires independent
behavior, data/algorithm, and error/tracing reviews before it can be marked
`verified`.

## Rollback

Keep `inference.qwen3asr_dml.utils.load_audio` and `_load_wav_audio` as runtime
owners.
