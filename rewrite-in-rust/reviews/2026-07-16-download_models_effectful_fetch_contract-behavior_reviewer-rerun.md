# download_models_effectful_fetch_contract - behavior_reviewer rerun

Date: 2026-07-16
Decision: pass

Unit: `download_models_effectful_fetch_contract`
Role: `behavior_reviewer`

## Findings

No behavior findings.

Behavior evidence reviewed:

- Location: rewrite-in-rust/fixtures/download_models_effectful_fetch_contract.jsonl:13
- Assessment: Non-HTTP GitHub stream exception parity is now fixture-backed.
- Evidence: Legacy `download_github_model` calls `stream_download` at `download_models.py:342`, catches only `urllib.error.HTTPError` at `download_models.py:368`, and unlinks the temporary zip in the `finally` block at `download_models.py:377` through `download_models.py:381`. Record 0038 intentionally preserves this quirk at `rewrite-in-rust/records/0038-confirm-download-models-effectful-fetch-boundary.md:34` through `rewrite-in-rust/records/0038-confirm-download-models-effectful-fetch-boundary.md:35`. Fixture lines `rewrite-in-rust/fixtures/download_models_effectful_fetch_contract.jsonl:13` and `rewrite-in-rust/fixtures/download_models_effectful_fetch_contract.jsonl:14` assert the escaped `URLError` and `TimeoutError` shapes, initial stdout, no stderr, no extraction, and no temp-zip leftovers. The Python checker injects these stream errors at `rewrite-in-rust/bootstrap/check_download_models_effectful_fetch_contract.py:199` through `rewrite-in-rust/bootstrap/check_download_models_effectful_fetch_contract.py:202` and records escaped exception output at `rewrite-in-rust/bootstrap/check_download_models_effectful_fetch_contract.py:230` through `rewrite-in-rust/bootstrap/check_download_models_effectful_fetch_contract.py:244`. The Rust model stops before extraction at `rewrite-in-rust/rust/crates/v2m-core/src/download_models_effectful.rs:208` through `rewrite-in-rust/rust/crates/v2m-core/src/download_models_effectful.rs:217`, and its fixture adapter emits matching error status at `rewrite-in-rust/rust/crates/v2m-core/src/download_models_effectful.rs:757` through `rewrite-in-rust/rust/crates/v2m-core/src/download_models_effectful.rs:778`.
- Required fix: None.

- Location: rewrite-in-rust/fixtures/download_models_effectful_fetch_contract.jsonl:19
- Assessment: Qwen cleanup unlink-error swallowing is now fixture-backed.
- Evidence: Legacy `_cleanup_qwen_artifacts` swallows `OSError` for `.gitattributes` and immediate `*.incomplete` cleanup at `download_models.py:440` through `download_models.py:452`. Record 0038 documents retained files with no successful removal log at `rewrite-in-rust/records/0038-confirm-download-models-effectful-fetch-boundary.md:37` through `rewrite-in-rust/records/0038-confirm-download-models-effectful-fetch-boundary.md:39`, and the dependency record requires this checker coverage at `rewrite-in-rust/dependencies/download_models_effectful_fetch_contract.yaml:29`. Fixture line `rewrite-in-rust/fixtures/download_models_effectful_fetch_contract.jsonl:19` asserts retained `.gitattributes`, retained `a.incomplete`, retained `keep.bin`, and no stdout logs. The Python checker uses a fake path whose `unlink()` raises `OSError` at `rewrite-in-rust/bootstrap/check_download_models_effectful_fetch_contract.py:363` through `rewrite-in-rust/bootstrap/check_download_models_effectful_fetch_contract.py:408`. The Rust model accepts `unlink_error_paths`, keeps failed removals, and emits no success log at `rewrite-in-rust/rust/crates/v2m-core/src/download_models_effectful.rs:367` through `rewrite-in-rust/rust/crates/v2m-core/src/download_models_effectful.rs:395`; its fixture adapter reads `unlink_error` at `rewrite-in-rust/rust/crates/v2m-core/src/download_models_effectful.rs:850` through `rewrite-in-rust/rust/crates/v2m-core/src/download_models_effectful.rs:877`.
- Required fix: None.

