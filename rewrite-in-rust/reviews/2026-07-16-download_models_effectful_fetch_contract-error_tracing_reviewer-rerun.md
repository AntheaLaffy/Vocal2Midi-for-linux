# download_models_effectful_fetch_contract - error_tracing_reviewer rerun

Date: 2026-07-16
Decision: pass

Unit: `download_models_effectful_fetch_contract`
Role: `error_tracing_reviewer`

## Findings

No blocking findings.

The previous medium finding for non-HTTP GitHub stream exceptions is covered.
Legacy `download_github_model` calls `stream_download` at `download_models.py:342`,
catches only `urllib.error.HTTPError` at `download_models.py:368`, and still
removes the temporary zip in the `finally` block at `download_models.py:377`.
Record 0038 now records the intended quirk: `URLError` and `TimeoutError`
escape after the initial download lines while temporary zip cleanup still runs
at `rewrite-in-rust/records/0038-confirm-download-models-effectful-fetch-boundary.md:34`.
The fixture table asserts both cases with exception status, type, message,
initial stdout, empty stderr, no extract calls, and no temp zip leftovers at
`rewrite-in-rust/fixtures/download_models_effectful_fetch_contract.jsonl:13`
and `rewrite-in-rust/fixtures/download_models_effectful_fetch_contract.jsonl:14`.
The Python checker injects those stream exceptions at
`rewrite-in-rust/bootstrap/check_download_models_effectful_fetch_contract.py:199`
through `rewrite-in-rust/bootstrap/check_download_models_effectful_fetch_contract.py:202`
and records escaped exception diagnostics plus temp-file leftovers at
`rewrite-in-rust/bootstrap/check_download_models_effectful_fetch_contract.py:230`
through `rewrite-in-rust/bootstrap/check_download_models_effectful_fetch_contract.py:244`.
The Rust fixture model stops before extraction with no stderr for those injected
stream errors at
`rewrite-in-rust/rust/crates/v2m-core/src/download_models_effectful.rs:208`,
and the Rust fixture adapter asserts `URLError` and `TimeoutError` status fields
at `rewrite-in-rust/rust/crates/v2m-core/src/download_models_effectful.rs:757`
through `rewrite-in-rust/rust/crates/v2m-core/src/download_models_effectful.rs:778`.

The previous low finding for Qwen cleanup unlink-error swallowing is covered.
Legacy `_cleanup_qwen_artifacts` only logs after successful unlink and swallows
`OSError` for `.gitattributes` and immediate `*.incomplete` files at
`download_models.py:440` through `download_models.py:452`. The dependency record
now requires checker coverage for cleanup unlink-error swallowing at
`rewrite-in-rust/dependencies/download_models_effectful_fetch_contract.yaml:29`.
The fixture row at
`rewrite-in-rust/fixtures/download_models_effectful_fetch_contract.jsonl:19`
asserts retained files and no cleanup success logs when both unlink attempts
fail. The Python checker uses a fake path whose `unlink()` raises `OSError` at
`rewrite-in-rust/bootstrap/check_download_models_effectful_fetch_contract.py:363`
through `rewrite-in-rust/bootstrap/check_download_models_effectful_fetch_contract.py:408`.
The Rust model carries `unlink_error_paths`, retains failed removals, and emits
no success log for them at
`rewrite-in-rust/rust/crates/v2m-core/src/download_models_effectful.rs:367`
through `rewrite-in-rust/rust/crates/v2m-core/src/download_models_effectful.rs:395`;
the Rust fixture adapter reads `unlink_error` from the JSONL at
`rewrite-in-rust/rust/crates/v2m-core/src/download_models_effectful.rs:850`
through `rewrite-in-rust/rust/crates/v2m-core/src/download_models_effectful.rs:877`.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_download_models_effectful_fetch_contract.py`: passed.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml download_models_effectful`: passed; `download_models_effectful::tests::download_models_effectful_fetch_fixtures_match` passed, with 1 matching test and 59 filtered out in `v2m-core`.
- `git diff --check -- rewrite-in-rust/manifest.yaml rewrite-in-rust/rust/crates/v2m-core/src/lib.rs`: passed.
- `rg -n "[ \\t]+$|<<<<<<<|=======$|>>>>>>>" ...target unit files...`: no trailing whitespace or conflict-marker matches.
- Focused read of the previous fail report, fixture JSONL, Python checker, Rust implementation, `download_models.py`, manifest entry, record 0038, dependency record, and bootstrap doc.

## Residual Risk

This pass covers the two prior error-tracing findings for the mocked contract.
Actual network transfer, real package installation, external CLI execution,
archive extraction, and model-weight inspection remain legacy-owned and outside
this unit, as stated in
`rewrite-in-rust/dependencies/download_models_effectful_fetch_contract.yaml:43`
through `rewrite-in-rust/dependencies/download_models_effectful_fetch_contract.yaml:45`
and `rewrite-in-rust/bootstrap/download_models_effectful_fetch_contract.md:42`
through `rewrite-in-rust/bootstrap/download_models_effectful_fetch_contract.md:44`.

A future production bridge must explicitly preserve or revise the escaped
stream-exception policy before Rust owns live IO. This review only approves the
fixture-backed error and cleanup diagnostics for the current library seam.

## Promotion Note

The error-tracing rerun passes and no longer blocks coordinator state update for
this role. The coordinator still owns manifest state changes, and Python remains
the runtime owner until a later bridge/promotion record changes ownership.
