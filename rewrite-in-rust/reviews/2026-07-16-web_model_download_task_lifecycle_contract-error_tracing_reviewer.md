# web_model_download_task_lifecycle_contract - error_tracing_reviewer

Date: 2026-07-16
Decision: pass-with-followups

## Findings

- Severity: medium
- Location: web_model_download_manager.py:209
- Issue: `start_task` sets `active_task_id`, marks the task `running`, and records `started_at` before calling `task.thread.start()`. If `thread.start()` raises, the exception escapes after the manager has already left a task in active/running state. The Flask route catches every `RuntimeError` from `start_task` as a `409` start conflict, so a thread-start resource failure can be reported through the same boundary as "A model download task is already running." The Rust lifecycle model and fixtures only cover active conflict and successful fake start metadata, so this diagnostic/stale-state branch is not represented.
- Evidence: `start_task` assigns active/running state at `web_model_download_manager.py:209` through `web_model_download_manager.py:214`; the route maps any `RuntimeError` to `409` at `web_server.py:701` through `web_server.py:705`; the Rust model only returns the active-conflict string at `rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_lifecycle.rs:179` through `rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_lifecycle.rs:185` and otherwise creates successful thread metadata at `rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_lifecycle.rs:203` through `rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_lifecycle.rs:212`. A fake-thread probe that raises from `start()` produced `raised=can't start new thread`, a non-null `active_task_id`, and `task_status=running` without running the target.
- Required fix: Before promoting runtime ownership or claiming this lifecycle as an error-complete replacement, add a fixture or explicit record for `Thread.start` failure. Prefer separating the active-conflict error from thread/create failures and rolling back or marking failed state when the thread cannot start.

- Severity: medium
- Location: web_model_download_manager.py:187
- Issue: Active conflict detection is not atomic with task creation and active-task assignment. `start_task` checks the current active task under the lock, releases the lock while creating the task and thread object, then reacquires it to assign `active_task_id`. Two concurrent callers can both pass the active check and start two running tasks. The Rust model takes `&mut self` and the fixture harness is single-threaded, so this lock/concurrency risk is hidden by the contract.
- Evidence: The legacy check happens at `web_model_download_manager.py:187` through `web_model_download_manager.py:190`, while task creation and the later active assignment happen outside that same critical section at `web_model_download_manager.py:192` through `web_model_download_manager.py:214`. Record 0032 says fixture parity injects fake thread/event state, but does not claim concurrent-start behavior (`rewrite-in-rust/records/0032-confirm-web-model-download-task-lifecycle-boundary.md:41` through `rewrite-in-rust/records/0032-confirm-web-model-download-task-lifecycle-boundary.md:44`). A barrier probe with fake threads returned `results=2 errors=[]` and two `running` tasks after two concurrent starts.
- Required fix: Before any manager replacement or promotion to a Rust-owned lifecycle, make the conflict check, task registration, and active assignment one atomic manager operation, or explicitly document and fixture the legacy race as accepted behavior.

## Evidence

The active-conflict diagnostic string is preserved for the single-threaded contract. Legacy raises `RuntimeError("A model download task is already running.")` at `web_model_download_manager.py:187` through `web_model_download_manager.py:190`; the fixture expects the same string at `rewrite-in-rust/fixtures/web_model_download_task_lifecycle_contract.jsonl:7`; Rust returns the same string at `rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_lifecycle.rs:184` through `rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_lifecycle.rs:185`.

Missing and stopped-state behavior is otherwise preserved at the manager-state level. Legacy `stop_task` returns `False` for missing tasks and statuses outside `pending`/`running` at `web_model_download_manager.py:217` through `web_model_download_manager.py:220`; Rust matches that branch at `rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_lifecycle.rs:215` through `rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_lifecycle.rs:222`. Fixtures cover missing/completed stop results and no-process pending/running transitions at `rewrite-in-rust/fixtures/web_model_download_task_lifecycle_contract.jsonl:9` through `rewrite-in-rust/fixtures/web_model_download_task_lifecycle_contract.jsonl:10`.

The fake thread/event boundary is respected. The bootstrap checker replaces `Event` and `Thread`, and its fake `start()` only marks metadata without invoking the target (`rewrite-in-rust/bootstrap/check_web_model_download_task_lifecycle_contract.py:25` through `rewrite-in-rust/bootstrap/check_web_model_download_task_lifecycle_contract.py:45`). The fixture asserts `target_called: false` for successful starts at `rewrite-in-rust/fixtures/web_model_download_task_lifecycle_contract.jsonl:6`, and the Rust metadata stores `target_called: false` at `rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_lifecycle.rs:203` through `rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_lifecycle.rs:208`.

Route mapping remains outside this unit's Rust implementation. The manifest keeps `current_owner: legacy` and rollback through `web_model_download_manager.ModelDownloadManager` at `rewrite-in-rust/manifest.yaml:642` through `rewrite-in-rust/manifest.yaml:663`; the Rust crate is not wired into the Python runtime (`rewrite-in-rust/rust/crates/v2m-core/src/lib.rs:1` through `rewrite-in-rust/rust/crates/v2m-core/src/lib.rs:5`). The route-level not-found/cannot-stop strings remain in `web_server.py:742` through `web_server.py:759`, while request/catalog route mapping is reviewed separately.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_web_model_download_task_lifecycle_contract.py`: pass
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml web_model_download_lifecycle`: pass, 1 matched test passed
- `uv run pytest tests/test_web_api.py -k ModelDownload -q`: pass, 19 passed and 34 deselected
- `git diff --check`: pass
- `git diff -- rewrite-in-rust/records/0031-split-web-model-download-lifecycle-termination.md rewrite-in-rust/records/0032-confirm-web-model-download-task-lifecycle-boundary.md rewrite-in-rust/dependencies/web_model_download_task_lifecycle_contract.yaml rewrite-in-rust/bootstrap/web_model_download_task_lifecycle_contract.md rewrite-in-rust/fixtures/web_model_download_task_lifecycle_contract.jsonl rewrite-in-rust/bootstrap/check_web_model_download_task_lifecycle_contract.py rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_lifecycle.rs rewrite-in-rust/rust/crates/v2m-core/src/lib.rs web_model_download_manager.py web_server.py tests/test_web_api.py`: no source/fixture/Rust diff for this unit before this report
- Manual fake `Thread.start` failure probe: exposed stale active/running task state without running the target
- Manual concurrent-start barrier probe with fake threads: exposed two running tasks and no active-conflict error without running download targets

## Residual Risk

This review does not prove real OS thread creation, real Flask request concurrency, SocketIO delivery, subprocess execution, process termination, network downloads, package installation, archive extraction, or model marker safety. Those remain legacy-owned or assigned to adjacent model-download units. `create_task` failures from UUID, clock, event construction, or lock/runtime errors are not fixture-backed; most would surface through the legacy Flask error handler today.

## Promotion Note

This role does not block coordinator verification for `web_model_download_task_lifecycle_contract` while runtime ownership remains legacy Python and no Rust bridge is introduced. The coordinator may mark this error-tracing role satisfied as `pass-with-followups`. The two findings should block any later promotion that claims Rust-owned model-download lifecycle management or concurrency-safe start diagnostics until they are fixed, explicitly recorded as accepted legacy behavior, or covered by fixtures.
