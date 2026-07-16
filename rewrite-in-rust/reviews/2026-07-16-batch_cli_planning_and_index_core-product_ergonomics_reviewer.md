# batch_cli_planning_and_index_core - product_ergonomics_reviewer

Date: 2026-07-16
Decision: pass-with-followups

## Findings

- Severity: medium
- Location: rewrite-in-rust/rust/crates/v2m-core/src/batch_cli_planning.rs:285
- Issue: The Rust source-index writer stores fixture-normalized `__case__/...` paths instead of the resolved source path that the Python CLI records for recovery and operator inspection.
- Evidence: Legacy `update_source_index` writes `source_path` from `audio_path.resolve()` in `scripts/slice_asr_cli.py:298`. The Rust implementation writes `format!("__case__/{audio_path}")` in `rewrite-in-rust/rust/crates/v2m-core/src/batch_cli_planning.rs:285` and the JSON renderer repeats the same placeholder at `rewrite-in-rust/rust/crates/v2m-core/src/batch_cli_planning.rs:303`. The fixture expects the normalized placeholder in `rewrite-in-rust/fixtures/batch_cli_planning_and_index_core.jsonl:22`, while the checker normalizes real temp paths to `__case__` in `rewrite-in-rust/bootstrap/check_batch_cli_planning_and_index_core.py:52`.
- Required fix: Before any production bridge or CLI route uses the Rust source-index helper, pass/store the actual resolved source path and keep `__case__` normalization only in test comparison code.

- Severity: low
- Location: rewrite-in-rust/fixtures/batch_cli_planning_and_index_core.jsonl:27
- Issue: The batch-loop fixtures verify counters, skipped records, and source-index state but do not model the user-visible progress/recovery messages that the CLI prints while scanning, batching, skipping, failing, and finishing.
- Evidence: Python prints the no-audio message in `scripts/slice_asr_cli.py:694`, batch banners in `scripts/slice_asr_cli.py:725`, MD5 failure details in `scripts/slice_asr_cli.py:733`, skip-existing messages in `scripts/slice_asr_cli.py:742`, process failure details in `scripts/slice_asr_cli.py:770`, and the final summary in `scripts/slice_asr_cli.py:789`. The Rust result model returns data only in `rewrite-in-rust/rust/crates/v2m-core/src/batch_cli_planning.rs:491`.
- Required fix: Keep Python as the message renderer, or add a later CLI-message/event fixture before routing user-visible batch progress through Rust.

- Severity: low
- Location: rewrite-in-rust/fixtures/batch_cli_planning_and_index_core.jsonl:2
- Issue: The fixture covers the helper-level `batch_iter` error, but not the CLI flag-specific batch-size validation messages users see from `validate_args`.
- Evidence: Legacy CLI validation reports `--file-batch-size must be greater than 0`, `--asr-batch-size must be greater than 0`, and `--rmvpe-batch-size must be greater than 0` in `scripts/slice_asr_cli.py:656`. The fixture asserts only `batch_size must be greater than 0`, and the Rust helper returns that same generic message in `rewrite-in-rust/rust/crates/v2m-core/src/batch_cli_planning.rs:117`.
- Required fix: Keep those validations legacy-owned, or add a separate CLI validation fixture before Rust owns user-facing argument validation.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_batch_cli_planning_and_index_core.py`: passed.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml batch_cli_planning`: passed.
- `rg -n "load_audio|run_slicer|process_one_file|batch_transcribe|load_qwen|RmvpeTranscriber|clear_qwen_model_cache|slice_audio_from_json" rewrite-in-rust/bootstrap/check_batch_cli_planning_and_index_core.py rewrite-in-rust/rust/crates/v2m-core/src/batch_cli_planning.rs rewrite-in-rust/fixtures/batch_cli_planning_and_index_core.jsonl rewrite-in-rust/bootstrap/batch_cli_planning_and_index_core.md`: no excluded runtime calls in the fixture, checker, or Rust module; matches only boundary text in the bootstrap note.

## Residual Risk

This review did not run the real batch CLI, decode audio, write chunks, load ASR/RMVPE runtimes, or verify full stdout parity. That is intentional for this unit boundary. The product risk is limited while `scripts/slice_asr_cli.py` remains the runtime owner, but the followups above must be resolved before a user-facing Rust bridge can preserve source-index recovery and CLI message ergonomics.

## Promotion Note

Writer/reviewer separation was preserved: this review edited only this report and did not change production code or the manifest. The unit can be used as fixture-level promotion evidence with followups, but this report does not mark the manifest verified and does not approve a production CLI bridge.
