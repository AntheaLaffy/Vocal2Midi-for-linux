# asr_romaji_vocab_ctc_decode_core - data_algorithm_reviewer

FAIL

Date: 2026-07-18
Decision: fail
Role: data_algorithm_reviewer
Unit: asr_romaji_vocab_ctc_decode_core

## Findings

- Severity: high
- Location: rust/crates/v2m-core/src/asr_romaji_vocab_ctc_decode.rs:113
- Issue: `decode_outputs_uint` does not preserve Python unsigned integer token-id semantics for values above `i64::MAX`.
- Evidence: Python `decode_outputs` branches on `np.issubdtype(item.dtype, np.integer)` and then `decode_pred_ids` converts each `np.uint64` element through Python `int(...)`, preserving arbitrary-size positive integers (`../inference/romaji_asr/common.py:123`, `../inference/romaji_asr/common.py:138`). Rust accepts `ArrayView2<'_, u64>` but maps each value with `*value as i64` before CTC decoding (`rust/crates/v2m-core/src/asr_romaji_vocab_ctc_decode.rs:113`, `rust/crates/v2m-core/src/asr_romaji_vocab_ctc_decode.rs:120`). That wraps large unsigned values instead of preserving the positive token id. A targeted Python probe returned `[['big', 'max']]` for `np.array([[9223372036854775808, 18446744073709551615]], dtype=np.uint64)` with matching positive `id2token` keys; the Rust code would pass `-9223372036854775808` and `-1` into the CTC state machine. The only unsigned fixture uses small `uint32` ids at `fixtures/asr_romaji_vocab_ctc_decode_core.jsonl:12`, so this gap is currently untested.
- Required fix: Either change the Rust integer-id representation/uint path so `uint64` values above `i64::MAX` remain distinguishable and lookup-compatible, or explicitly narrow the contract to signed/small class ids and remove the claimed broad uint equivalence. Add a fixture for at least one `uint64` value above `i64::MAX`.

- Severity: medium
- Location: rust/crates/v2m-core/src/asr_romaji_vocab_ctc_decode.rs:158
- Issue: Rust argmax does not match `np.argmax(..., axis=-1)` when a later NaN appears in a logits row.
- Evidence: Python `decode_logits` delegates directly to `np.argmax(logits, axis=-1)` (`../inference/romaji_asr/common.py:134`). The Rust loop only updates on `value > best_value` (`rust/crates/v2m-core/src/asr_romaji_vocab_ctc_decode.rs:165` through `rust/crates/v2m-core/src/asr_romaji_vocab_ctc_decode.rs:171`), and `PartialOrd` comparisons against NaN are false. A targeted Python probe showed `np.argmax(np.array([[1.0, np.nan]], dtype=np.float32), axis=-1).tolist() == [1]`; the Rust loop would keep index `0`. Existing argmax fixtures cover finite first-tie behavior only (`fixtures/asr_romaji_vocab_ctc_decode_core.jsonl:9` and `fixtures/asr_romaji_vocab_ctc_decode_core.jsonl:10`).
- Required fix: Add NaN logits fixtures and implement the observed NumPy selection behavior, or explicitly constrain the Rust API to finite logits before promotion.

## Passing Evidence

