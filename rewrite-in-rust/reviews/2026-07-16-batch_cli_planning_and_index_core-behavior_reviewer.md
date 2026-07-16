# batch_cli_planning_and_index_core - behavior_reviewer

Date: 2026-07-16
Decision: pass-with-followups

## Findings

- Severity: medium
- Location: rewrite-in-rust/fixtures/batch_cli_planning_and_index_core.jsonl:19
- Issue: Source-index persistence evidence does not cover loading a valid pre-existing object index or saving an updated multi-record index. The fixture covers missing, malformed, non-object, and a one-record UTF-8 update/save case, but the dependency record asks for source-index object-file coverage and the legacy helper persists the whole dict.
- Evidence: `rewrite-in-rust/dependencies/batch_cli_planning_and_index_core.yaml:42` requires object-file source-index fixtures; `rg -n "initial_index|source_index_loads_object|saved_json" rewrite-in-rust/fixtures/batch_cli_planning_and_index_core.jsonl` only found the one-record `saved_json` case at line 22. Legacy `load_source_index` and `save_source_index` operate on the whole JSON object at `scripts/slice_asr_cli.py:247` and `scripts/slice_asr_cli.py:260`. Rust test code renders saved JSON through the one-record helper at `rewrite-in-rust/rust/crates/v2m-core/src/batch_cli_planning.rs:294` and `rewrite-in-rust/rust/crates/v2m-core/src/batch_cli_planning.rs:787`.
- Required fix: Add a source-index fixture with `initial_index` containing at least one valid UTF-8 record, then update with a second record and assert `loaded`, `updated`, and exact `saved_json`. Update the Rust fixture runner to render the whole source-index object in Python-compatible order, or explicitly constrain the unit to map-shape parity rather than exact multi-record JSON text.

- Severity: low
- Location: scripts/slice_asr_cli.py:271
- Issue: Truthy non-string `output_key` values in a malformed source-index record have different observable behavior. Legacy Python raises `TypeError` while constructing `output_dir / "labs" / key`; Rust silently treats the same record as incomplete because it requires `output_key` to be a string.
- Evidence: `uv run python - <<'PY' ... cli.index_has_completed_output({'md5': {'output_key': 123}}, 'md5', out) ... PY` failed with `TypeError: unsupported operand type(s) for /: 'PosixPath' and 'int'`. Rust uses `record.get("output_key")?.as_str()?` at `rewrite-in-rust/rust/crates/v2m-core/src/batch_cli_planning.rs:359`, which returns `None` instead of preserving the legacy failure mode.
- Required fix: Either add a malformed-record fixture and make Rust return a legacy-compatible error for truthy non-string `output_key`, or document that the Rust fixture contract only accepts records written by `update_source_index`.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_batch_cli_planning_and_index_core.py`: passed.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml batch_cli_planning`: passed; `batch_cli_planning::tests::batch_cli_planning_fixtures_match` ok.
- `cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check`: passed.
- `rg -n "initial_index|source_index_loads_object|saved_json" rewrite-in-rust/fixtures/batch_cli_planning_and_index_core.jsonl`: only the one-record save fixture is present.
- `uv run python - <<'PY' ... index_has_completed_output malformed output_key probe ... PY`: confirmed legacy raises `TypeError` for truthy non-string `output_key`.

## Residual Risk

The reviewed seam stays inside the confirmed deterministic planning/index boundary and does not route Rust into production. Audio decode/write, JSON re-slicing, slicer algorithms, ASR/RMVPE runtime behavior, FFmpeg setup, and full CLI parser UX remain intentionally excluded by `rewrite-in-rust/records/0039-confirm-batch-cli-planning-index-boundary.md`.

Fixture parity is strongest for normal legacy-written source-index records, supported file suffix scans, batch grouping, slicing method normalization, slice bounds, MD5/source-key generation, existing-output checks, and fake loop counters. Edge behavior around arbitrary malformed source-index record values and exact multi-record JSON save text remains unproven.

## Promotion Note

This report keeps the role name `behavior_reviewer` and can serve as behavior evidence for the manifest's `stage_behavior_reviewer` requirement, but it does not mark the manifest verified. The unit is acceptable as `reimplemented`; coordinator promotion should either track the two follow-ups or explicitly accept the valid-record-only source-index contract.
