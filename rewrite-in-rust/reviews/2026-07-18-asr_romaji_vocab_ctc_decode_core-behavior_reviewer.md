# asr_romaji_vocab_ctc_decode_core - behavior_reviewer

Date: 2026-07-18
Decision: PASS

## Findings

No blocking behavior parity findings.

Evidence:

- `manifest.yaml:1847` keeps the unit at `reimplemented`, `current_owner: legacy`, and records rollback to Python-owned `inference.romaji_asr.common` helpers at `manifest.yaml:1870`.
- `bootstrap/asr_romaji_vocab_ctc_decode_core.md:9` scopes the public boundary to `load_vocab`, `decode_pred_ids`, `decode_logits`, `decode_outputs`, and `chunked`, while excluding audio loading, ONNX session creation/provider selection, session metadata, resampling, padding, and model execution at `bootstrap/asr_romaji_vocab_ctc_decode_core.md:20`.
- `dependencies/asr_romaji_vocab_ctc_decode_core.yaml:4` through `dependencies/asr_romaji_vocab_ctc_decode_core.yaml:15` describe the selected behavior as vocab inversion, CTC/list decode helpers, dtype-shaped output decoding, and chunked iteration, with legacy ownership and no bridge dependencies at `dependencies/asr_romaji_vocab_ctc_decode_core.yaml:16`.
- Legacy `load_vocab` performs Python `json.load`, id inversion via `int(v)`, duplicate-id last write, and `<blank>` then `PAD` then `0` blank fallback at `../inference/romaji_asr/common.py:15`.
- Rust `load_vocab_from_json_str` mirrors that by parsing JSON, iterating the JSON object in preserved order, inserting `python_int(id_value)` into `Id2Token`, and selecting `<blank>` then `PAD` then `0` at `rust/crates/v2m-core/src/asr_romaji_vocab_ctc_decode.rs:45`. `serde_json` is built with `preserve_order` at `rust/crates/v2m-core/Cargo.toml:17`, which is required for Python duplicate-id overwrite parity.
- Legacy CTC collapse skips blanks, suppresses adjacent duplicates, resets duplicates after blank, and falls back to `<unk>` at `../inference/romaji_asr/common.py:123`. Rust `decode_pred_ids` matches the same `prev = -1` and blank/token logic at `rust/crates/v2m-core/src/asr_romaji_vocab_ctc_decode.rs:66`.
- Legacy `decode_logits` delegates to `np.argmax(..., axis=-1)` at `../inference/romaji_asr/common.py:134`. Rust `argmax_axis_last` updates only on `value > best_value`, preserving first-index tie behavior for the covered finite numeric cases at `rust/crates/v2m-core/src/asr_romaji_vocab_ctc_decode.rs:158`.
- Legacy `decode_outputs` iterates batch dimension 0 and dispatches per item on `np.issubdtype(item.dtype, np.integer)` at `../inference/romaji_asr/common.py:138`. Rust provides integer batch helpers at `rust/crates/v2m-core/src/asr_romaji_vocab_ctc_decode.rs:102` and logit batch helpers at `rust/crates/v2m-core/src/asr_romaji_vocab_ctc_decode.rs:124`; the fixture adapter exercises integer-vs-logit dispatch from fixture dtype.
- Legacy `chunked` uses `max(1, int(chunk_size))` and list slicing at `../inference/romaji_asr/common.py:149`. Rust `chunked` preserves the integer step behavior at `rust/crates/v2m-core/src/asr_romaji_vocab_ctc_decode.rs:146`, while `chunked_json_values` preserves fixture-facing Python `int(...)` coercion and errors at `rust/crates/v2m-core/src/asr_romaji_vocab_ctc_decode.rs:151`.
- `fixtures/asr_romaji_vocab_ctc_decode_core.jsonl:1` through `fixtures/asr_romaji_vocab_ctc_decode_core.jsonl:20` cover the requested parity points: vocab id inversion and blank fallback, duplicate-id overwrite, blank reset and duplicate collapse, unknown fallback, logits tie-first argmax, integer and float output batch iteration, unsigned integer dispatch, float64 logits, and chunk sizes including zero, negative, float, numeric string, and invalid `None`.
- Records `0117` and `0118` confirm the bootstrap boundary and implementation state without adding runtime wiring or promotion.

## Checks

- `cargo test --manifest-path rust/Cargo.toml asr_romaji_vocab_ctc_decode_core` from `rewrite-in-rust/`: passed; 1 focused fixture test passed, 124 filtered out in `v2m-core`, and the bridge crate had 0 matching tests.
- `uv run python rewrite-in-rust/bootstrap/check_asr_romaji_vocab_ctc_decode_core.py` from `/home/fuurin/code/Vocal2Midi-for-linux`: passed; `asr_romaji_vocab_ctc_decode_core fixtures ok: 20 cases`.
- `uv run python -m py_compile inference/romaji_asr/common.py` from `/home/fuurin/code/Vocal2Midi-for-linux`: passed with no output.
- Targeted fixture/source inspection: confirmed the fixture file has 20 cases and the Rust test includes the same JSONL fixture.

## Residual Risk

This review did not redo dependency/bootstrap review, ONNX/audio/model execution behavior, session metadata behavior, or data/algorithm review. Invalid vocab JSON/object-shape errors, NaN argmax behavior, empty-logit-axis errors, and Python arbitrary-precision integer edge cases are not fixture-covered; they appear outside the realistic model-vocab/logit path for this unit but remain unproven.

## Promotion Note

This behavior review does not block promotion. Coordinator recommendation: record the behavior gate as passed for `asr_romaji_vocab_ctc_decode_core`, keep `current_owner: legacy`, and do not mark the unit `verified` until the required `data_algorithm_reviewer` gate also passes.
