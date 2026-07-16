# batch_cli_planning_and_index_core - product_ergonomics_reviewer rerun

Date: 2026-07-16
Decision: pass-with-followups

## Findings

- Severity: low
- Location: rewrite-in-rust/rust/crates/v2m-core/src/batch_cli_planning.rs:485
- Issue: The previous medium source-path finding is closed for `update_source_index`, but the fake batch-loop planner still supplies fixture-normalized `__case__/...` paths internally. This is acceptable for the current fixture-only seam, but it must not become production CLI behavior.
- Evidence: Legacy Python stores `source_path` from `audio_path.resolve()` in `scripts/slice_asr_cli.py:298`. The Rust helper now accepts `resolved_source_path` as a caller argument and stores it directly in `rewrite-in-rust/rust/crates/v2m-core/src/batch_cli_planning.rs:268` and `rewrite-in-rust/rust/crates/v2m-core/src/batch_cli_planning.rs:286`, closing the previous hard-coded-helper issue. The test harness and fake planner still pass `format!("__case__/{audio_path}")` at `rewrite-in-rust/rust/crates/v2m-core/src/batch_cli_planning.rs:857` and `rewrite-in-rust/rust/crates/v2m-core/src/batch_cli_planning.rs:485`. The bootstrap keeps the seam non-production and bridge-free in `rewrite-in-rust/bootstrap/batch_cli_planning_and_index_core.md:79`.
- Required fix: No fix is required before fixture-level promotion. Before any production CLI bridge routes batch planning through Rust, thread the actual resolved source path through the planner instead of using fixture placeholders.

- Severity: low
- Location: rewrite-in-rust/fixtures/batch_cli_planning_and_index_core.jsonl:31
- Issue: Batch-loop fixtures still verify counters, skipped records, and index state rather than the user-visible progress and recovery messages printed by the CLI.
- Evidence: Python prints the no-audio message in `scripts/slice_asr_cli.py:694`, file-batch progress in `scripts/slice_asr_cli.py:725`, MD5 failure details in `scripts/slice_asr_cli.py:733`, skip-existing messages in `scripts/slice_asr_cli.py:742`, process failure details in `scripts/slice_asr_cli.py:770`, and the final summary in `scripts/slice_asr_cli.py:789`. The Rust batch result is structured state only in `rewrite-in-rust/rust/crates/v2m-core/src/batch_cli_planning.rs:505`.
- Required fix: Keep Python as the message renderer, or add a separate CLI-message/event fixture before routing user-visible batch progress through Rust.

- Severity: low
- Location: rewrite-in-rust/fixtures/batch_cli_planning_and_index_core.jsonl:2
- Issue: CLI flag-specific batch-size validation messages remain outside this unit.
- Evidence: Legacy CLI validation reports `--file-batch-size must be greater than 0`, `--asr-batch-size must be greater than 0`, and `--rmvpe-batch-size must be greater than 0` in `scripts/slice_asr_cli.py:656`. This unit verifies the helper-level `batch_size must be greater than 0` behavior in `rewrite-in-rust/fixtures/batch_cli_planning_and_index_core.jsonl:2`, matching `rewrite-in-rust/rust/crates/v2m-core/src/batch_cli_planning.rs:117`. The bootstrap excludes full parser/validation ownership in `rewrite-in-rust/bootstrap/batch_cli_planning_and_index_core.md:50`.
- Required fix: Keep flag-specific validation legacy-owned, or add a separate CLI validation fixture before Rust owns argument validation.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_batch_cli_planning_and_index_core.py`: passed.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml batch_cli_planning`: passed.
- `rg -n "load_audio|run_slicer|process_one_file|batch_transcribe|load_qwen|RmvpeTranscriber|clear_qwen_model_cache|slice_audio_from_json" rewrite-in-rust/bootstrap/check_batch_cli_planning_and_index_core.py rewrite-in-rust/rust/crates/v2m-core/src/batch_cli_planning.rs rewrite-in-rust/fixtures/batch_cli_planning_and_index_core.jsonl rewrite-in-rust/bootstrap/batch_cli_planning_and_index_core.md`: no excluded runtime calls in the fixture, checker, or Rust module; matches only bootstrap boundary text.

## Residual Risk

This rerun did not execute the real CLI, decode audio, write waveform chunks, load ASR/RMVPE runtimes, or verify stdout parity. That matches the confirmed unit boundary. The main source-index recovery risk from the prior review is closed for the reusable helper, while production bridge risk remains if a future caller reuses fixture-only path placeholders or bypasses Python message rendering.

## Previous Medium Finding

Closed. `update_source_index` no longer hard-codes `__case__/...`; it takes `resolved_source_path` from the caller and stores that value. The remaining `__case__` usage is in fixture/test plumbing and the fake batch planner, not in the source-index update primitive.

## Promotion Note

Writer/reviewer separation was preserved: this rerun edited only this report and did not change production code or the manifest. This role does not block coordinator state update for the current fixture-only unit, but it does not approve a production CLI bridge and does not mark the manifest verified.
