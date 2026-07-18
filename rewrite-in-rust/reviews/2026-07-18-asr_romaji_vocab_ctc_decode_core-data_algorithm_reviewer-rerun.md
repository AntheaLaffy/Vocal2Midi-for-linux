# asr_romaji_vocab_ctc_decode_core - data_algorithm_reviewer rerun

PASS

Date: 2026-07-18
Decision: pass
Role: data_algorithm_reviewer
Unit: asr_romaji_vocab_ctc_decode_core

## Findings

No blocking findings.

## Evidence

- Prior blocker fixed: large unsigned token ids no longer wrap through a signed
  `i64` path. Rust now defines `TokenId = i128` for vocab maps, blank ids, CTC
  state, and integer decode paths, and `decode_outputs_uint` casts `u64` values
  directly into `TokenId` (`rust/crates/v2m-core/src/asr_romaji_vocab_ctc_decode.rs:13`,
  `rust/crates/v2m-core/src/asr_romaji_vocab_ctc_decode.rs:122`,
  `rust/crates/v2m-core/src/asr_romaji_vocab_ctc_decode.rs:131`). The new
  fixture `decode_outputs_uint64_large_ids` covers ids
  `9223372036854775808` and `18446744073709551615` above `i64::MAX`
  (`fixtures/asr_romaji_vocab_ctc_decode_core.jsonl:21`).

- Prior blocker fixed: logits argmax now matches the observed NumPy NaN winner
  behavior for the reviewed seam. Python still delegates to
  `np.argmax(logits, axis=-1)` (`../inference/romaji_asr/common.py:134`), and
  Rust updates from a finite best value to the first NaN, then keeps that NaN
  winner (`rust/crates/v2m-core/src/asr_romaji_vocab_ctc_decode.rs:175`,
  `rust/crates/v2m-core/src/asr_romaji_vocab_ctc_decode.rs:209`). Fixtures now
  cover later-NaN, first-NaN, and batched float-logits NaN cases
  (`fixtures/asr_romaji_vocab_ctc_decode_core.jsonl:22`,
  `fixtures/asr_romaji_vocab_ctc_decode_core.jsonl:23`,
  `fixtures/asr_romaji_vocab_ctc_decode_core.jsonl:24`).

- CTC state remains algorithmically aligned with Python for signed, unsigned,
  large-id, blank, duplicate, and unknown-token cases. Python initializes
  `prev = -1`, converts each token through `int(...)`, suppresses repeats, skips
  the configured blank id, and emits `<unk>` for missing ids
  (`../inference/romaji_asr/common.py:123`). Rust mirrors that state machine
  with `TokenId` and `BTreeMap` lookup
  (`rust/crates/v2m-core/src/asr_romaji_vocab_ctc_decode.rs:69`).

- Vocab inversion and duplicate-id overwrite are appropriate for this boundary.
  The Rust crate enables `serde_json` `preserve_order`
  (`rust/crates/v2m-core/Cargo.toml:17`), then inserts parsed id/token pairs into
  a `BTreeMap`, so later duplicate ids overwrite earlier values while lookup is
  deterministic (`rust/crates/v2m-core/src/asr_romaji_vocab_ctc_decode.rs:55`).
  The fixture suite keeps the duplicate-id overwrite case
  (`fixtures/asr_romaji_vocab_ctc_decode_core.jsonl:5`).

- ndarray axis handling and dtype split match the fixture-backed seam. Python
  `decode_outputs` iterates leading batch axis, dispatches integer arrays to
  `decode_pred_ids`, and otherwise decodes each 2D logits item
  (`../inference/romaji_asr/common.py:138`). Rust uses `Axis(0)` for 2D integer
  batches and 3D logits batches, with separate signed, unsigned, f32, and f64
  entry points (`rust/crates/v2m-core/src/asr_romaji_vocab_ctc_decode.rs:105`,
  `rust/crates/v2m-core/src/asr_romaji_vocab_ctc_decode.rs:122`,
  `rust/crates/v2m-core/src/asr_romaji_vocab_ctc_decode.rs:139`,
  `rust/crates/v2m-core/src/asr_romaji_vocab_ctc_decode.rs:150`).