- Location: rewrite-in-rust/fixtures/download_models_effectful_fetch_contract.jsonl:1
- Assessment: The rerun fixture table covers the declared mocked behavior surface and stays inside the confirmed unit boundary.
- Evidence: The manifest entry keeps the unit `reimplemented`, legacy-owned, and fixture-verified at `rewrite-in-rust/manifest.yaml:734` through `rewrite-in-rust/manifest.yaml:753`. The bootstrap boundary lists the covered helpers at `rewrite-in-rust/bootstrap/download_models_effectful_fetch_contract.md:5` through `rewrite-in-rust/bootstrap/download_models_effectful_fetch_contract.md:20` and excludes live network, package installation, subprocess execution, archive extraction, asset download, and model-weight execution at `rewrite-in-rust/bootstrap/download_models_effectful_fetch_contract.md:42` through `rewrite-in-rust/bootstrap/download_models_effectful_fetch_contract.md:44`. The Python checker runs every JSONL case at `rewrite-in-rust/bootstrap/check_download_models_effectful_fetch_contract.py:596` through `rewrite-in-rust/bootstrap/check_download_models_effectful_fetch_contract.py:606`; the Rust test consumes the same table at `rewrite-in-rust/rust/crates/v2m-core/src/download_models_effectful.rs:659` through `rewrite-in-rust/rust/crates/v2m-core/src/download_models_effectful.rs:705`.
- Required fix: None.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_download_models_effectful_fetch_contract.py`: passed, silent exit 0.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml download_models_effectful`: passed; `download_models_effectful::tests::download_models_effectful_fetch_fixtures_match` passed, with 1 matching test and 59 filtered out in `v2m-core`, plus 0 matching tests in `v2m_quant_bridge`.
- `git diff --check -- rewrite-in-rust/fixtures/download_models_effectful_fetch_contract.jsonl rewrite-in-rust/bootstrap/check_download_models_effectful_fetch_contract.py rewrite-in-rust/rust/crates/v2m-core/src/download_models_effectful.rs rewrite-in-rust/records/0038-confirm-download-models-effectful-fetch-boundary.md rewrite-in-rust/dependencies/download_models_effectful_fetch_contract.yaml rewrite-in-rust/bootstrap/download_models_effectful_fetch_contract.md rewrite-in-rust/manifest.yaml download_models.py`: passed.
- `rg -n "^[<]{7}|^[=]{7}$|^[>]{7}|[ \t]+$" rewrite-in-rust/fixtures/download_models_effectful_fetch_contract.jsonl rewrite-in-rust/bootstrap/check_download_models_effectful_fetch_contract.py rewrite-in-rust/rust/crates/v2m-core/src/download_models_effectful.rs rewrite-in-rust/records/0038-confirm-download-models-effectful-fetch-boundary.md rewrite-in-rust/dependencies/download_models_effectful_fetch_contract.yaml rewrite-in-rust/bootstrap/download_models_effectful_fetch_contract.md download_models.py`: no matches.
- Scoped effect scan for process, network, package-install, archive, temp-file, filesystem, and the new `url_error`/`timeout_error`/`unlink_error` paths: reviewed. Hits are the legacy Python owner, Python fake harness/patch points, docs, fixtures, and Rust state-model strings; the Rust unit does not perform live IO.
- Focused read completed for control-plane docs, manifest entry, record 0038, dependency/bootstrap docs, prior behavior report, error-tracing rerun, fixture JSONL, Python checker, Rust implementation, and `download_models.py`.

## Residual Risk

This behavior rerun proves fixture-backed parity only. Actual GitHub traffic, real streaming failures, package installation, external CLI execution, archive extraction, and model-weight inspection remain legacy-owned and out of scope per `rewrite-in-rust/dependencies/download_models_effectful_fetch_contract.yaml:43` through `rewrite-in-rust/dependencies/download_models_effectful_fetch_contract.yaml:45` and `rewrite-in-rust/bootstrap/download_models_effectful_fetch_contract.md:42` through `rewrite-in-rust/bootstrap/download_models_effectful_fetch_contract.md:44`.

A future production bridge still needs an explicit decision on whether to preserve or intentionally change the escaped non-HTTP stream-exception behavior before Rust owns live IO.

## Promotion Note

This behavior rerun does not block coordinator state update for `download_models_effectful_fetch_contract`. Runtime promotion is not implied: `download_models.py` remains the rollback/runtime owner, and record 0038 keeps production ownership unchanged at `rewrite-in-rust/records/0038-confirm-download-models-effectful-fetch-boundary.md:54` through `rewrite-in-rust/records/0038-confirm-download-models-effectful-fetch-boundary.md:57`.