- The selected boundary remains narrow. The manifest keeps this unit legacy-owned and excludes audio loading, ONNX sessions, provider selection, and model execution (`manifest.yaml:1847` through `manifest.yaml:1870`).
- `ndarray` usage is appropriate and narrow for shaped 2D/3D numeric arrays. The crate dependency is limited to `ndarray = "0.17"` and `serde_json` with `preserve_order` (`rust/crates/v2m-core/Cargo.toml:15`, `rust/crates/v2m-core/Cargo.toml:17`).
- Duplicate-id overwrite behavior is preserved for JSON vocab loading because `serde_json` is built with `preserve_order`, iteration follows parsed object order, and duplicate ids overwrite in `id2token.insert(...)` (`rust/crates/v2m-core/src/asr_romaji_vocab_ctc_decode.rs:52` through `rust/crates/v2m-core/src/asr_romaji_vocab_ctc_decode.rs:55`). The current duplicate fixture proves a simple last-token case (`fixtures/asr_romaji_vocab_ctc_decode_core.jsonl:5`).
- The core CTC state machine matches the Python reference for fixture-covered signed ids: both initialize `prev = -1`, suppress repeats, skip `blank_id`, reset duplicates after blanks, and use `<unk>` for missing ids (`../inference/romaji_asr/common.py:123` through `../inference/romaji_asr/common.py:131`; `rust/crates/v2m-core/src/asr_romaji_vocab_ctc_decode.rs:66` through `rust/crates/v2m-core/src/asr_romaji_vocab_ctc_decode.rs:83`).
- The `decode_outputs` axis split matches the fixture-covered contract: 2D integer batches iterate over axis 0, and 3D float batches iterate over axis 0 then argmax each 2D item (`../inference/romaji_asr/common.py:138` through `../inference/romaji_asr/common.py:146`; `rust/crates/v2m-core/src/asr_romaji_vocab_ctc_decode.rs:102` through `rust/crates/v2m-core/src/asr_romaji_vocab_ctc_decode.rs:144`).
- Chunking complexity and behavior are appropriate for the fixture-backed JSON projection: `max(1, int(chunk_size))` is preserved for positive, zero, negative, float-truncated, string, and `None` cases (`../inference/romaji_asr/common.py:149` through `../inference/romaji_asr/common.py:152`; `rust/crates/v2m-core/src/asr_romaji_vocab_ctc_decode.rs:146` through `rust/crates/v2m-core/src/asr_romaji_vocab_ctc_decode.rs:156`; `fixtures/asr_romaji_vocab_ctc_decode_core.jsonl:15` through `fixtures/asr_romaji_vocab_ctc_decode_core.jsonl:20`).
- Complexity is linear in the number of tokens/logit cells for the reviewed helpers: vocab inversion is O(vocab size), CTC decode is O(predicted ids), logits argmax is O(time steps * class count), batch decode adds only the leading batch dimension, and chunking is O(items). `BTreeMap` lookup adds O(log vocab) per emitted token, which is acceptable at this helper boundary and does not change fixture-observed output ordering because token output follows prediction order.

## Checks

- `cargo test --manifest-path rust/Cargo.toml asr_romaji_vocab_ctc_decode_core`: passed; 1 fixture test passed, 124 filtered out.
- `uv run python rewrite-in-rust/bootstrap/check_asr_romaji_vocab_ctc_decode_core.py`: passed; `asr_romaji_vocab_ctc_decode_core fixtures ok: 20 cases`.
- `uv run python - <<'PY' ...`: targeted Python probe confirmed large `np.uint64` ids remain positive Python `int` values through `decode_outputs`, and `np.argmax` selects a later NaN in rows such as `[1.0, NaN]`.
- `rg -n "serde_json|preserve_order|indexmap|ndarray" rust/Cargo.toml rust/Cargo.lock rust/crates/*/Cargo.toml`: confirmed `v2m-core` enables `serde_json` `preserve_order` and uses `ndarray` only as the numeric array crate for this module.

## Residual Risk

This review did not run a Rust executable for the uncovered `uint64` and NaN counterexamples because the review stance is read-only and no temporary Rust harness was added. The Rust outcomes are derived directly from the reviewed conversion and comparison code. Remaining unproven surfaces include Python arbitrary-size vocab ids beyond `i64`, non-object `load_vocab` error messages outside the fixture set, empty logits rows, and non-contiguous/strided NumPy layout behavior before any future bridge maps Python arrays into Rust.

## Coordinator State Recommendation

Do not mark `asr_romaji_vocab_ctc_decode_core` verified. Keep the unit `reimplemented` and request writer follow-up for the unsigned-id and NaN-argmax parity gaps, or narrow the manifest/bootstrap contract so those inputs are explicitly outside the promoted Rust boundary.
