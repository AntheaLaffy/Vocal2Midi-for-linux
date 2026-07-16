# batch_cli_planning_and_index_core - behavior_reviewer

Date: 2026-07-16
Decision: pass-with-followups

## Findings

- Severity: medium
- Location: rewrite-in-rust/rust/crates/v2m-core/src/batch_cli_planning.rs:296
- Issue: Source-index multi-record save parity still has an ordering gap for exact JSON text. Legacy Python preserves the loaded dict insertion order and appends new records when `update_source_index` runs, while Rust renders `serde_json::Map` iteration order. With the current `serde_json = "1"` dependency and no `preserve_order` feature, that map is backed by `BTreeMap`, so top-level source-MD5 keys are sorted. The new fixture proves a multi-record save only for `aaaold` then `zzznew`, where insertion order and sorted order happen to match.
- Evidence: The fixture added `source_index_loads_object_and_saves_multi_record` at `rewrite-in-rust/fixtures/batch_cli_planning_and_index_core.jsonl:23`, but its keys are already lexicographically ordered. Rust iterates the map at `rewrite-in-rust/rust/crates/v2m-core/src/batch_cli_planning.rs:298`. `rewrite-in-rust/rust/crates/v2m-core/Cargo.toml:15` enables default `serde_json`, and `cargo tree --manifest-path rewrite-in-rust/rust/Cargo.toml -p v2m-core -e features | rg "serde_json|preserve_order|indexmap"` showed only `serde_json feature "default"` and `std`, not `preserve_order`. Local serde_json source confirms default `Map` uses `BTreeMap` at `/home/fuurin/.cargo/registry/src/rsproxy.cn-e3de039b2554c837/serde_json-1.0.150/src/map.rs:3` and `:34`. A Python probe with initial key `zzzold` and appended key `aaanew` saved `zzzold` first, then `aaanew`, matching legacy insertion order.
- Required fix: Add an adversarial multi-record source-index fixture whose update key sorts before the existing key, then either preserve insertion order in Rust for source-index rendering or explicitly downgrade the saved JSON contract to semantic JSON-object parity rather than exact text ordering.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_batch_cli_planning_and_index_core.py`: passed.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml batch_cli_planning`: passed; `batch_cli_planning::tests::batch_cli_planning_fixtures_match` ok.
- `cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check`: passed.
- `cargo tree --manifest-path rewrite-in-rust/rust/Cargo.toml -p v2m-core -e features | rg "serde_json|preserve_order|indexmap"`: confirmed `serde_json` default/std only; no `preserve_order`/`indexmap`.
- `uv run python - <<'PY' ... non-lexicographic source-index save probe ... PY`: confirmed legacy saves existing `zzzold` before appended `aaanew`.

## Previous Findings Status

- Previous medium, source-index object + multi-record save: partially closed. The fixture now covers loading a valid object and saving a two-record UTF-8 index, and Rust now renders the whole index object. Exact top-level record ordering remains unproven and likely divergent for non-lexicographic insertion order.
- Previous low, malformed truthy non-string `output_key`: closed. The fixture now expects the legacy `TypeError` text at `rewrite-in-rust/fixtures/batch_cli_planning_and_index_core.jsonl:30`, the Python checker captures exception status at `rewrite-in-rust/bootstrap/check_batch_cli_planning_and_index_core.py:187`, and Rust returns a matching error through `legacy_output_key` at `rewrite-in-rust/rust/crates/v2m-core/src/batch_cli_planning.rs:650`.
- Indexed lab/slice completion: closed for the covered cases. Fixtures at `rewrite-in-rust/fixtures/batch_cli_planning_and_index_core.jsonl:25` and `:26` verify indexed lab and slice outputs independently from direct `has_existing_outputs` detection.
- Source-path handling: closed for the covered update path. The checker uses `audio_path.resolve()` through legacy `update_source_index`, normalizes it to `__case__`, and Rust now accepts the resolved source path separately at `rewrite-in-rust/rust/crates/v2m-core/src/batch_cli_planning.rs:268`.

## Residual Risk

The reviewed unit remains fixture-bound and outside production routing. Audio decoding, waveform chunk writing, JSON re-slicing, slicer algorithms, ASR/RMVPE runtime behavior, FFmpeg setup, and full CLI parser UX remain excluded by the boundary record.

The remaining risk is limited to exact source-index JSON text ordering. Semantically, the map contents, source paths, malformed-record behavior, indexed JSON/lab/slice completion, MD5/source-key behavior, skip detection, and fake batch-loop counters match the covered fixtures.

## Promotion Note

This rerun keeps the role name `behavior_reviewer` and can serve as behavior evidence for the manifest's `stage_behavior_reviewer` requirement, but it does not mark the manifest verified. Coordinator state update is reasonable if exact source-index key order is accepted as non-semantic or tracked as a follow-up; otherwise fix the ordering gap before verification.
