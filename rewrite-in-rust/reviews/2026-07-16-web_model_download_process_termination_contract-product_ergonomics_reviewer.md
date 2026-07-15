# web_model_download_process_termination_contract - product_ergonomics_reviewer

Date: 2026-07-16
Decision: pass

## Findings

No findings.

## Evidence

- The public stop contract says `/api/models/download/stop` requests cancellation and terminates the child download process group when possible; it returns `400` for missing or non-stoppable tasks and `404` for missing download tasks (`docs/web-api.md:310`, `docs/web-api.md:312`, `docs/web-api.md:327`, `docs/web-api.md:328`).
- Legacy `ModelDownloadManager.stop_task` preserves the user-facing stop workflow: running or pending tasks move to `stopping`, the stop event is set, a live process invokes `_terminate_process_tree`, and `OSError` maps the stop request to `False` after the state/event update (`web_model_download_manager.py:217`, `web_model_download_manager.py:221`, `web_model_download_manager.py:222`, `web_model_download_manager.py:225`, `web_model_download_manager.py:227`, `web_model_download_manager.py:228`).
- Legacy process termination behavior covers already-exited no-op, Windows `taskkill /T` plus `/F`, POSIX SIGTERM/SIGKILL process-group targeting, ProcessLookupError swallowing, and terminate/kill fallbacks (`web_model_download_manager.py:333`, `web_model_download_manager.py:337`, `web_model_download_manager.py:340`, `web_model_download_manager.py:341`, `web_model_download_manager.py:342`, `web_model_download_manager.py:359`, `web_model_download_manager.py:361`, `web_model_download_manager.py:362`, `web_model_download_manager.py:364`).
- The termination fixtures directly cover the UX-critical live-process stop paths: success sets `status: stopping`, sets the stop event, and records a termination request; termination `OSError` returns `success: false` while preserving the same state/event update (`rewrite-in-rust/fixtures/web_model_download_process_termination_contract.jsonl:11`, `rewrite-in-rust/fixtures/web_model_download_process_termination_contract.jsonl:12`).
- The Rust unit mirrors those fixture-backed semantics: it declares that it never sends signals, invokes `taskkill`, or terminates a real process; `stop_task_with_process` sets `stopping`, sets the fake stop event, records a live-process termination call, returns failure on the injected termination error, and otherwise succeeds (`rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_termination.rs:1`, `rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_termination.rs:3`, `rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_termination.rs:127`, `rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_termination.rs:128`, `rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_termination.rs:132`, `rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_termination.rs:135`, `rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_termination.rs:138`, `rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_termination.rs:143`).
- The unit does not overclaim production ownership: bootstrap excludes Web route mapping, real process termination, and SocketIO delivery; record 0034 keeps production ownership in legacy Python until a later promotion record (`rewrite-in-rust/bootstrap/web_model_download_process_termination_contract.md:30`, `rewrite-in-rust/bootstrap/web_model_download_process_termination_contract.md:32`, `rewrite-in-rust/bootstrap/web_model_download_process_termination_contract.md:45`, `rewrite-in-rust/bootstrap/web_model_download_process_termination_contract.md:59`, `rewrite-in-rust/bootstrap/web_model_download_process_termination_contract.md:85`, `rewrite-in-rust/records/0034-confirm-web-model-download-process-termination-boundary.md:35`, `rewrite-in-rust/records/0034-confirm-web-model-download-process-termination-boundary.md:39`, `rewrite-in-rust/records/0034-confirm-web-model-download-process-termination-boundary.md:44`).

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_web_model_download_process_termination_contract.py`: pass
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml web_model_download_termination`: pass
- `uv run pytest tests/test_web_api.py -k "ModelDownload and (stop or terminate)" -q`: pass, 4 passed
- `git diff --check -- rewrite-in-rust/fixtures/web_model_download_process_termination_contract.jsonl rewrite-in-rust/bootstrap/check_web_model_download_process_termination_contract.py rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_termination.rs rewrite-in-rust/bootstrap/web_model_download_process_termination_contract.md rewrite-in-rust/dependencies/web_model_download_process_termination_contract.yaml rewrite-in-rust/records/0034-confirm-web-model-download-process-termination-boundary.md rewrite-in-rust/manifest.yaml`: pass
- `uv run python rewrite-in-rust/bootstrap/check_web_model_download_request_catalog_contract.py`: pass, route-mapping context only
- `cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check`: pass
- Manual legacy HTTP probe with `_terminate_process_tree` patched to raise `OSError`: returned `400`, body `{"success": false, "error": "Download task cannot be stopped (current status: stopping)"}`, and task state remained `{"status": "stopping", "stop_event_set": true}`.

## Residual Risk

The unit proves branch decisions and manager-level user-visible stop state with fake process APIs. It does not prove real OS process death, signal delivery timing, or Windows `taskkill` availability, and it should not be used as evidence that production process killing has moved to Rust.

SocketIO `stop_response` and room `status_change` delivery remain legacy Web behavior. The handler delegates to `model_download_manager.stop_task` and emits the documented response shape (`web_server.py:904`, `web_server.py:916`, `web_server.py:919`, `web_server.py:937`), but this termination unit correctly excludes SocketIO delivery from its claim.

## Promotion Note

This product ergonomics role does not block coordinator state update for `web_model_download_process_termination_contract`. Runtime ownership should remain legacy Python until a later bridge/promotion record verifies real Web integration and process termination behavior.
