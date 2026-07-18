# 0123 - Fix ASR Romaji Batch Metadata Review Findings

Date: 2026-07-18

## Unit

`asr_romaji_batch_metadata_contract`

## Trigger

Initial reviews found three fixture gaps:

- `error_tracing_reviewer` failed the gate because negative fixed sample
  dimensions were silently converted into a zero-width Rust batch.
- `behavior_reviewer` noted missing synthetic waveform keys were treated as
  empty waveforms instead of surfacing a load error after the load call.
- `dependency_bootstrap_reviewer` noted that `attention_mask` float16 was
  implemented but not fixture-backed.

## Change

Added three Python-generated golden fixtures:

- `prepare_negative_fixed_num_samples_error_after_load`
- `prepare_missing_waveform_key_error_after_load`
- `prepare_attention_mask_float16_type`

Updated the Rust helper to:

- return `ValueError: negative dimensions are not allowed` after recording load
  calls when a fixed sample dimension is negative
- return `KeyError` with the missing synthetic path after recording the load
  call when `prepare_batch_from_waveforms` lacks a waveform entry
- keep the existing `half::f16` mask cast covered by fixture evidence

## Verification

```bash
uv run python rewrite-in-rust/bootstrap/check_asr_romaji_batch_metadata_contract.py
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml asr_romaji_batch_metadata_contract
```

## Rollback

Keep `inference.romaji_asr.common` metadata and `prepare_batch` helpers as the
runtime owners.
