# 0122 - Implement ASR Romaji Batch Metadata Contract

Date: 2026-07-18

## Unit

`asr_romaji_batch_metadata_contract`

## Change

Added an independent Rust `v2m-core` module for the fake-session metadata and
batch preparation contract from `inference/romaji_asr/common.py`.

The Rust module implements:

- `get_fixed_batch_size` and `get_fixed_num_samples` over fixture-owned metadata
- Python-compatible bool-as-int dimension handling
- case-sensitive `ort_type_to_numpy_dtype` substring mapping
- `prepare_batch_from_waveforms` over synthetic waveforms
- ndarray-backed `input_values` and `attention_mask` matrices
- dtype projection for float16, float32, int64, and int32 feeds
- Python-compatible error ordering for empty paths, fixed batch mismatch, and
  missing `input_values`
- Python-compatible error projection for missing synthetic waveform keys and
  negative fixed sample dimensions

Added `half` for float16 feed projection. `ndarray` remains the owner of 2D
matrix storage; ONNX Runtime sessions, audio IO, resampling, and model execution
remain legacy-owned.

## Dependency Lesson

The earlier WAV PCM unit already followed the `rodio` dependency trail to
Symphonia before narrowing the final implementation. Keep that pattern for later
audio and AI inference work: start from existing Rust-side high-level crates
such as `rodio`, inspect their concrete codec/model dependencies, and prefer a
small crate-owned lower layer plus compatibility adapter when fixtures prove it.
Only choose a hand-written replacement when the semantic gap is smaller and
better verified than the crate integration.

## Verification

```bash
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml asr_romaji_batch_metadata_contract -- --nocapture
```

Result before review fixes: passed, 23 JSONL fixture cases matched.

## State

Mark the unit `reimplemented`. Required independent reviews are still pending:

- `dependency_bootstrap_reviewer`
- `stage_behavior_reviewer`
- `error_tracing_reviewer`

## Rollback

Keep `inference.romaji_asr.common` metadata and `prepare_batch` helpers as the
runtime owners.
