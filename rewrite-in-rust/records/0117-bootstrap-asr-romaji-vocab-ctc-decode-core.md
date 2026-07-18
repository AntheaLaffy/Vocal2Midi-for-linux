# 0117 - Bootstrap ASR Romaji Vocab CTC Decode Core

Date: 2026-07-18

## Unit

`asr_romaji_vocab_ctc_decode_core`

## Decision

Confirm `asr_romaji_vocab_ctc_decode_core` as a narrow helper unit.

The source file imports ONNX Runtime, SoundFile, SciPy, and NumPy, but the
selected public boundary reaches only JSON vocab parsing, NumPy argmax/dtype
dispatch, CTC list decoding, and chunked list iteration.

## Boundary

In scope:

- `load_vocab`
- `decode_pred_ids`
- `decode_logits`
- `decode_outputs`
- `chunked`

Out of scope:

- `load_audio`
- `create_session`
- `get_fixed_batch_size`
- `get_fixed_num_samples`
- `ort_type_to_numpy_dtype`
- `prepare_batch`
- ONNX Runtime session execution
- audio file IO and resampling

## Dependency Decision

Use `ndarray` in Rust for array-shaped logits and batch outputs. Hand-write the
Python-specific CTC collapse, blank handling, unknown fallback, vocab id
inversion, and chunk-size coercion against fixtures.

Do not expand ONNX Runtime, SoundFile/libsndfile, or SciPy sources for this unit.
Those dependencies do not reach the selected helper behavior.

## Fixtures

Added:

- `fixtures/asr_romaji_vocab_ctc_decode_core.jsonl`
- `bootstrap/check_asr_romaji_vocab_ctc_decode_core.py`

The fixture set has 20 Python golden cases covering vocab blank selection/id
inversion, CTC collapse, argmax tie-first behavior, decode-output dtype
dispatch, batch iteration, and chunking edge cases.

## Verification

```bash
uv run python rewrite-in-rust/bootstrap/check_asr_romaji_vocab_ctc_decode_core.py
```
