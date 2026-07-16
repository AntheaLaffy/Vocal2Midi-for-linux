# web_task_registry_contract - behavior_reviewer

Date: 2026-07-16
Decision: pass

## Findings

No behavior-parity findings.

The prior dependency follow-up for multi-task lookup/list coverage is now represented and passing. Fixture case `multi_task_lookup_and_list_order` creates two tasks with distinct IDs, timestamps, configs, and audio paths, asserts `get_task` hits for both IDs, asserts a missing lookup, and checks `list_tasks` summary order/shape (`rewrite-in-rust/fixtures/web_task_registry_contract.jsonl:8`). The Python checker exercises that case through `TaskManager.create_task`, `get_task`, and `list_tasks` (`rewrite-in-rust/bootstrap/check_web_task_registry_contract.py:140` through `rewrite-in-rust/bootstrap/check_web_task_registry_contract.py:153`). The Rust test consumes the same fixture path and checks the same hits, miss, and ordered summaries (`rewrite-in-rust/rust/crates/v2m-core/src/web_task.rs:152`, `rewrite-in-rust/rust/crates/v2m-core/src/web_task.rs:285` through `rewrite-in-rust/rust/crates/v2m-core/src/web_task.rs:299`).

Task creation defaults match the legacy source. Python creates UUID-backed pending tasks with progress `0`, stage `idle`, retained config/audio path, `created_at=datetime.now()`, no start/completion/error/thread, empty outputs/logs, and a clear stop event (`web_task_manager.py:58` through `web_task_manager.py:88`). The fixture locks those fields (`rewrite-in-rust/fixtures/web_task_registry_contract.jsonl:1`), the checker patches UUID/time/event/thread sources before invoking legacy Python (`rewrite-in-rust/bootstrap/check_web_task_registry_contract.py:64` through `rewrite-in-rust/bootstrap/check_web_task_registry_contract.py:92`), and Rust mirrors the injected values and defaults (`rewrite-in-rust/rust/crates/v2m-core/src/web_task.rs:58` through `rewrite-in-rust/rust/crates/v2m-core/src/web_task.rs:83`).

Start/stop behavior matches the legacy state machine. Python only starts pending tasks, assigns `started_at`, creates a daemon thread named `Pipeline-<first8>`, starts it, and only sets the stop event for running tasks (`web_task_manager.py:90` through `web_task_manager.py:131`). Fixtures cover pending start, missing start, non-pending start, running stop, pending stop, and missing stop (`rewrite-in-rust/fixtures/web_task_registry_contract.jsonl:2` through `rewrite-in-rust/fixtures/web_task_registry_contract.jsonl:7`). Rust applies the same pending/running guards, timestamp assignment, thread metadata, start flag, and stop flag (`rewrite-in-rust/rust/crates/v2m-core/src/web_task.rs:86` through `rewrite-in-rust/rust/crates/v2m-core/src/web_task.rs:115`).

Lookup/list behavior and rollback boundary match the reviewed policy. Python returns stored tasks or `None` under the manager lock and lists summaries containing only `id`, `status`, `progress`, `stage`, and ISO timestamp fields in registry insertion order (`web_task_manager.py:133` through `web_task_manager.py:163`). Rust returns the same summary fields in vector insertion order (`rewrite-in-rust/rust/crates/v2m-core/src/web_task.rs:117` through `rewrite-in-rust/rust/crates/v2m-core/src/web_task.rs:136`). Records 0019 and 0020 keep `_execute_pipeline`, `WebStreamRedirector`, live SocketIO, Flask routes, config mapping, output scanning, and model inference out of this unit (`rewrite-in-rust/records/0019-split-web-task-lifecycle-unit.md:24`, `rewrite-in-rust/records/0020-confirm-web-task-registry-boundary.md:15` through `rewrite-in-rust/records/0020-confirm-web-task-registry-boundary.md:28`). Production Web routes still call `web_task_manager.TaskManager` directly (`web_server.py:251` through `web_server.py:253`, `web_server.py:295` through `web_server.py:360`, `web_server.py:913` through `web_server.py:935`), and runtime ownership remains legacy.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_web_task_registry_contract.py`: passed.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml web_task`: passed; 2 `web_task` tests passed, 58 filtered out in `v2m_core`, and 0 matching tests ran in `v2m_quant_bridge`.
- `uv run pytest tests/test_web_api.py`: passed; 53 tests passed.
- `git diff --check`: passed.

## Residual Risk

This behavior gate does not review error tracing, Rust style, or promotion architecture. The registry model deliberately fakes thread scheduling and does not execute the stored thread target, so completion/cancel/failure events remain covered by `web_pipeline_execution_events`, not this unit.

## Promotion Note

This behavior role does not block coordinator state update. Keep runtime ownership legacy and do not mark the manifest verified from this report alone; the manifest still requires the separate `error_tracing_reviewer` gate for `web_task_registry_contract`.