- Chunking preserves the selected `max(1, int(chunk_size))` behavior for the
  JSON-compatible fixture projection. Python implements that at
  `../inference/romaji_asr/common.py:149`; Rust implements typed chunking plus
  JSON fixture coercion at
  `rust/crates/v2m-core/src/asr_romaji_vocab_ctc_decode.rs:161`.

- The latest dependency/bootstrap wording is consistent with the implementation
  and fixture scope. The dependency record now names NumPy argmax
  tie-first/NaN behavior as selected seam behavior
  (`dependencies/asr_romaji_vocab_ctc_decode_core.yaml:11`) and says the fixture
  file contains 24 golden cases including argmax tie-first and NaN behavior plus
  uint64 large-id preservation
  (`dependencies/asr_romaji_vocab_ctc_decode_core.yaml:22`). The bootstrap
  record likewise names first-index tie and NumPy NaN behavior in the Rust
  boundary (`bootstrap/asr_romaji_vocab_ctc_decode_core.md:51`) and lists both
  NaN argmax and `uint64` ids above `i64::MAX` in the fixture harness
  (`bootstrap/asr_romaji_vocab_ctc_decode_core.md:68`).

- Fixture strength is sufficient for this data/algorithm rerun. The file has 24
  cases: 5 `load_vocab`, 3 `decode_pred_ids`, 4 `decode_logits`, 6
  `decode_outputs`, and 6 `chunked` cases. It covers the two prior blockers
  directly, plus blank precedence/defaults, duplicate overwrite, CTC blank reset,
  unknown fallback, finite tie-first argmax, integer-vs-logit dispatch, f64
  logits, batch iteration, and chunk-size coercion.

- Complexity remains appropriate for the helper boundary: vocab inversion is
  linear in vocab entries, CTC decode is linear in predicted ids, logits argmax
  is linear in rows times classes, batch decode adds the leading batch dimension,
  and chunking is linear in item count. `BTreeMap` lookup adds O(log vocab) per
  emitted token, acceptable for this deterministic helper unit.

## Checks

- `cargo test --manifest-path rust/Cargo.toml asr_romaji_vocab_ctc_decode_core`:
  passed. One focused fixture test passed in `v2m-core`; 124 tests were
  filtered out. The bridge crate had 0 matching tests.
- `uv run python rewrite-in-rust/bootstrap/check_asr_romaji_vocab_ctc_decode_core.py`:
  passed with `asr_romaji_vocab_ctc_decode_core fixtures ok: 24 cases`.
- `uv run python - <<'PY' ...`: targeted Python probe confirmed
  `decode_outputs` preserves large `np.uint64` ids as positive Python `int`
  lookups and that `np.argmax` returns `[1, 0, 2, 1]` for later-NaN,
  first-NaN, trailing-NaN, and finite-tie rows.
- `uv run python - <<'PY' ...`: fixture inspection confirmed 24 cases and the
  category/call/dtype distribution described above.
- `rg -n "serde_json|preserve_order|ndarray" rust/Cargo.toml rust/crates/v2m-core/Cargo.toml rust/Cargo.lock`:
  confirmed `v2m-core` uses `ndarray = "0.17"` and enables `serde_json`
  `preserve_order`.

## Residual Risk

This review did not prove Python arbitrary-precision token ids outside the
signed `i64` plus unsigned `u64` fixture boundary, empty logits rows, ragged
arrays, or future Python bridge behavior for non-contiguous NumPy arrays. Those
remain outside the current fixture-backed Rust helper API and should be
revisited if a promotion unit exposes this code directly to Python arrays.

The fixture suite has f32 NaN cases and f64 finite-logit coverage; it does not
have a dedicated f64 NaN fixture. The f64 implementation uses the same update
rule as f32, so this is not a blocker for this rerun.

## Coordinator State Recommendation

This data/algorithm rerun does not block promotion. Record the
`data_algorithm_reviewer` gate as passed for
`asr_romaji_vocab_ctc_decode_core`, keep `current_owner: legacy`, and let the
coordinator decide whether the existing behavior gate plus this rerun are enough
to move the unit from `reimplemented` to `verified`.
