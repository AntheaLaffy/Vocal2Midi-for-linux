# web_task_registry_contract - error_tracing_reviewer

Date: 2026-07-16
Decision: pass-with-followups

## Findings

- Severity: low
- Location: web_task_manager.py:100
- Issue: Concurrent `start_task`/`stop_task` transition diagnostics are not defined by the current fake-thread contract. `get_task` and `list_tasks` take `_lock`, but `start_task` releases the lock after lookup and mutates `status`, `started_at`, and `thread` outside the lock; `stop_task` likewise mutates `stop_event` after lookup. The Rust model uses `&mut self`, so the tested contract proves sequential missing/non-pending and missing/non-running outcomes, but it does not prove or document what a future bridge should report if two callers race the same task.
- Evidence: `web_task_manager.py:100` through `web_task_manager.py:115` and `web_task_manager.py:127` through `web_task_manager.py:130`; Rust transitions return only `bool` at `rewrite-in-rust/rust/crates/v2m-core/src/web_task.rs:87` and `rewrite-in-rust/rust/crates/v2m-core/src/web_task.rs:106`; fixtures cover sequential cases in `rewrite-in-rust/fixtures/web_task_registry_contract.jsonl:2`, `:4`, `:5`, `:6`, and `:7`.
- Required fix: Before promoting a Rust-owned task registry, define the lock/atomicity contract for concurrent start/stop calls and add either bridge-level synchronization evidence or explicit fixtures documenting the accepted behavior. This does not block the current legacy-owned registry contract.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_web_task_registry_contract.py`: passed.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml web_task`: passed, 2 tests.
- `env PYTHONDONTWRITEBYTECODE=1 UV_CACHE_DIR=/tmp/v2m-uv-cache uv run pytest -p no:cacheprovider tests/test_web_api.py`: passed, 53 tests.
- `rg -n "return False|return None|raise|except|print|logger|traceback|error|logs|stop_event|threading.Lock|with self\\._lock|Thread\\(|Event\\(|Task not found|cannot be stopped|Failed to start task" web_task_manager.py web_server.py rewrite-in-rust/bootstrap/check_web_task_registry_contract.py rewrite-in-rust/rust/crates/v2m-core/src/web_task.rs rewrite-in-rust/fixtures/web_task_registry_contract.jsonl`: inspected registry error/log/lock paths.

## Residual Risk

The reviewed boundary intentionally excludes `_execute_pipeline`, SocketIO delivery, stream redirection, output scanning, and model runtime failures. Within the registry-only scope, missing start, non-pending start, missing stop, non-running stop, and `get_task` miss are represented as legacy-compatible `False`/`None` results, while route-level status/stop calls map misses and non-running stop attempts to JSON errors without exposing config or audio paths. Registry operations do not append task logs; the only fixture-level log assertion is the empty-log creation default.

Path/config redaction risk is low for this unit because `list_tasks` exposes only summary fields and the stop/status miss errors are fixed strings or status-only messages. A future direct Rust bridge should still avoid exposing the internal full task/config/audio-path model as an external diagnostic surface unless a separate API contract permits it.

## Promotion Note

This role does not block coordinator use of the report as current legacy-owned verification evidence. Runtime ownership should remain `legacy`; do not promote the Rust registry or mark the manifest from this reviewer pass alone.
