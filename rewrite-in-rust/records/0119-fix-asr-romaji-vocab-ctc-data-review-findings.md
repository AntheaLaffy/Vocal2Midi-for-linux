# 0119 - Fix ASR Romaji Vocab CTC Data Review Findings

Date: 2026-07-18

## Unit

`asr_romaji_vocab_ctc_decode_core`

## Trigger

The first `data_algorithm_reviewer` report failed on two uncovered numeric
edge cases:

- `decode_outputs_uint` wrapped `uint64` ids above `i64::MAX`
- logits argmax did not match NumPy when a later NaN appears in a row

## Fixes

Changed the Rust token id representation from `i64` to `i128` for vocab maps,
blank ids, CTC state, and integer decode paths. This preserves all signed `i64`
and unsigned `u64` fixture ids without wrapping.

Updated logits argmax to match NumPy's observed NaN behavior: the first NaN in a
row wins over finite values, and later finite values do not replace an existing
NaN winner.

Expanded `fixtures/asr_romaji_vocab_ctc_decode_core.jsonl` from 20 to 24 cases:

- one large `uint64` decode-output case with ids above `i64::MAX`
- two direct logits NaN argmax cases
- one batched float-logits NaN case

Updated the Python checker to parse `nan`/`inf` string sentinels explicitly
before constructing NumPy arrays.

## Verification

```bash
uv run python rewrite-in-rust/bootstrap/check_asr_romaji_vocab_ctc_decode_core.py
cargo test --manifest-path rust/Cargo.toml asr_romaji_vocab_ctc_decode_core
```
