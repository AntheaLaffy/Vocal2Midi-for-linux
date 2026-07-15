# web_task_registry_contract - dependency_bootstrap_reviewer

Date: 2026-07-15
Decision: pass-with-followups

## Findings

- Severity: low
- Location: rewrite-in-rust/fixtures/web_task_registry_contract.jsonl:1
- Issue: The fixture table proves single-task creation/listing plus missing start/stop and pending/running transitions, but it does not include a multi-task registry case or an explicit missing `get_task` case. The manifest and bootstrap include `get_task` and `list_tasks` in the public surface, and both Python and Rust expose registry lookup/list behavior (`web_task_manager.py:133`, `web_task_manager.py:145`, `rewrite-in-rust/rust/crates/v2m-core/src/web_task.rs:117`, `rewrite-in-rust/rust/crates/v2m-core/src/web_task.rs:122`).
- Evidence: `rewrite-in-rust/bootstrap/web_task_registry_contract.md:27` says `get_task` returns the stored task or `None`, and `rewrite-in-rust/bootstrap/web_task_registry_contract.md:28` covers list summaries. Current fixtures are one created task or missing start/stop only (`rewrite-in-rust/fixtures/web_task_registry_contract.jsonl:1`, `rewrite-in-rust/fixtures/web_task_registry_contract.jsonl:3`, `rewrite-in-rust/fixtures/web_task_registry_contract.jsonl:7`).
- Required fix: Add a follow-up fixture/checker case before promotion that creates at least two tasks with distinct IDs/timestamps/configs, asserts lookup by ID and missing lookup, and asserts `list_tasks` summary order/shape for both tasks.

## Scope Assessment

Manifest unit boundary: confirmed. Records 0019 and 0020 justify splitting the broad lifecycle unit into registry, stream redirector, and execution-events units, and this unit is limited to `Task` defaults, `create_task`, `start_task`, `stop_task`, `get_task`, and `list_tasks` (`rewrite-in-rust/records/0019-split-web-task-lifecycle-unit.md:24`, `rewrite-in-rust/records/0020-confirm-web-task-registry-boundary.md:15`).

Capability coverage is otherwise sufficient for dependency bootstrap. The dependency record covers creation defaults, pending-only start, running-only stop, lookup/listing, and fake thread metadata (`rewrite-in-rust/dependencies/web_task_registry_contract.yaml:3`). The bootstrap excludes `_execute_pipeline`, `WebStreamRedirector`, SocketIO, Flask routes, config mapping, output scanning, and model inference (`rewrite-in-rust/bootstrap/web_task_registry_contract.md:31`).

Kept-legacy decisions are appropriate. Real Python threading, Flask/SocketIO transport, stream redirection, and pipeline execution remain Python-owned until separate units review them (`rewrite-in-rust/dependencies/web_task_registry_contract.yaml:35`).

The seam choice is appropriate: independent Rust library, legacy runtime owner, and no bridge dependencies (`rewrite-in-rust/bootstrap/web_task_registry_contract.md:49`). The hand-written Rust model is justified because UUID generation, wall-clock time, and thread scheduling are environmental inputs that the harness injects (`rewrite-in-rust/records/0020-confirm-web-task-registry-boundary.md:9`).

No missing production crate risk found for this unit. The Rust model avoids UUID/time/thread crates by accepting injected task IDs and timestamps; `serde_json` is only a dev dependency for fixture parsing (`rewrite-in-rust/rust/crates/v2m-core/Cargo.toml:12`).

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_web_task_registry_contract.py`: passed.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml web_task`: passed; 2 `web_task` tests passed. The command also emitted an unrelated warning from `web_stream.rs:81` after current workspace changes exported that module.
- `uv run pytest tests/test_web_api.py`: passed; 53 tests passed.
- Current git diff/status inspected. The working tree includes active unrelated/unreviewed changes and untracked Web stream artifacts; this review did not modify them.

## Residual Risk

This role did not perform behavior, error-tracing, Rust style, or architecture review. The remaining dependency-bootstrap risk is fixture breadth around multi-task registry ordering/lookup and the impossible-in-practice UUID collision edge created by injected IDs.

## Promotion Note

This role does not block coordinator state update once the low follow-up is accepted for the next fixture pass. Runtime ownership and rollback stay with `web_task_manager.TaskManager`; do not mark the manifest verified from this report alone.
