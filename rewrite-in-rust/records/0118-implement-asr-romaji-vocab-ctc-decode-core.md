# 0118 - Implement ASR Romaji Vocab CTC Decode Core

Date: 2026-07-18

## Unit

`asr_romaji_vocab_ctc_decode_core`

## Implementation

Added `rust/crates/v2m-core/src/asr_romaji_vocab_ctc_decode.rs` and exported it
from `v2m-core`.

The module implements:

- `load_vocab_from_json_str` for JSON vocab id inversion and blank-id fallback
- `decode_pred_ids` for CTC duplicate collapse, blank suppression, blank reset,
  and `<unk>` fallback
- ndarray-backed `decode_logits_f32` and `decode_logits_f64` with tie-first
  argmax behavior
- ndarray-backed integer and logit batch decode helpers
- typed `chunked` plus fixture-facing `chunked_json_values` for Python
  `int(chunk_size)` projection

No ONNX Runtime session creation, provider selection, audio loading, resampling,
batch padding/mask preparation, runtime Python route, PyO3 bridge, or subprocess
bridge was added.

## Fixtures

`fixtures/asr_romaji_vocab_ctc_decode_core.jsonl` has 24 Python golden cases
covering vocab behavior, CTC decode, logits/output dispatch, and chunk edge
cases.

## Verification

```bash
uv run python rewrite-in-rust/bootstrap/check_asr_romaji_vocab_ctc_decode_core.py
cargo test --manifest-path rust/Cargo.toml asr_romaji_vocab_ctc_decode_core
```

## State

`asr_romaji_vocab_ctc_decode_core` is now `reimplemented`. It still requires
independent `stage_behavior_reviewer` and `data_algorithm_reviewer` gates before
it can be marked `verified`.
