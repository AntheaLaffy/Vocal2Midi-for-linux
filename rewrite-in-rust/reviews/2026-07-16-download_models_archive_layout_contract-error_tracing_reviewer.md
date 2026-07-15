# download_models_archive_layout_contract - error_tracing_reviewer

Date: 2026-07-16
Decision: fail

## Findings

- Severity: medium
- Location: rewrite-in-rust/rust/crates/v2m-core/src/download_models_archive.rs:111
- Issue: Unsafe archive error text does not fully preserve legacy Python `repr` formatting for all unsafe member names.
- Evidence: Legacy `download_models.py` builds the user-visible error with `ValueError(f"Unsafe zip member path: {name!r}")` in both unsafe-member branches at `download_models.py:291` and `download_models.py:303`. The Rust replacement hand-rolls `python_string_repr` at `rewrite-in-rust/rust/crates/v2m-core/src/download_models_archive.rs:117`, always starts with a single quote at line 118, only escapes a small subset of control characters at lines 120-127, and therefore diverges from Python for unsafe names such as `../evil's.txt` and `../evil\x08.txt`. Manual legacy probe output was:

  ```text
  Unsafe zip member path: "../evil's.txt"
  Unsafe zip member path: '../evil\x08.txt'
  ```

  The current fixtures prove the empty, backslash, and NUL cases at `rewrite-in-rust/fixtures/download_models_archive_layout_contract.jsonl:7`, `rewrite-in-rust/fixtures/download_models_archive_layout_contract.jsonl:13`, and `rewrite-in-rust/fixtures/download_models_archive_layout_contract.jsonl:14`, but they do not cover these repr edge cases.
- Required fix: Add unsafe-member fixtures containing a single quote and at least one non-`\\n`/`\\r`/`\\t` nonprintable control character, then make the Rust formatter match Python `repr` for those cases before rerunning this role.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_download_models_archive_layout_contract.py`: passed.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml download_models_archive`: passed; 1 matching Rust test passed.
- `uv run pytest tests/test_web_api.py -k ModelZipExtraction -q`: passed; 3 passed, 50 deselected.
- `rg -n "println!|eprintln!|dbg!|trace!|debug!|info!|warn!|error!|tracing::|log::|BadZipFile|stream_download|target_has_model|github_api|ZipArchive|reqwest|ureq|marker|human_size|asset_url|qwen|download_github_model" ...`: no matches in the unit fixture/checker/Rust files, so the unit does not invent logging/tracing or pull in residual download/marker paths.
- `git diff --check -- rewrite-in-rust/fixtures/download_models_archive_layout_contract.jsonl rewrite-in-rust/bootstrap/check_download_models_archive_layout_contract.py rewrite-in-rust/rust/crates/v2m-core/src/download_models_archive.rs rewrite-in-rust/dependencies/download_models_archive_layout_contract.yaml rewrite-in-rust/bootstrap/download_models_archive_layout_contract.md rewrite-in-rust/records/0035-split-download-models-asset-safety.md rewrite-in-rust/manifest.yaml`: passed.
- Manual legacy probe for `''`, backslash, NUL, single quote, and `\x08` unsafe names: confirmed fixture-covered cases match legacy and exposed the uncovered repr mismatch above.

## Residual Risk

The reviewed unit correctly keeps `BadZipFile`, download/network failures, package installation, marker verification, and model asset execution outside the Rust claim, as documented in `rewrite-in-rust/bootstrap/download_models_archive_layout_contract.md` and `rewrite-in-rust/dependencies/download_models_archive_layout_contract.yaml`. This review did not assess full archive extraction semantics beyond the fixture-backed error-message and boundary checks.

## Promotion Note

This error-tracing role blocks coordinator state update for `download_models_archive_layout_contract` until the repr mismatch is fixed and rerun.
