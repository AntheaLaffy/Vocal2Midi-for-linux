# download_models_asset_catalog_contract - behavior_reviewer

Date: 2026-07-16
Decision: pass

## Findings

No findings.

Evidence:

- `download_models.py:66` through `download_models.py:118` define the legacy
  `GithubModel` catalog, static four-model order, lookup-key order, Qwen model
  id, and Qwen local directory. The fixture preserves `game`, `hfa`, `rmvpe`,
  and `romaji` in that order at
  `rewrite-in-rust/fixtures/download_models_asset_catalog_contract.jsonl:1`,
  and Rust mirrors it in
  `rewrite-in-rust/rust/crates/v2m-core/src/download_models_catalog.rs:63`.
- `download_models.py:165` through `download_models.py:180` define
  `human_size` and `asset_url`. The fixture covers B/KiB/MiB/GiB/TiB boundaries,
  the TiB clamp, a negative input, and plain URL concatenation without URL
  encoding at
  `rewrite-in-rust/fixtures/download_models_asset_catalog_contract.jsonl:2`
  through
  `rewrite-in-rust/fixtures/download_models_asset_catalog_contract.jsonl:3`.
  Rust mirrors those helpers at
  `rewrite-in-rust/rust/crates/v2m-core/src/download_models_catalog.rs:113`
  through
  `rewrite-in-rust/rust/crates/v2m-core/src/download_models_catalog.rs:134`.
- `download_models.py:319` through `download_models.py:320` use
  `Path.exists()` for marker checks. The fixture includes file markers,
  directory markers, missing markers, and nested non-markers at
  `rewrite-in-rust/fixtures/download_models_asset_catalog_contract.jsonl:4`;
  the checker creates both file and directory entries, and Rust checks the same
  injected path set at
  `rewrite-in-rust/rust/crates/v2m-core/src/download_models_catalog.rs:136`
  through
  `rewrite-in-rust/rust/crates/v2m-core/src/download_models_catalog.rs:139`.
- `download_models.py:514` through `download_models.py:521` scan only immediate
  Qwen children by lowercased name and do not require file type checks. The
  fixture covers missing/empty directories, immediate `.safetensors`, uppercase
  `.BIN`, nested-only weights, a directory named `cache.bin`, and a non-weight
  file at
  `rewrite-in-rust/fixtures/download_models_asset_catalog_contract.jsonl:5`;
  Rust mirrors the immediate-name rule at
  `rewrite-in-rust/rust/crates/v2m-core/src/download_models_catalog.rs:141`
  through
  `rewrite-in-rust/rust/crates/v2m-core/src/download_models_catalog.rs:157`.
- `download_models.py:546` through `download_models.py:565` define the
  color-disabled `list_planned` rows, size lookup behavior, check/cross
  markers, label rows, Qwen `skip` to `skipped` display, and alignment widths.
  The fixture asserts exact output lines with injected asset sizes and fake
  filesystem state at
  `rewrite-in-rust/fixtures/download_models_asset_catalog_contract.jsonl:6`.
  The Python checker disables color and patches size lookup/root paths at
  `rewrite-in-rust/bootstrap/check_download_models_asset_catalog_contract.py:166`
  through
  `rewrite-in-rust/bootstrap/check_download_models_asset_catalog_contract.py:194`;
  Rust renders the matching lines at
  `rewrite-in-rust/rust/crates/v2m-core/src/download_models_catalog.rs:159`
  through
  `rewrite-in-rust/rust/crates/v2m-core/src/download_models_catalog.rs:199`.
- `rewrite-in-rust/manifest.yaml:695` through
  `rewrite-in-rust/manifest.yaml:712` keeps this unit scoped to deterministic
  metadata, marker, and dry-run display behavior with fake filesystem and
  injected size maps. Network calls, model weights, CLI planning, and effectful
  fetch remain outside this behavior gate.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_download_models_asset_catalog_contract.py`: passed.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml download_models_catalog`: passed; 1 matching Rust fixture test passed.
- Focused read of `download_models.py:66` through `download_models.py:118`,
  `download_models.py:165` through `download_models.py:180`,
  `download_models.py:319` through `download_models.py:320`, and
  `download_models.py:514` through `download_models.py:565` against the fixture,
  checker, Rust implementation, and manifest block: passed.
- `git diff --check -- download_models.py rewrite-in-rust/fixtures/download_models_asset_catalog_contract.jsonl rewrite-in-rust/bootstrap/check_download_models_asset_catalog_contract.py rewrite-in-rust/rust/crates/v2m-core/src/download_models_catalog.rs rewrite-in-rust/manifest.yaml`: passed.

## Residual Risk

This review proves the declared fixture-bound catalog/display surface, not a
production bridge. Rust display rows assume normalized relative target strings
equivalent to Python's `target.relative_to(ROOT_DIR)` output. Rare filesystem
states outside the declared Qwen directory contract, such as `QWEN_LOCAL_DIR`
existing as a regular file, are not modeled. Real GitHub API calls, downloads,
archive extraction, package installation, external Qwen CLIs, and model-weight
inspection remain split to later units or legacy runtime ownership.

## Promotion Note

This behavior role does not block coordinator state update for
`download_models_asset_catalog_contract`. The coordinator still owns manifest
state changes and any separate required review roles.
