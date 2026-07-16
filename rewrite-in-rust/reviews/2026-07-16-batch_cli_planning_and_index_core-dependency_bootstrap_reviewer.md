# batch_cli_planning_and_index_core - dependency_bootstrap_reviewer

Date: 2026-07-16
Decision: pass-with-followups

## Findings

- Severity: medium
- Location: rewrite-in-rust/dependencies/batch_cli_planning_and_index_core.yaml:43
- Issue: The dependency record requires completed-output coverage through JSON, lab, and slice outputs, but the durable fixtures only prove the md5-index path through `jsons/<key>.json`.
- Evidence: `rewrite-in-rust/fixtures/batch_cli_planning_and_index_core.jsonl:23` covers an indexed JSON hit; `rewrite-in-rust/fixtures/batch_cli_planning_and_index_core.jsonl:24` and `rewrite-in-rust/fixtures/batch_cli_planning_and_index_core.jsonl:25` cover direct output-tree lab/slice hits, not `index_has_completed_output` lab/slice hits. The legacy helper checks indexed JSON, labs, and slices at `scripts/slice_asr_cli.py:275`, `scripts/slice_asr_cli.py:280`, and `scripts/slice_asr_cli.py:282`; the Rust model mirrors those branches at `rewrite-in-rust/rust/crates/v2m-core/src/batch_cli_planning.rs:365`, `rewrite-in-rust/rust/crates/v2m-core/src/batch_cli_planning.rs:371`, and `rewrite-in-rust/rust/crates/v2m-core/src/batch_cli_planning.rs:379`.
- Required fix: Add fixture cases where `source_index` points to completed `output/labs/<key>/*.lab` and `output/slices/<key>/*.wav` outputs, then rerun both Python and Rust fixture checks.

- Severity: low
- Location: rewrite-in-rust/bootstrap/check_batch_cli_planning_and_index_core.py:17
- Issue: The Python checker imports `scripts.slice_asr_cli` directly, so the bootstrap harness still depends on module-import availability for excluded audio/model-adjacent packages.
- Evidence: `scripts/slice_asr_cli.py:29` and `scripts/slice_asr_cli.py:30` import `librosa` and `soundfile`; `scripts/slice_asr_cli.py:37` through `scripts/slice_asr_cli.py:40` import ASR, RMVPE, slicer, and device modules at module import time. The bootstrap correctly keeps those capabilities out of the replacement surface at `rewrite-in-rust/bootstrap/batch_cli_planning_and_index_core.md:68`, but the checker still requires them to be importable even though it does not call model or audio runtime paths.
- Required fix: Either split the legacy helper import surface so this checker can import deterministic helpers without heavy runtime imports, or document the full uv environment as a checker prerequisite and keep this as an accepted harness limitation.

## Boundary Decision

Manifest unit boundary: confirmed.

The unit should stay as a fixture-bound deterministic planning/index unit. The dependency expansion justifies keeping audio decoding, chunk/lab writing, JSON re-slicing, ASR, RMVPE, FFmpeg path mutation, full argparse help, and full device/language validation legacy-owned.

The Rust dependency choices are scoped to the declared capabilities: `encoding_rs` for GB18030/GBK repair parity and `md-5` for source-key hashing. `rewrite-in-rust/rust/crates/v2m-core/Cargo.toml:13` and `rewrite-in-rust/rust/crates/v2m-core/Cargo.toml:14` match the dependency record at `rewrite-in-rust/dependencies/batch_cli_planning_and_index_core.yaml:33` and `rewrite-in-rust/dependencies/batch_cli_planning_and_index_core.yaml:35`; `rewrite-in-rust/rust/Cargo.lock:41` and `rewrite-in-rust/rust/Cargo.lock:66` show the resolved crates.

Writer/reviewer separation: preserved. This review did not edit production code and did not mark the manifest verified.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_batch_cli_planning_and_index_core.py`: passed
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml batch_cli_planning`: passed, 1 test passed, 60 filtered out in `v2m-core`
- `uv run python -c "import yaml; yaml.safe_load(open('rewrite-in-rust/manifest.yaml')); yaml.safe_load(open('rewrite-in-rust/dependencies/batch_cli_planning_and_index_core.yaml'))"`: passed
- `uv run python scripts/audit_vendored_sources.py`: passed, 135 Python packages, 41 native-extension packages, 269 foreign runtime native binaries, 0 third_party binary artifacts

## Residual Risk

The remaining risk is fixture breadth, not seam choice. The bootstrap proves the main deterministic path and excludes runtime/model behavior correctly, but indexed lab/slice skip detection is not yet fixture-backed. The checker also remains coupled to the full legacy module import graph.

## Promotion Note

This dependency/bootstrap role does not block collecting the remaining required reviews. Do not promote or mark the unit verified until the missing indexed lab/slice fixture coverage is added or explicitly accepted by the behavior review, and until the stage behavior and product ergonomics review requirements are satisfied.
