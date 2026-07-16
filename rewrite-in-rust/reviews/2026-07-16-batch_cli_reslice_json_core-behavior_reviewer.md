# batch_cli_reslice_json_core - behavior_reviewer

Date: 2026-07-16
Decision: pass-with-followups

## Findings

- Severity: medium
- Location: scripts/slice_asr_cli.py:374, scripts/slice_asr_cli.py:319, rewrite-in-rust/rust/crates/v2m-core/src/batch_cli_reslice_json.rs:229, rewrite-in-rust/rust/crates/v2m-core/src/batch_cli_reslice_json.rs:106, rewrite-in-rust/rust/crates/v2m-core/src/batch_cli_reslice_json.rs:111, rewrite-in-rust/rust/crates/v2m-core/src/batch_cli_reslice_json.rs:341
- Issue: Index handling is narrower than Python for accepted helper inputs. `slice_audio_from_json` uses `int(record["index"])`, so JSON values such as `"7"` and `8.9` are accepted and formatted as chunk indices. `save_timestamps_json` also accepts Python negative `chunk_indices` because list indexing is delegated to `chunks[chunk_index]`. The Rust model requires JSON integers with `as_i64()` and casts timestamp chunk indices to `usize`, so numeric strings/floats are rejected and negative chunk indices become out-of-range instead of selecting from the end.
- Evidence: A focused Python probe accepted `{"index": "7"}` and `{"index": 8.9}` in `slice_audio_from_json`, producing `written: 2` with writes `song_name_chunk0007...wav` and `song_name_chunk0008...wav`. A second probe showed `save_timestamps_json(..., chunk_indices=[-1])` writes a record with `"index": -1` using the last chunk. The durable fixture only covers normal integer indices, one positive out-of-range timestamp index, and a missing record index at rewrite-in-rust/fixtures/batch_cli_reslice_json_core.jsonl:2, rewrite-in-rust/fixtures/batch_cli_reslice_json_core.jsonl:3, rewrite-in-rust/fixtures/batch_cli_reslice_json_core.jsonl:4, and rewrite-in-rust/fixtures/batch_cli_reslice_json_core.jsonl:6.
- Required fix: Add fixture rows for Python-compatible `int()` coercion in re-slice records and negative timestamp `chunk_indices`, then either implement the matching behavior in Rust or explicitly narrow the boundary with a record/manifest note.

- Severity: low
- Location: scripts/slice_asr_cli.py:377, scripts/slice_asr_cli.py:391, rewrite-in-rust/rust/crates/v2m-core/src/batch_cli_reslice_json.rs:256, rewrite-in-rust/rust/crates/v2m-core/src/batch_cli_reslice_json.rs:397
- Issue: Lab sidecar content diverges for JSON object/list `text` values. Python writes `str(record.get("text", ""))`, which uses Python repr-like formatting for lists and dicts. Rust's `python_str` serializes arrays/objects as compact JSON. Existing fixtures cover `None`, `False`, `0`, whitespace, missing text, and empty text, but not object/list text values.
- Evidence: A focused Python probe wrote lab contents `"{'nested': 1}"` and `"[1, 2]"` for object/list `text` values. The Rust model would produce `{"nested":1}` and `[1,2]` from `Value::to_string()`. The current lab fixture coverage is concentrated in rewrite-in-rust/fixtures/batch_cli_reslice_json_core.jsonl:4.
- Required fix: Add object/list text fixtures for `slice_audio_from_json` and either mirror Python `str()` formatting for those JSON values or document that non-scalar lab text is outside the Rust unit's accepted input surface.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_batch_cli_reslice_json_core.py`: passed
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml batch_cli_reslice_json`: passed
- `rg -n "librosa|sf\\.write|load_audio|run_slicer|process_one_file|batch_transcribe_asr|load_qwen_model|RmvpeTranscriber|clear_qwen_model_cache" rewrite-in-rust/bootstrap/check_batch_cli_reslice_json_core.py rewrite-in-rust/rust/crates/v2m-core/src/batch_cli_reslice_json.rs`: only the checker monkeypatches `load_audio` and `cli.sf.write`; no Rust runtime call path and no excluded ASR/RMVPE/slicer/model calls were found.
- Focused Python probe for re-slice numeric-string/float indices and object/list lab text: Python accepted the indices and wrote Python `str()` lab content.
- Focused Python probe for negative `save_timestamps_json` chunk index: Python selected the last chunk and wrote `"index": -1`.

## Residual Risk

The main fixture table is useful but not exhaustive because both Python and Rust harnesses assert expected subsets. Rounding, ordering, source metadata, missing file errors, empty chunks/audio, invalid range skipping, scalar lab sidecars, and save-chunks write plans are covered by the current rows. Remaining risk is concentrated in Python's broad dynamic coercion behavior for hand-authored JSON values and less-common `chunk_indices` inputs.

The checker imports `scripts.slice_asr_cli`, so module-import dependencies still need to be available even though the test monkeypatches real audio load/write effects and does not call model/audio runtimes.

## Promotion Note

This report is behavior-parity evidence for the manifest's `stage_behavior_reviewer` requirement while keeping the report role name `behavior_reviewer`. The unit should not be promoted solely on this report until the medium index-handling gap is fixed or explicitly accepted as out of scope by the coordinator. I did not edit production code and did not mark the manifest verified.
