# web_model_download_request_catalog_contract - error_tracing_reviewer

Date: 2026-07-16
Decision: pass-with-followups

## Findings

- Severity: low
- Location: rewrite-in-rust/rust/crates/v2m-core/src/web_model_download.rs:195
- Issue: Proxy URL redaction differs from legacy for the malformed but currently accepted empty-scheme shape `://user:pass@host`. Legacy splits on `://` but only preserves the prefix when `scheme` is truthy, returning `***@host`; Rust enters the `split_once("://")` branch and always formats `{scheme}://***@{host}`, returning `://***@host`.
- Evidence: Legacy `_redact_proxy_url` conditionally includes the scheme at `web_model_download_manager.py:540` and `web_model_download_manager.py:542`. Rust redaction unconditionally includes the separator in the split branch at `rewrite-in-rust/rust/crates/v2m-core/src/web_model_download.rs:195` and `rewrite-in-rust/rust/crates/v2m-core/src/web_model_download.rs:197`. Manual proxy validation only checks for the substring `://`, so this malformed shape is not rejected before task creation (`web_model_download_manager.py:519`, `web_model_download_manager.py:524`; `rewrite-in-rust/rust/crates/v2m-core/src/web_model_download.rs:227`, `rewrite-in-rust/rust/crates/v2m-core/src/web_model_download.rs:234`).
- Required fix: Add a redaction fixture or unit assertion for empty-scheme credential URLs and either match the legacy `scheme` truthiness branch or intentionally tighten proxy validation in a separate contract decision. This does not expose credentials and does not block coordinator verification for the current legacy-owned unit.

## Evidence

Validation error shape is preserved for the public request contract. The legacy route returns `{"success": false, "error": ...}` with status `400` for invalid model lists and validation strings (`web_server.py:666`, `web_server.py:690`), and Rust uses the same shaped `start_error` helper (`rewrite-in-rust/rust/crates/v2m-core/src/web_model_download.rs:401`, `rewrite-in-rust/rust/crates/v2m-core/src/web_model_download.rs:409`). Fixtures cover invalid model list shape, non-string list entries, empty selections, unknown ids, invalid `qwen_source`, invalid `proxy_mode`, missing manual proxy URL, and missing proxy scheme (`rewrite-in-rust/fixtures/web_model_download_request_catalog_contract.jsonl:9`, `rewrite-in-rust/fixtures/web_model_download_request_catalog_contract.jsonl:16`).

Normal credential-bearing proxy redaction is safe. Active-task serialization redacts `http://user:pass@127.0.0.1:7890` to `http://***@127.0.0.1:7890` in the durable fixture (`rewrite-in-rust/fixtures/web_model_download_request_catalog_contract.jsonl:2`). Legacy serialization routes all user-visible task `proxy_url` values through `_redact_proxy_url` (`web_model_download_manager.py:232`, `web_model_download_manager.py:243`), and Rust does the same through `redact_proxy_url` (`rewrite-in-rust/rust/crates/v2m-core/src/web_model_download.rs:165`, `rewrite-in-rust/rust/crates/v2m-core/src/web_model_download.rs:177`). The Rust redaction unit test covers empty, no-credential, ordinary credential, and no-scheme credential strings (`rewrite-in-rust/rust/crates/v2m-core/src/web_model_download.rs:592`, `rewrite-in-rust/rust/crates/v2m-core/src/web_model_download.rs:602`).

Conflict, not-found, and stop diagnostics preserve the legacy route strings. Start conflicts are mapped from `RuntimeError` to `409` with the original string (`web_server.py:701`, `web_server.py:705`; `rewrite-in-rust/rust/crates/v2m-core/src/web_model_download.rs:317`, `rewrite-in-rust/rust/crates/v2m-core/src/web_model_download.rs:324`; fixture line `rewrite-in-rust/fixtures/web_model_download_request_catalog_contract.jsonl:17`). Status lookup not-found returns `Download task not found` with `404` (`web_server.py:719`, `web_server.py:723`; `rewrite-in-rust/rust/crates/v2m-core/src/web_model_download.rs:329`, `rewrite-in-rust/rust/crates/v2m-core/src/web_model_download.rs:337`; fixture line `rewrite-in-rust/fixtures/web_model_download_request_catalog_contract.jsonl:18`). Stop missing-id, not-found, cannot-stop, and success paths are all explicitly fixture-backed (`rewrite-in-rust/fixtures/web_model_download_request_catalog_contract.jsonl:20`, `rewrite-in-rust/fixtures/web_model_download_request_catalog_contract.jsonl:23`) and match route strings at `web_server.py:739`, `web_server.py:745`, `web_server.py:753`, `web_server.py:759` and Rust mappings at `rewrite-in-rust/rust/crates/v2m-core/src/web_model_download.rs:351`, `rewrite-in-rust/rust/crates/v2m-core/src/web_model_download.rs:399`.

Unexpected subprocess/runtime exceptions are outside this unit rather than hidden. Record 0029 excludes subprocess execution, output parsing, SocketIO failures, active-task locking, termination, real downloads, network access, and archive safety (`rewrite-in-rust/records/0029-split-web-model-download-contract.md:37`, `rewrite-in-rust/records/0029-split-web-model-download-contract.md:45`). The request/catalog bootstrap repeats that no live Web server, SocketIO transport, model runtime, download process, network, or archive extraction is needed (`rewrite-in-rust/bootstrap/web_model_download_request_catalog_contract.md:58`, `rewrite-in-rust/bootstrap/web_model_download_request_catalog_contract.md:60`). The legacy manager's later execution exception path stores `str(exc)` and a traceback log on the task (`web_model_download_manager.py:303`, `web_model_download_manager.py:310`), which is reviewed by the separate execution-result unit, not this route/request unit.

The runtime owner remains legacy. The manifest marks this unit `reimplemented`, keeps `current_owner: legacy`, and names the Python routes/manager/download metadata as the rollback path (`rewrite-in-rust/manifest.yaml:588`, `rewrite-in-rust/manifest.yaml:609`). The Rust module also documents that Flask, SocketIO, subprocesses, real marker checks, and downloads stay with legacy Python (`rewrite-in-rust/rust/crates/v2m-core/src/web_model_download.rs:1`, `rewrite-in-rust/rust/crates/v2m-core/src/web_model_download.rs:6`).

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_web_model_download_request_catalog_contract.py`: pass
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml web_model_download`: pass, 7 matching tests passed and 53 filtered out in `v2m-core`
- `uv run pytest tests/test_web_api.py::TestModelDownloadAPI -q`: pass, 12 passed
- `uv run pytest tests/test_web_api.py -k ModelDownload -q`: pass, 19 passed and 34 deselected
- `git diff --check`: pass

## Residual Risk

Truthier non-object JSON request bodies remain intentionally weakly specified: legacy Flask can fall into an unexpected exception path after `request.get_json(silent=True) or {}`, while the Rust model returns a structured `400`. The behavior rerun accepted this for the documented object-body API after adding falsey `null` and empty-array coverage. This review does not reclassify that accepted residual as an error-tracing blocker.

Task `error` and `logs` fields are serialized as legacy stores them. This request/catalog unit proves route-level response shape and proxy-field redaction, not full traceback body sanitization or log-message redaction for process execution failures.

## Promotion Note

This role is satisfied with a low follow-up. The empty-scheme proxy redaction edge is non-blocking because it does not leak credentials, is outside the documented proxy URL shape, and runtime ownership remains legacy. The coordinator may mark the `error_tracing_reviewer` role satisfied for `web_model_download_request_catalog_contract`; other required review roles remain separate gates.
