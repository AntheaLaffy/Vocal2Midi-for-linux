# batch_cli_planning_and_index_core - behavior_reviewer

Date: 2026-07-16
Decision: pass

## Findings

No findings.

## Previous Findings Status

- Previous medium, source-index object + multi-record save ordering: closed. The fixture now includes `source_index_preserves_insertion_order_not_sorted_order` at `rewrite-in-rust/fixtures/batch_cli_planning_and_index_core.jsonl:24`, with existing key `zzzold` followed by appended key `aaaanew`. `rewrite-in-rust/rust/crates/v2m-core/Cargo.toml:15` enables `serde_json` with `preserve_order`, and `cargo tree --manifest-path rewrite-in-rust/rust/Cargo.toml -p v2m-core -e features | rg "serde_json|preserve_order|indexmap"` confirms `serde_json feature "preserve_order"` and `indexmap`.
- Previous low, malformed truthy non-string `output_key`: closed. The fixture at `rewrite-in-rust/fixtures/batch_cli_planning_and_index_core.jsonl:31` expects the legacy `TypeError` text; the Python checker captures error status at `rewrite-in-rust/bootstrap/check_batch_cli_planning_and_index_core.py:187`; Rust returns the matching error through `legacy_output_key` at `rewrite-in-rust/rust/crates/v2m-core/src/batch_cli_planning.rs:650`.
- Source-path handling remains covered. The Python checker uses legacy `update_source_index` then normalizes the resolved path at `rewrite-in-rust/bootstrap/check_batch_cli_planning_and_index_core.py:151`; Rust accepts the resolved fixture path separately at `rewrite-in-rust/rust/crates/v2m-core/src/batch_cli_planning.rs:268` and uses it in the test harness at `rewrite-in-rust/rust/crates/v2m-core/src/batch_cli_planning.rs:857`.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_batch_cli_planning_and_index_core.py`: passed.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml batch_cli_planning`: passed; `batch_cli_planning::tests::batch_cli_planning_fixtures_match` ok.
- `cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check`: passed.
- `cargo tree --manifest-path rewrite-in-rust/rust/Cargo.toml -p v2m-core -e features | rg "serde_json|preserve_order|indexmap"`: passed and confirmed `preserve_order`/`indexmap` are active.

## Residual Risk

The review only covers the fixture-bound planning/index unit. Audio decoding, waveform chunk writing, JSON re-slicing, slicer algorithms, ASR/RMVPE runtime behavior, FFmpeg setup, and full CLI parser UX remain outside this unit.

## Promotion Note

This final rerun keeps the role name `behavior_reviewer` and can serve as behavior evidence for the manifest's `stage_behavior_reviewer` requirement. I did not mark the manifest verified. From this role, the unit is ready for coordinator state update.
