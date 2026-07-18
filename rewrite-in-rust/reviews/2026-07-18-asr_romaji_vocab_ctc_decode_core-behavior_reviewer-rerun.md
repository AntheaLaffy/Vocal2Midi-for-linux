# asr_romaji_vocab_ctc_decode_core - behavior_reviewer rerun

PASS

Date: 2026-07-18
Decision: pass
Role: behavior_reviewer
Unit: asr_romaji_vocab_ctc_decode_core

## Findings

No blocking behavior parity findings.

## Evidence

- The manifest keeps `asr_romaji_vocab_ctc_decode_core` at `status: reimplemented`, with `current_owner: legacy`, required `stage_behavior_reviewer` and `data_algorithm_reviewer` gates, and an explicit rollback route to Python-owned `inference.romaji_asr.common` helpers (`manifest.yaml:1847`, `manifest.yaml:1849`, `manifest.yaml:1851`, `manifest.yaml:1856`, `manifest.yaml:1871`).
- The dependency record confirms the narrow helper seam, `bridge_dependencies: []`, 24 golden fixtures, and the updated tie-first/NaN argmax wording for selected CTC decode behavior (`dependencies/asr_romaji_vocab_ctc_decode_core.yaml:8`, `dependencies/asr_romaji_vocab_ctc_decode_core.yaml:16`, `dependencies/asr_romaji_vocab_ctc_decode_core.yaml:20`).
- The bootstrap record scopes the public boundary to `load_vocab`, `decode_pred_ids`, `decode_logits`, `decode_outputs`, and `chunked`, while excluding `load_audio`, `create_session`, ONNX providers/session metadata, audio IO, resampling, padding/mask preparation, and model execution (`bootstrap/asr_romaji_vocab_ctc_decode_core.md:9`, `bootstrap/asr_romaji_vocab_ctc_decode_core.md:20`).
- Legacy Python behavior remains the selected compatibility source: `load_vocab` uses `json.load`, `int(...)` id inversion, duplicate-id last-write behavior, and `<blank>`/`PAD`/`0` blank fallback (`../inference/romaji_asr/common.py:15`); `decode_pred_ids` performs CTC duplicate collapse, blank suppression, blank reset, and `<unk>` fallback (`../inference/romaji_asr/common.py:123`); `decode_logits` delegates to `np.argmax(..., axis=-1)` (`../inference/romaji_asr/common.py:134`); `decode_outputs` dispatches per batch item on `np.issubdtype(item.dtype, np.integer)` (`../inference/romaji_asr/common.py:138`); `chunked` uses `max(1, int(chunk_size))` (`../inference/romaji_asr/common.py:149`).
- Rust now represents token ids as `i128`, uses that representation for vocab maps, blank ids, CTC state, and unsigned output paths, and maps `u64` ids into `TokenId` without the earlier `i64` wrap (`rust/crates/v2m-core/src/asr_romaji_vocab_ctc_decode.rs:13`, `rust/crates/v2m-core/src/asr_romaji_vocab_ctc_decode.rs:122`).
- Rust argmax now updates when a later NaN appears before any earlier NaN, and does not replace an existing NaN winner, matching the fixture-proven NumPy behavior for this public seam (`rust/crates/v2m-core/src/asr_romaji_vocab_ctc_decode.rs:175`, `rust/crates/v2m-core/src/asr_romaji_vocab_ctc_decode.rs:209`).
- The fixture file has 24 cases. The four cases added after the failed data review cover the prior behavior risks directly: large `uint64` ids above `i64::MAX` at line 21, direct NaN argmax at lines 22-23, and batched float-logits NaN dispatch at line 24 (`fixtures/asr_romaji_vocab_ctc_decode_core.jsonl:21`, `fixtures/asr_romaji_vocab_ctc_decode_core.jsonl:22`, `fixtures/asr_romaji_vocab_ctc_decode_core.jsonl:23`, `fixtures/asr_romaji_vocab_ctc_decode_core.jsonl:24`).
- The Python fixture checker explicitly parses `nan`/`inf` string sentinels before constructing NumPy arrays, so the JSONL cases exercise current Python `np.argmax` behavior rather than JSON parser behavior (`bootstrap/check_asr_romaji_vocab_ctc_decode_core.py:33`, `bootstrap/check_asr_romaji_vocab_ctc_decode_core.py:62`, `bootstrap/check_asr_romaji_vocab_ctc_decode_core.py:66`).
- The runtime and CLI still import and call Python `common.py` helpers directly for model workflow behavior: `RomajiASROnnxModel.from_model_path` calls Python `load_vocab`, `transcribe_batch` calls Python `decode_outputs`, and batching uses Python `chunked` (`../inference/romaji_asr/runtime.py:6`, `../inference/romaji_asr/runtime.py:84`, `../inference/romaji_asr/runtime.py:104`, `../inference/romaji_asr/runtime.py:124`; `../inference/romaji_asr/infer_dml.py:5`, `../inference/romaji_asr/infer_dml.py:46`). No production bridge or runtime owner switch was found in the reviewed scope.
- `serde_json` is built with `preserve_order`, preserving the fixture-relevant duplicate-id overwrite order for vocab JSON inversion (`rust/crates/v2m-core/Cargo.toml:17`; `rust/crates/v2m-core/src/asr_romaji_vocab_ctc_decode.rs:55`).

## Checks

- `cargo test --manifest-path rust/Cargo.toml asr_romaji_vocab_ctc_decode_core` from `/home/fuurin/code/Vocal2Midi-for-linux/rewrite-in-rust`: passed. One focused `v2m-core` fixture test passed, 124 tests filtered out; bridge crate had 0 matching tests, 5 filtered out.
- `uv run python rewrite-in-rust/bootstrap/check_asr_romaji_vocab_ctc_decode_core.py` from `/home/fuurin/code/Vocal2Midi-for-linux`: passed with `asr_romaji_vocab_ctc_decode_core fixtures ok: 24 cases`.
- Targeted fixture inspection: confirmed 24 JSONL rows and the added `decode_outputs_uint64_large_ids`, `decode_logits_nan_later_wins`, `decode_logits_nan_first_stays`, and `decode_outputs_float_logits_nan_batch` cases.
- Targeted Python probe from `/home/fuurin/code/Vocal2Midi-for-linux`: confirmed legacy Python returns `[['big', 'max', 'big']]` for the large `np.uint64` sequence, `['a']` for `[0.0, NaN, 1.0]`, and `[]` for `[NaN, 0.0, 1.0]`.

## Residual Risk

This behavior rerun reviewed Python/Rust parity at the selected helper seam only. It did not review dependency strategy, Rust style, error/tracing, architecture, or the full data/algorithm role. ONNX Runtime sessions, audio loading, resampling, batch padding/mask preparation, model execution, and production bridge behavior remain intentionally outside this unit.

Remaining unproven behavior surfaces include non-contiguous Python array bridge layout, empty-logit-axis panic/error mapping before any Python bridge exists, Python integers beyond Rust `i128`, and exact invalid JSON/vocab error text outside fixture-covered projections.

## Coordinator State Recommendation

Record the behavior rerun as passed. Keep `current_owner: legacy` and keep the unit at `reimplemented` unless and until the required independent `data_algorithm_reviewer` rerun also passes; the previous data/algorithm report was a fail and should not be treated as superseded by this behavior review alone.
