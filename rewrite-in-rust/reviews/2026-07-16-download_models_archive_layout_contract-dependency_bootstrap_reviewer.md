# download_models_archive_layout_contract - dependency_bootstrap_reviewer

Date: 2026-07-16
Decision: pass

## Findings

No findings.

The manifest unit boundary is confirmed. The split from the broader
`download_models_asset_safety` candidate is justified because archive member
validation and merge layout are pure path/zip/file behavior, while catalog,
CLI-selection, and effectful network/CLI fetch behavior have different
verification shapes and are assigned to separate follow-up units.

Evidence:

- `rewrite-in-rust/records/0035-split-download-models-asset-safety.md:5`
  through `rewrite-in-rust/records/0035-split-download-models-asset-safety.md:21`
  identify the mixed capabilities in the old broad unit and why they require
  separate verification.
- `rewrite-in-rust/records/0035-split-download-models-asset-safety.md:25`
  through `rewrite-in-rust/records/0035-split-download-models-asset-safety.md:45`
  confirm the four-way split and keep GitHub metadata, CLI selection, network,
  package installation, external CLIs, and model weight/format execution outside
  this unit.
- `rewrite-in-rust/manifest.yaml:675` through
  `rewrite-in-rust/manifest.yaml:693` define this unit as safe archive member
  validation and merge layout only, with no network, package install, or model
  weight reading.
- `rewrite-in-rust/manifest.yaml:695` through
  `rewrite-in-rust/manifest.yaml:740` assign catalog/marker helpers,
  CLI-selection planning, and effectful fetch behavior to follow-up units.
- `rewrite-in-rust/dependencies/download_models_archive_layout_contract.yaml:12`
  through
  `rewrite-in-rust/dependencies/download_models_archive_layout_contract.yaml:21`
  record a legacy-owned library seam with no bridge dependencies and fixtures
  that avoid network, package installs, external CLIs, and model weights.
- `rewrite-in-rust/dependencies/download_models_archive_layout_contract.yaml:32`
  through
  `rewrite-in-rust/dependencies/download_models_archive_layout_contract.yaml:40`
  explicitly keep catalog, CLI, effectful fetch, and model weight/format
  capabilities legacy-owned or assigned elsewhere.
- `rewrite-in-rust/bootstrap/download_models_archive_layout_contract.md:34`
  through `rewrite-in-rust/bootstrap/download_models_archive_layout_contract.md:57`
  justify stdlib-only dependency expansion, an independent Rust library seam,
  and no production bridge.
- `download_models.py:247` through `download_models.py:317` show the legacy
  archive helpers in scope.
- `download_models.py:323` through `download_models.py:635` show the excluded
  download, package install, Qwen CLI, marker, list, parse, and main behaviors.
- `rewrite-in-rust/rust/crates/v2m-core/src/download_models_archive.rs:1`
  through
  `rewrite-in-rust/rust/crates/v2m-core/src/download_models_archive.rs:101`
  implement only member-name validation and target-layout modeling.
- `rewrite-in-rust/rust/crates/v2m-core/Cargo.toml:12` through
  `rewrite-in-rust/rust/crates/v2m-core/Cargo.toml:13` add no archive, network,
  subprocess, package-manager, or model-weight dependency.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_download_models_archive_layout_contract.py`: passed.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml download_models_archive`: passed, 1 matching Rust test passed.
- `uv run pytest tests/test_web_api.py -k ModelZipExtraction -q`: passed, 3 tests passed.
- `git diff --check -- rewrite-in-rust/manifest.yaml rewrite-in-rust/records/0035-split-download-models-asset-safety.md rewrite-in-rust/dependencies/download_models_archive_layout_contract.yaml rewrite-in-rust/bootstrap/download_models_archive_layout_contract.md rewrite-in-rust/fixtures/download_models_archive_layout_contract.jsonl rewrite-in-rust/bootstrap/check_download_models_archive_layout_contract.py rewrite-in-rust/rust/crates/v2m-core/src/download_models_archive.rs rewrite-in-rust/rust/crates/v2m-core/src/lib.rs`: passed.
- `cargo tree --manifest-path rewrite-in-rust/rust/Cargo.toml -p v2m-core`: passed; `v2m-core` depends only on `serde_json` for this workspace crate.
- `rg -n "std::process|Command::|tokio::process|reqwest|ureq|curl|urlopen|urllib|pip|install|modelscope|huggingface|zip::|ZipArchive|safetensors|onnx|gguf|read_to_end|File::open" rewrite-in-rust/rust/crates/v2m-core/src/download_models_archive.rs rewrite-in-rust/bootstrap/check_download_models_archive_layout_contract.py rewrite-in-rust/dependencies/download_models_archive_layout_contract.yaml rewrite-in-rust/bootstrap/download_models_archive_layout_contract.md`: passed as a scoped effectful-API scan; hits were only boundary documentation and `.onnx`/`.zip` layout text, not effectful APIs.

## Residual Risk

This review does not prove behavior parity for every possible `zipfile`
metadata case, symlink/platform extraction nuance, directory-only archive, or
real model release archive. The unit intentionally models member names and
final layout rather than introducing a Rust zip extraction dependency or
production bridge. Catalog, CLI selection, effectful fetch, package install,
external CLI, and model-weight checks remain provisional follow-up units.

## Promotion Note

This dependency/bootstrap role does not block coordinator state update for the
unit. The coordinator should not mark the manifest verified from this report
alone; the remaining required review roles are separate gates.
