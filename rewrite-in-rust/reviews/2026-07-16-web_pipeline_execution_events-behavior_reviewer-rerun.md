# web_pipeline_execution_events - behavior_reviewer

Date: 2026-07-16
Decision: pass

## Findings

No behavior blockers found in the rerun.

The previous stdout/stderr restoration blocker is fixed. The Python checker now
asserts `stdout_restored` and `stderr_restored` from the fixture after
`TaskManager._execute_pipeline` returns, and the Rust outcome/test surface
asserts the same fixture fields (`rewrite-in-rust/bootstrap/check_web_pipeline_execution_events.py:302`,
`rewrite-in-rust/rust/crates/v2m-core/src/web_pipeline_events.rs:80`,
`rewrite-in-rust/rust/crates/v2m-core/src/web_pipeline_events.rs:729`).

The previous SocketIO payload/order blocker is fixed. The checker compares one
ordered `emit_events` trace preserving event name, room, `task_id`, log
timestamp, progress/status payload keys, result payloads, and omitted-vs-null
status `error` distinctions (`rewrite-in-rust/bootstrap/check_web_pipeline_execution_events.py:109`,
`rewrite-in-rust/bootstrap/check_web_pipeline_execution_events.py:224`,
`rewrite-in-rust/bootstrap/check_web_pipeline_execution_events.py:347`). The
Rust model now carries a single ordered `socket_events` vector and serializes
the same payload shape for fixture comparison
(`rewrite-in-rust/rust/crates/v2m-core/src/web_pipeline_events.rs:41`,
`rewrite-in-rust/rust/crates/v2m-core/src/web_pipeline_events.rs:63`,
`rewrite-in-rust/rust/crates/v2m-core/src/web_pipeline_events.rs:552`,
`rewrite-in-rust/rust/crates/v2m-core/src/web_pipeline_events.rs:752`).

The previous multi-file output ordering blocker is fixed for this behavior
gate. The completed fixture uses multiple files per collected extension, the
Python checker compares exact `task.output_files`, and the Rust test compares
the same ordered fixture output (`rewrite-in-rust/fixtures/web_pipeline_execution_events.jsonl:1`,
`rewrite-in-rust/bootstrap/check_web_pipeline_execution_events.py:331`,
`rewrite-in-rust/rust/crates/v2m-core/src/web_pipeline_events.rs:417`,
`rewrite-in-rust/rust/crates/v2m-core/src/web_pipeline_events.rs:739`). The
legacy source still collects by extension group order `.mid`, `.ustx`, `.txt`,
then `.csv` (`web_task_manager.py:241`).

## Scope Confirmed

- Unit and role: reviewed exactly `web_pipeline_execution_events` as
  `behavior_reviewer`.
- Boundary: record 0022 keeps the unit limited to
  `TaskManager._execute_pipeline` event/state behavior and excludes task
  registry behavior, `WebStreamRedirector` internals, Web config mapping, Flask
  routes, real SocketIO transport, and real model inference
  (`rewrite-in-rust/records/0022-confirm-web-pipeline-execution-events-boundary.md:17`).
- Rollback remains keeping `web_task_manager.TaskManager._execute_pipeline` as
  the production owner, with no Rust bridge introduced for this unit
  (`rewrite-in-rust/bootstrap/web_pipeline_execution_events.md:86`).

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_web_pipeline_execution_events.py`:
  passed with no output.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml web_pipeline_events`:
  passed; 1 `web_pipeline_events` test passed in `v2m-core`, 46 filtered out,
  and 0 matching tests in `v2m-quant-bridge`.
- Inspected
  `rewrite-in-rust/fixtures/web_pipeline_execution_events.jsonl`,
  `rewrite-in-rust/bootstrap/check_web_pipeline_execution_events.py`,
  `rewrite-in-rust/rust/crates/v2m-core/src/web_pipeline_events.rs`,
  `web_task_manager.py`, record 0022, the dependency record, bootstrap doc, and
  manifest entry for this unit.

## Residual Risk

Real SocketIO transport, real model inference, Flask routes, task registry
lifecycle, Web config mapping, and `WebStreamRedirector` internals remain
outside this unit by record 0022. Error behavior for failed SocketIO emits is
also intentionally left to the required error-tracing review.

## Promotion Note

This behavior rerun does not block coordinator verification for
`web_pipeline_execution_events`. The coordinator should still wait for any
other required role reviews before marking the unit verified.
