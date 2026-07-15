# download_models_asset_catalog_contract - dependency_bootstrap_reviewer

Date: 2026-07-16
Decision: pass

## Findings

No blocking findings.

The manifest boundary is confirmed. `rewrite-in-rust/manifest.yaml:695` scopes this unit to `download_models_asset_catalog_contract`, marks it `reimplemented` and `confirmed`, and limits verification to fixtures, injected GitHub size maps, and fake filesystem state at `rewrite-in-rust/manifest.yaml:707`.

The split ownership is documented honestly. `rewrite-in-rust/records/0036-confirm-download-models-asset-catalog-boundary.md:17` says the catalog/display behavior is deterministic with injected size and filesystem state, while `rewrite-in-rust/dependencies/download_models_asset_catalog_contract.yaml:35` keeps archive extraction, CLI selection, GitHub API calls, streamed downloads, package installation, external Qwen CLI flow, cleanup, and model execution outside this unit.

The bootstrap seam does not introduce a runtime bridge or external dependency. `rewrite-in-rust/bootstrap/download_models_asset_catalog_contract.md:40` identifies only stdlib-backed behavior, `rewrite-in-rust/bootstrap/download_models_asset_catalog_contract.md:51` keeps the seam as an independent Rust library, and `rewrite-in-rust/bootstrap/download_models_asset_catalog_contract.md:87` keeps production ownership in `download_models.py`.

The checker and Rust code stay inside the declared dependency boundary. The Python checker patches `download_models.github_api_asset_sizes` with an injected map at `rewrite-in-rust/bootstrap/check_download_models_asset_catalog_contract.py:173` and restores it at `rewrite-in-rust/bootstrap/check_download_models_asset_catalog_contract.py:188`. The Rust module uses fixture inputs and in-memory collections, with no network, process, archive, package-install, or model-weight inspection APIs in `rewrite-in-rust/rust/crates/v2m-core/src/download_models_catalog.rs:7`.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_download_models_asset_catalog_contract.py`: pass.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml download_models_catalog`: pass; `download_models_catalog::tests::download_models_catalog_fixtures_match` passed.
- Scoped broad effect scan for `urllib`, `requests`, `urlopen`, process APIs, archive APIs, package install terms, external CLI terms, and model-weight terms across the checker, Rust module, bootstrap, and dependency record: pass; matches were documentation of excluded work, URL-format strings, Qwen marker suffixes, fixture inclusion, and the injected `github_api_asset_sizes` patch.
- Scoped actual effectful API scan for network/process/archive imports and calls across the same files: pass; no matches.
- `git diff --check -- rewrite-in-rust/manifest.yaml rewrite-in-rust/records/0036-confirm-download-models-asset-catalog-boundary.md rewrite-in-rust/dependencies/download_models_asset_catalog_contract.yaml rewrite-in-rust/bootstrap/download_models_asset_catalog_contract.md rewrite-in-rust/fixtures/download_models_asset_catalog_contract.jsonl rewrite-in-rust/bootstrap/check_download_models_asset_catalog_contract.py rewrite-in-rust/rust/crates/v2m-core/src/download_models_catalog.rs rewrite-in-rust/rust/crates/v2m-core/src/lib.rs`: pass; no whitespace errors reported.

## Residual Risk

This dependency-bootstrap pass does not replace the behavior review. Exact parity of every formatted row, marker edge case, and future catalog drift remains the behavior gate's responsibility.

The scoped files are currently uncommitted, and several unit files are untracked in this worktree; this review treats the working tree contents as the review target.

## Promotion Note

The dependency_bootstrap gate passes. The unit boundary should remain confirmed, not split, merged, deferred, or replaced.
