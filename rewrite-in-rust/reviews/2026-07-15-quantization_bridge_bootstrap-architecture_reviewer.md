# quantization_bridge_bootstrap - architecture_reviewer

Date: 2026-07-15
Decision: pass

## Findings

No architecture findings.

Evidence:

- Owner boundary is explicit in the manifest: `quantization_bridge_bootstrap` keeps `current_owner: legacy`, proves only the bridge contract, and rolls back by keeping `inference.quant.quantization` as the default owner (`rewrite-in-rust/manifest.yaml:279`, `rewrite-in-rust/manifest.yaml:288`, `rewrite-in-rust/manifest.yaml:300`).
- The split/promotion record keeps caller defaults and final routing outside this unit; `quantization_bridge_bootstrap` is only a prerequisite, while caller/default behavior and final production ownership remain separate work (`rewrite-in-rust/records/0011-split-quantization-promotion.md:27`, `rewrite-in-rust/records/0011-split-quantization-promotion.md:42`).
- The bridge shape has a matching record and rollback route: `v2m-quant-bridge` is a stdin/stdout JSON CLI, `inference/quant/rust_bridge.py` is opt-in only, production callers remain unchanged, and rollback is selecting legacy or not importing the wrapper (`rewrite-in-rust/records/0012-quantization-json-bridge-proof.md:15`, `rewrite-in-rust/records/0012-quantization-json-bridge-proof.md:19`, `rewrite-in-rust/records/0012-quantization-json-bridge-proof.md:31`, `rewrite-in-rust/records/0012-quantization-json-bridge-proof.md:41`).
- The implementation follows that shape: `quantize_notes_with_backend` defaults to `legacy`, only enters the bridge for `backend="rust-json"`, and requires an explicit executable or `V2M_QUANT_BRIDGE_BIN` (`inference/quant/rust_bridge.py:39`, `inference/quant/rust_bridge.py:44`, `inference/quant/rust_bridge.py:47`, `inference/quant/rust_bridge.py:64`).
- The CLI bridge is confined to the independent Rust workspace and bridge crate dependencies; the workspace adds `crates/v2m-quant-bridge`, and `serde`/`serde_json` are declared only in that bridge crate (`rewrite-in-rust/rust/Cargo.toml:2`, `rewrite-in-rust/rust/crates/v2m-quant-bridge/Cargo.toml:13`).
- Production caller search found the live pipeline still importing and calling `inference.quant.quantization.quantize_notes` and `should_apply_quantization`, with no production reference to `rust_bridge` or `quantize_notes_with_backend` outside the wrapper itself (`inference/pipeline/auto_lyric_hybrid.py:19`, `inference/pipeline/auto_lyric_hybrid.py:443`, `inference/pipeline/auto_lyric_hybrid.py:444`).
- The bridge response remains mutation-instruction shaped rather than a new note object graph: Rust returns `index`, `onset`, and `offset`, and the Python wrapper mutates/reorders the original note objects in place (`rewrite-in-rust/rust/crates/v2m-quant-bridge/src/main.rs:46`, `rewrite-in-rust/rust/crates/v2m-quant-bridge/src/main.rs:155`, `inference/quant/rust_bridge.py:135`, `inference/quant/rust_bridge.py:146`, `inference/quant/rust_bridge.py:153`).

## Checks

- `git status --short`: inspected current worktree; unit artifacts are present in a dirty tree, with no staged changes.
- `git diff -- rewrite-in-rust/manifest.yaml rewrite-in-rust/rust/Cargo.toml rewrite-in-rust/rust/crates/v2m-core/src/lib.rs`: inspected tracked control-plane/workspace diff.
- `git diff -- rewrite-in-rust/rust/Cargo.lock`: inspected new bridge dependency lockfile entries.
- `rg -n "quantize_notes_with_backend|inference\\.quant\\.rust_bridge|rust_bridge|V2M_QUANT_BACKEND|V2M_QUANT_BRIDGE_BIN|v2m-quant-bridge" .`: confirmed bridge references are limited to the wrapper, rewrite records/bootstrap/checks, workspace metadata, and bridge crate.
- `rg -n "\\bquantize_notes\\b|\\bshould_apply_quantization\\b" --glob '!rewrite-in-rust/**' --glob '!third_party/**' --glob '!*.pyc' .`: confirmed the production pipeline still calls the legacy quantization module directly.
- `cargo build --locked --manifest-path rewrite-in-rust/rust/Cargo.toml --bin v2m-quant-bridge`: passed.
- `cargo test --locked --manifest-path rewrite-in-rust/rust/Cargo.toml -p v2m-quant-bridge`: passed, 5 tests.
- `uv run python rewrite-in-rust/bootstrap/check_quantization_bridge_bootstrap.py`: passed.
- `uv run pytest tests/test_web_api.py`: passed, 53 tests.
- `cargo test --locked --manifest-path rewrite-in-rust/rust/Cargo.toml`: passed, 37 `v2m-core` tests, 5 `v2m-quant-bridge` tests, and doc tests.

## Residual Risk

This architecture review does not replace the required behavior or error-tracing reviews. It also does not approve `quantization_pipeline_promotion`; caller/default locking, product ergonomics, cancellation behavior, and representative performance remain promotion risks because no production caller routes to the bridge in this unit (`rewrite-in-rust/bootstrap/quantization_bridge_bootstrap.md:18`, `rewrite-in-rust/bootstrap/quantization_bridge_bootstrap.md:93`, `rewrite-in-rust/manifest.yaml:302`, `rewrite-in-rust/manifest.yaml:327`).

## Promotion Note

This architecture role does not block `quantization_bridge_bootstrap` verification once the other required reviews pass. It does not mark the manifest verified and does not approve final runtime owner promotion.
