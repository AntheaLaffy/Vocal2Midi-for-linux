# download_models_archive_layout_contract - error_tracing_reviewer rerun

Date: 2026-07-16
Decision: pass

## Findings

No findings.

The previous failure is resolved for the scoped repr edges. Legacy
`download_models.py::_validated_zip_member_path` still raises
`ValueError(f"Unsafe zip member path: {name!r}")` for both early unsafe-name
checks and path escape checks at `download_models.py:291` through
`download_models.py:303`. The fixture table now covers both missing cases:
single quote quote-selection behavior at
`rewrite-in-rust/fixtures/download_models_archive_layout_contract.jsonl:15`
and non-`\\n`/`\\r`/`\\t` control-character hex escaping at
`rewrite-in-rust/fixtures/download_models_archive_layout_contract.jsonl:16`.

The Python checker sends those fixture names through the legacy function at
`rewrite-in-rust/bootstrap/check_download_models_archive_layout_contract.py:97`
through `rewrite-in-rust/bootstrap/check_download_models_archive_layout_contract.py:110`.
The Rust unit formats the same user-visible error through
`python_string_repr` at
`rewrite-in-rust/rust/crates/v2m-core/src/download_models_archive.rs:117`
through `rewrite-in-rust/rust/crates/v2m-core/src/download_models_archive.rs:143`,
and the Rust fixture test consumes the shared fixture table at
`rewrite-in-rust/rust/crates/v2m-core/src/download_models_archive.rs:180`
through `rewrite-in-rust/rust/crates/v2m-core/src/download_models_archive.rs:189`.

The reviewed files still keep this unit inside the archive member validation
and layout boundary. I found no invented logging/tracing, `BadZipFile`,
download/network, marker-check, package-installation, or model-weight failure
paths in the fixture/checker/Rust unit.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_download_models_archive_layout_contract.py`: passed.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml download_models_archive`: passed; 1 matching Rust test passed.
- `uv run pytest tests/test_web_api.py -k ModelZipExtraction -q`: passed; 3 passed, 50 deselected.
- `uv run python -c "... _validated_zip_member_path(...)"`: passed as a manual legacy repr probe, producing `Unsafe zip member path: "bad'name/../x"` and `Unsafe zip member path: 'bad\\x08/../x'`.
- `rg -n "println!|eprintln!|dbg!|trace!|debug!|info!|warn!|error!|tracing::|log::|BadZipFile|stream_download|target_has_model|github_api|ZipArchive|reqwest|ureq|marker|human_size|asset_url|qwen|download_github_model" rewrite-in-rust/fixtures/download_models_archive_layout_contract.jsonl rewrite-in-rust/bootstrap/check_download_models_archive_layout_contract.py rewrite-in-rust/rust/crates/v2m-core/src/download_models_archive.rs`: no matches.
- `cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check`: passed.
- `git diff --check -- rewrite-in-rust/manifest.yaml rewrite-in-rust/records/0035-split-download-models-asset-safety.md rewrite-in-rust/dependencies/download_models_archive_layout_contract.yaml rewrite-in-rust/bootstrap/download_models_archive_layout_contract.md rewrite-in-rust/fixtures/download_models_archive_layout_contract.jsonl rewrite-in-rust/bootstrap/check_download_models_archive_layout_contract.py rewrite-in-rust/rust/crates/v2m-core/src/download_models_archive.rs`: passed.

## Residual Risk

This rerun proves the previous single-quote and `\\x08` repr findings through
fixtures shared by the legacy checker and Rust test. It is not a full audit of
Python `repr` for every possible Unicode nonprintable member name outside the
current archive-layout fixture contract.

## Promotion Note

This error-tracing rerun does not block coordinator state update for
`download_models_archive_layout_contract`. The coordinator still owns manifest
state changes and any later promotion decision.
