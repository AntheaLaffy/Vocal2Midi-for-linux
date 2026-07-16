# batch_cli_planning_and_index_core - dependency_bootstrap_reviewer final rerun

Date: 2026-07-16
Decision: pass

## Findings

No findings.

## Previous Finding Status

Previous key-order low follow-up: closed.

The fixture `source_index_preserves_insertion_order_not_sorted_order` at `rewrite-in-rust/fixtures/batch_cli_planning_and_index_core.jsonl:24` now distinguishes insertion order from sorted key order by preserving `zzzold` before the later inserted `aaaanew`. The Python checker writes the initial object, calls the legacy update/save helpers, and compares saved JSON at `rewrite-in-rust/bootstrap/check_batch_cli_planning_and_index_core.py:132` through `rewrite-in-rust/bootstrap/check_batch_cli_planning_and_index_core.py:164`.

The Rust side now has the matching dependency support: `rewrite-in-rust/rust/crates/v2m-core/Cargo.toml:15` enables `serde_json` with `preserve_order`; `rewrite-in-rust/rust/Cargo.lock:72` records `indexmap`, and `rewrite-in-rust/rust/Cargo.lock:152` through `rewrite-in-rust/rust/Cargo.lock:157` show `serde_json` depending on `indexmap`. The Rust source-index harness renders from the ordered map at `rewrite-in-rust/rust/crates/v2m-core/src/batch_cli_planning.rs:296` through `rewrite-in-rust/rust/crates/v2m-core/src/batch_cli_planning.rs:322`, and the fixture test path compares that rendered JSON at `rewrite-in-rust/rust/crates/v2m-core/src/batch_cli_planning.rs:851` through `rewrite-in-rust/rust/crates/v2m-core/src/batch_cli_planning.rs:867`.

## Boundary Decision

Manifest unit boundary: confirmed. The unit remains `reimplemented`, not `verified`, at `rewrite-in-rust/manifest.yaml:813` through `rewrite-in-rust/manifest.yaml:834`; this report does not mark the manifest verified.

No split, merge, defer, or replacement is recommended for this dependency/bootstrap role. The dependency expansion remains narrow: `encoding_rs` for legacy mojibake repair, `md-5` for source-key hashing, and `serde_json/preserve_order` for Python-compatible source-index object ordering.

Writer/reviewer separation: preserved. This final rerun did not edit production code.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_batch_cli_planning_and_index_core.py`: passed
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml batch_cli_planning`: passed, 1 test passed, 60 filtered out in `v2m-core`
- `cargo tree --manifest-path rewrite-in-rust/rust/Cargo.toml -p v2m-core --edges features | rg -n "serde_json|preserve_order|indexmap"`: passed; output shows `serde_json feature "preserve_order"` and `indexmap`
- `uv run python -c "import yaml; yaml.safe_load(open('rewrite-in-rust/manifest.yaml')); yaml.safe_load(open('rewrite-in-rust/dependencies/batch_cli_planning_and_index_core.yaml'))"`: passed

## Residual Risk

Residual risk is limited to this review role's scope: dependency/bootstrap evidence is sufficient, but promotion still depends on the coordinator's required review set and state update policy.

## Promotion Note

This dependency/bootstrap role passes and no longer blocks coordinator state update. Do not mark the unit verified from this report alone unless the coordinator confirms the remaining required review evidence is complete.
