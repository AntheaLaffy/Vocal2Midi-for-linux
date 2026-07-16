# web_pipeline_execution_events - error_tracing_reviewer

Date: 2026-07-16
Decision: pass-with-followups

## Findings

- Severity: low
- Location: web_task_manager.py:185
- Issue: SocketIO emit failures are source-inspected but not represented in the fixture/Rust error model. Legacy log and progress emit failures are swallowed after printing a WebSocket error, terminal `status_change` emit failures are not locally swallowed and instead fall into the outer task-manager error path, and the final manager-error `status_change` fallback is swallowed if it also fails.
- Evidence: `log_callback` catches `socketio.emit('log', ...)` failures and prints at `web_task_manager.py:185`, and `progress_callback` does the same at `web_task_manager.py:193`. Terminal status emits at `web_task_manager.py:252`, `web_task_manager.py:265`, `web_task_manager.py:274`, and `web_task_manager.py:292` have no local `try`, so they are handled by the outer manager-error branch at `web_task_manager.py:303`; only the final fallback emit is swallowed at `web_task_manager.py:309`. The checker `FakeSocketIO.emit` always records success (`rewrite-in-rust/bootstrap/check_web_pipeline_execution_events.py:57`), the fixture table has no emit-failure row (`rewrite-in-rust/fixtures/web_pipeline_execution_events.jsonl:1`), and Rust has no emit-failure input variant (`rewrite-in-rust/rust/crates/v2m-core/src/web_pipeline_events.rs:89`).
- Required fix: Before any Rust-owned execution-event or SocketIO bridge promotion, either add explicit fake emit-failure fixtures covering log/progress swallow diagnostics, terminal status escalation to manager error, and final fallback swallowing, or narrow the promotion record to keep emit-failure diagnostics legacy/transport-owned.

- Severity: low
- Location: web_task_manager.py:303
- Issue: Manager-error traceback diagnostics are not part of the durable fake event trace. The current contract proves task status/error and fallback `status_change`, but it does not prove the console traceback printed by the outer task-manager error branch.
- Evidence: The manager-error branch sets `task.error`, prints `[Task Manager Error]` plus `traceback.format_exc()`, then emits failed status (`web_task_manager.py:303`). The `manager_config_error` fixture expects the task error and failed `status_change` but no console traceback payload (`rewrite-in-rust/fixtures/web_pipeline_execution_events.jsonl:6`). The Python checker installs fake stdout/stderr and asserts restoration (`rewrite-in-rust/bootstrap/check_web_pipeline_execution_events.py:289`, `rewrite-in-rust/bootstrap/check_web_pipeline_execution_events.py:302`) but never asserts `fake_stdout.writes`, and Rust models only the task error/status event for config errors (`rewrite-in-rust/rust/crates/v2m-core/src/web_pipeline_events.rs:141`).
- Required fix: Before Rust owns caller-facing diagnostics for this path, add a manager-error console-trace fixture or explicitly document that only `task.error` plus failed `status_change` is the Rust contract while traceback console output remains legacy-owned.

## Checks

- `env PYTHONDONTWRITEBYTECODE=1 UV_CACHE_DIR=/tmp/v2m-uv-cache uv run python rewrite-in-rust/bootstrap/check_web_pipeline_execution_events.py`: passed with no output.
- `env CARGO_TARGET_DIR=/tmp/v2m-web-pipeline-events-error-target cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml web_pipeline_events`: passed; 1 `web_pipeline_events` test passed in `v2m-core`, 59 filtered out, and 0 matching tests in `v2m-quant-bridge`.
- `env PYTHONDONTWRITEBYTECODE=1 UV_CACHE_DIR=/tmp/v2m-uv-cache uv run pytest -p no:cacheprovider tests/test_web_api.py`: passed; 53 tests.
- `rg -n "traceback|format_exc|emit\\(|SocketIO|WebSocket|except Exception|KeyboardInterrupt|stdout|stderr|cancel|manager error|Task Manager Error|status_change" web_task_manager.py rewrite-in-rust/bootstrap/check_web_pipeline_execution_events.py rewrite-in-rust/rust/crates/v2m-core/src/web_pipeline_events.rs rewrite-in-rust/fixtures/web_pipeline_execution_events.jsonl rewrite-in-rust/bootstrap/web_pipeline_execution_events.md rewrite-in-rust/dependencies/web_pipeline_execution_events.yaml`: reviewed targeted error/log/emit/restoration paths.
- `git diff --check`: passed.
- `git status --short`: reviewed existing unrelated dirty rewrite artifacts; this review wrote only `rewrite-in-rust/reviews/2026-07-16-web_pipeline_execution_events-error_tracing_reviewer.md`.

## Residual Risk

Raw exception strings and traceback text are intentionally forwarded in the legacy generic-failure path (`web_task_manager.py:286`, `web_task_manager.py:290`) and are fixture-proven only with synthetic messages (`rewrite-in-rust/fixtures/web_pipeline_execution_events.jsonl:5`). This review treats that as parity, not as a redaction guarantee. Real SocketIO transport, Flask routes, task registry concurrency, `WebStreamRedirector` internals, and real model inference stay outside this unit by record 0022.

## Promotion Note

This error-tracing role does not block coordinator state update for the current legacy-owned fixture contract. Do not use it as evidence for a Rust-owned SocketIO transport or caller-facing diagnostic bridge until the low follow-ups above are closed or explicitly kept out of the promoted boundary. Runtime ownership remains `legacy`; do not mark the manifest verified from this report alone.
