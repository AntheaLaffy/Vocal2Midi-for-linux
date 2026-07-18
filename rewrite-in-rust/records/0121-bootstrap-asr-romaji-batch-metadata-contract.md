# 0121 - Bootstrap ASR Romaji Batch Metadata Contract

Date: 2026-07-18

## Unit

`asr_romaji_batch_metadata_contract`

## Decision

Confirm `asr_romaji_batch_metadata_contract` as a narrow fake-session metadata
and batch-padding unit.

The unit remains separate from `asr_romaji_vocab_ctc_decode_core`,
`asr_resample_poly_contract`, and any future runtime promotion. It uses
synthetic waveforms and fake metadata only.

## Dependency Decision

Use `ndarray` for Rust 2D batch arrays. Do not expand or link ONNX Runtime,
SoundFile/libsndfile, or SciPy for this unit:

- ONNX Runtime is represented by fake `.get_inputs()` metadata.
- `load_audio` is monkeypatched in fixtures and stays legacy-owned.
- resampling is already owned by `asr_resample_poly_contract`.

## Fixtures

Added:

- `fixtures/asr_romaji_batch_metadata_contract.jsonl`
- `bootstrap/check_asr_romaji_batch_metadata_contract.py`

The fixture set has 26 Python golden cases covering metadata shape extraction,
dtype mapping, prepare-batch success paths, error ordering, dtype casts, optional
attention masks including float16, used lengths, missing synthetic waveform
keys, negative fixed sample dimensions, and the legacy fixed-zero-samples
fallback.

## Verification

```bash
uv run python rewrite-in-rust/bootstrap/check_asr_romaji_batch_metadata_contract.py
```
