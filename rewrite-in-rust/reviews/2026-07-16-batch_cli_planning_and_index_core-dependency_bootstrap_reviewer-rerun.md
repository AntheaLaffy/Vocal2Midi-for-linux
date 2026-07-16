# batch_cli_planning_and_index_core - dependency_bootstrap_reviewer rerun

Date: 2026-07-16
Decision: pass-with-followups

## Findings

- Severity: low
- Location: rewrite-in-rust/fixtures/batch_cli_planning_and_index_core.jsonl:23
- Issue: The new multi-record source-index save fixture proves that an existing record is preserved, but it does not prove Python insertion-order save parity because the existing key `aaaold` and new key `zzznew` have the same order under both insertion order and sorted map order.
- Evidence: Legacy `save_source_index` writes `json.dumps(index, ensure_ascii=False, indent=2)` at `scripts/slice_asr_cli.py:260`, preserving Python dict insertion order. The Rust renderer iterates `index.iter()` at `rewrite-in-rust/rust/crates/v2m-core/src/batch_cli_planning.rs:298`, and `rewrite-in-rust/rust/crates/v2m-core/Cargo.toml:15` depends on plain `serde_json = "1"` without an order-preserving feature. `cargo tree --manifest-path rewrite-in-rust/rust/Cargo.toml -p v2m-core --edges normal` shows `serde_json v1.0.150` as a normal dependency and no order-preserving map crate.
- Required fix: Add a source-index save fixture where the pre-existing key sorts after the newly inserted key, for example existing `zzzold` followed by update `aaa_new`, or explicitly decide that byte-for-byte save ordering is not part of this unit's compatibility surface.

## Previous Finding Status

Previous medium finding: closed.

The prior gap was missing fixture coverage for indexed lab/slice completion. The fixture table now includes `existing_outputs_indexed_lab_completion` at `rewrite-in-rust/fixtures/batch_cli_planning_and_index_core.jsonl:25` and `existing_outputs_indexed_slice_completion` at `rewrite-in-rust/fixtures/batch_cli_planning_and_index_core.jsonl:26`. The checker catches indexed completion and errors through `rewrite-in-rust/bootstrap/check_batch_cli_planning_and_index_core.py:187`, and the Rust implementation covers indexed lab and slice branches at `rewrite-in-rust/rust/crates/v2m-core/src/batch_cli_planning.rs:384` and `rewrite-in-rust/rust/crates/v2m-core/src/batch_cli_planning.rs:392`.

## Coverage Notes

Manifest unit boundary: confirmed. The unit remains a deterministic planning/index seam with legacy runtime ownership at `rewrite-in-rust/manifest.yaml:813` through `rewrite-in-rust/manifest.yaml:834`. No production bridge was introduced.

Malformed truthy non-string `output_key` behavior is fixture-backed at `rewrite-in-rust/fixtures/batch_cli_planning_and_index_core.jsonl:30`, checked against Python exceptions at `rewrite-in-rust/bootstrap/check_batch_cli_planning_and_index_core.py:187`, and modeled in Rust at `rewrite-in-rust/rust/crates/v2m-core/src/batch_cli_planning.rs:650`.

Caller-supplied source-path handling is represented in the updated boundary at `rewrite-in-rust/bootstrap/batch_cli_planning_and_index_core.md:41`, in the Rust API shape at `rewrite-in-rust/rust/crates/v2m-core/src/batch_cli_planning.rs:267`, and in source-index fixtures at `rewrite-in-rust/fixtures/batch_cli_planning_and_index_core.jsonl:22` and `rewrite-in-rust/fixtures/batch_cli_planning_and_index_core.jsonl:23`.

Rust dependency choices remain aligned with the dependency record: `encoding_rs` and `md-5` are declared at `rewrite-in-rust/rust/crates/v2m-core/Cargo.toml:13` and `rewrite-in-rust/rust/crates/v2m-core/Cargo.toml:14`, matching `rewrite-in-rust/dependencies/batch_cli_planning_and_index_core.yaml:33` and `rewrite-in-rust/dependencies/batch_cli_planning_and_index_core.yaml:35`.

Writer/reviewer separation: preserved. This rerun did not edit production code and did not mark the manifest verified.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_batch_cli_planning_and_index_core.py`: passed
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml batch_cli_planning`: passed, 1 test passed, 60 filtered out in `v2m-core`
- `uv run python -c "import yaml; yaml.safe_load(open('rewrite-in-rust/manifest.yaml')); yaml.safe_load(open('rewrite-in-rust/dependencies/batch_cli_planning_and_index_core.yaml'))"`: passed
- `cargo tree --manifest-path rewrite-in-rust/rust/Cargo.toml -p v2m-core --edges normal`: passed; dependency tree includes `encoding_rs`, `md-5`, and `serde_json`
- `uv run python scripts/audit_vendored_sources.py`: passed, 135 Python packages, 41 native-extension packages, 269 foreign runtime native binaries, 0 third_party binary artifacts

## Residual Risk

The main previous fixture gap is closed. Remaining risk is narrow: exact multi-record source-index JSON ordering is still not proven when insertion order differs from key sort order. The Python checker also still imports `scripts.slice_asr_cli.py`, so it requires the full uv environment even though the fixture path does not call audio/model runtime behavior.

## Promotion Note

This dependency/bootstrap role is pass-with-followups and does not require splitting, merging, deferring, or replacing the manifest unit. Do not mark the unit verified from this report alone; coordinator state updates still need the required review set and an explicit decision on the source-index ordering follow-up.
