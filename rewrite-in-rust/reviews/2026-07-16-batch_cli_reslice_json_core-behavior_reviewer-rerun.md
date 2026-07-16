# batch_cli_reslice_json_core - behavior_reviewer

Date: 2026-07-16
Decision: pass

## Findings

No behavior findings remain in the rerun scope.

Previous findings:

- Closed: medium index-handling gap. The fixture now covers negative `save_timestamps_json` chunk indices at rewrite-in-rust/fixtures/batch_cli_reslice_json_core.jsonl:3 and Python-compatible `slice_audio_from_json` index coercion for `"02"` and `2.7` at rewrite-in-rust/fixtures/batch_cli_reslice_json_core.jsonl:5. Rust now uses `python_int` for timestamp chunk indices and re-slice record indices, plus `python_list_index` for negative list indexing, at rewrite-in-rust/rust/crates/v2m-core/src/batch_cli_reslice_json.rs:106, rewrite-in-rust/rust/crates/v2m-core/src/batch_cli_reslice_json.rs:110, rewrite-in-rust/rust/crates/v2m-core/src/batch_cli_reslice_json.rs:231, rewrite-in-rust/rust/crates/v2m-core/src/batch_cli_reslice_json.rs:375, and rewrite-in-rust/rust/crates/v2m-core/src/batch_cli_reslice_json.rs:387.
- Closed: low array/object lab sidecar text gap. The fixture now covers array and object `text` sidecars at rewrite-in-rust/fixtures/batch_cli_reslice_json_core.jsonl:5. Rust now renders arrays/objects with Python-style repr spacing and quoted string items through `python_str`/`python_repr` at rewrite-in-rust/rust/crates/v2m-core/src/batch_cli_reslice_json.rs:438 and rewrite-in-rust/rust/crates/v2m-core/src/batch_cli_reslice_json.rs:459. `serde_json` is configured with `preserve_order` at rewrite-in-rust/rust/crates/v2m-core/Cargo.toml:15, preserving JSON object insertion order for this representation.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_batch_cli_reslice_json_core.py`: passed.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml batch_cli_reslice_json`: passed.
- `rg -n "librosa|sf\\.write|load_audio|run_slicer|process_one_file|batch_transcribe_asr|load_qwen_model|RmvpeTranscriber|clear_qwen_model_cache" rewrite-in-rust/bootstrap/check_batch_cli_reslice_json_core.py rewrite-in-rust/rust/crates/v2m-core/src/batch_cli_reslice_json.rs`: only the checker monkeypatches `load_audio` and `cli.sf.write`; no Rust runtime call path and no excluded ASR/RMVPE/slicer/model calls were found.
- Focused Python probe for the previous index and lab findings: accepted `"02"` and `2.7` indices, wrote list/object lab contents as `"[1, 'x']"` and `"{'a': 1}"`, and wrote `save_timestamps_json` output with `"index": -1` using the last chunk.

## Residual Risk

The unit remains fixture-bound and intentionally does not prove real audio decoding, resampling, SoundFile/libsndfile encoding, ASR/RMVPE/slicer/model execution, full CLI parser behavior, or production routing. The Python checker still imports `scripts.slice_asr_cli`, so module-import dependencies must exist even though real audio/model effects are monkeypatched or excluded.

The Python and Rust harnesses still use subset assertions for expected objects. In the inspected scope, the fixtures now cover the previously missing behavior paths and the returned shapes include write plans, labs, stdout lines, JSON payload strings, and error summaries relevant to the manifest policy.

## Promotion Note

This rerun is behavior-parity evidence for the manifest's `stage_behavior_reviewer` requirement while keeping the report role name `behavior_reviewer`. The behavior-review role no longer blocks coordinator state update for this unit. I did not edit production code and did not mark the manifest verified.
