# download_models_archive_layout_contract - behavior_reviewer

Date: 2026-07-16
Decision: pass

## Findings

No findings.

Evidence:

- `download_models.py:247` through `download_models.py:317` define the legacy
  behavior under review: safe member validation, temp extraction, single
  top-level directory stripping, fallback direct/mixed merge, and overwrite
  copy behavior.
- `tests/test_web_api.py:878` through `tests/test_web_api.py:916` keep the
  legacy Web regression tests for single-top-level stripping, parent traversal
  rejection, and Windows absolute/drive rejection.
- `rewrite-in-rust/fixtures/download_models_archive_layout_contract.jsonl:1`
  through `rewrite-in-rust/fixtures/download_models_archive_layout_contract.jsonl:14`
  cover safe extraction layout, parent traversal rejection, nested traversal
  rejection, empty member rejection, NUL rejection, backslash rejection, POSIX
  absolute rejection, Windows absolute rejection, drive-relative rejection,
  single-top-level stripping, direct-file merge, mixed-top-level merge,
  `.onnx`/`.zip` top-level non-stripping, and overwrite layout.
- `rewrite-in-rust/bootstrap/check_download_models_archive_layout_contract.py:68`
  through `rewrite-in-rust/bootstrap/check_download_models_archive_layout_contract.py:110`
  exercise the same fixtures against the legacy Python helpers using temp zip
  archives for extraction cases and direct `_validated_zip_member_path` calls
  for the NUL member case that cannot be represented safely as a normal zip
  extraction fixture.
- `rewrite-in-rust/rust/crates/v2m-core/src/download_models_archive.rs:33`
  through `rewrite-in-rust/rust/crates/v2m-core/src/download_models_archive.rs:101`
  mirror the validated member-name rules and final target layout from explicit
  member lists. The model preserves the fixture-backed behavior while avoiding
  real zip extraction ownership.
- `rewrite-in-rust/rust/crates/v2m-core/src/download_models_archive.rs:1`
  through `rewrite-in-rust/rust/crates/v2m-core/src/download_models_archive.rs:6`
  state the boundary clearly: Python remains runtime owner for real zip
  extraction, network downloads, package installation, and model assets.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_download_models_archive_layout_contract.py`: passed.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml download_models_archive`: passed; 1 matching Rust test passed.
- `uv run pytest tests/test_web_api.py -k ModelZipExtraction -q`: passed; 3 tests passed.
- `cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check`: passed.
- `git diff --check -- rewrite-in-rust/fixtures/download_models_archive_layout_contract.jsonl rewrite-in-rust/bootstrap/check_download_models_archive_layout_contract.py rewrite-in-rust/rust/crates/v2m-core/src/download_models_archive.rs rewrite-in-rust/bootstrap/download_models_archive_layout_contract.md rewrite-in-rust/dependencies/download_models_archive_layout_contract.yaml rewrite-in-rust/records/0035-split-download-models-asset-safety.md rewrite-in-rust/manifest.yaml`: passed.
- `rg -n "Zip|zipfile|extract|Command|reqwest|ureq|TcpStream|download|network|std::fs|File::|copy\\(|create_dir|remove_|kill|spawn" rewrite-in-rust/rust/crates/v2m-core/src/download_models_archive.rs rewrite-in-rust/bootstrap/check_download_models_archive_layout_contract.py`: passed as a scoped ownership/effect scan. Hits are expected boundary text, Python checker temp-zip setup, fixture inclusion, and `.onnx`/`.zip` layout names; the Rust unit does not read zip bytes, spawn commands, contact network services, or mutate production files.

## Residual Risk

This review proves the declared fixture-bound member-name and final-layout
contract, not a complete Rust zip extractor. It does not cover arbitrary zip
metadata, duplicate member ordering beyond the current fixtures, symlink or
platform-specific extraction attributes, binary model-weight inspection, or
real release archives. Those remain outside this unit by design.

## Promotion Note

This behavior role does not block coordinator state update for
`download_models_archive_layout_contract`. The coordinator still owns manifest
state changes and any separate required review roles.
