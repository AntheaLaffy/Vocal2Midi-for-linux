# web_pipeline_execution_events - dependency_bootstrap_reviewer

Date: 2026-07-16
Decision: pass-with-followups

## Findings

- Severity: medium
- Location: rewrite-in-rust/bootstrap/check_web_pipeline_execution_events.py:105
- Issue: The fake SocketIO fixture does not prove the full event payload surface that this unit claims. The checker reduces task logs to message/level, progress events to progress/stage, and status changes to status/error/output_dir/files, while only asserting the SocketIO room. That drops `task_id` from log/progress/status payloads and drops log timestamp values even though the manifest says this unit preserves log/progress/status_change SocketIO payloads. The Rust event structs mirror the reduced shape, so a future Rust event model could lose payload identity or timestamp behavior and still pass these fixtures.
- Evidence: `rewrite-in-rust/manifest.yaml:486` includes SocketIO payload preservation in the public policy. Legacy Python builds log payloads with `task_id`, `message`, `level`, and `timestamp` at `web_task_manager.py:177`, progress payloads with `task_id` at `web_task_manager.py:194`, completed status payloads with `task_id` at `web_task_manager.py:252`, and failed/cancelled status payloads with `task_id` at `web_task_manager.py:292`. The checker normalizes away those fields at `rewrite-in-rust/bootstrap/check_web_pipeline_execution_events.py:105`, `rewrite-in-rust/bootstrap/check_web_pipeline_execution_events.py:109`, and `rewrite-in-rust/bootstrap/check_web_pipeline_execution_events.py:120`; it only checks rooms at `rewrite-in-rust/bootstrap/check_web_pipeline_execution_events.py:160`. The Rust structs likewise omit payload identity/timestamp fields at `rewrite-in-rust/rust/crates/v2m-core/src/web_pipeline_events.rs:7`, `rewrite-in-rust/rust/crates/v2m-core/src/web_pipeline_events.rs:14`, and `rewrite-in-rust/rust/crates/v2m-core/src/web_pipeline_events.rs:21`. Record 0022 specifically calls for fake SocketIO and fake clock coverage at `rewrite-in-rust/records/0022-confirm-web-pipeline-execution-events-boundary.md:20`.
- Required fix: Before promotion, extend the fixture/checker/Rust model to assert `task_id` on log/progress/status_change payloads and deterministic log timestamp behavior from the fake clock, or explicitly narrow the unit documentation to state that payload identity and timestamps remain legacy-owned and are outside this Rust model.

## Scope Assessment

Manifest unit boundary: confirmed. The unit should not be split, merged, deferred, or replaced for this dependency/bootstrap gate.

Record 0019 justifies replacing the broad `web_task_lifecycle_contract` with registry, stream redirector, and pipeline execution event units (`rewrite-in-rust/records/0019-split-web-task-lifecycle-unit.md:24`). Record 0022 confirms this unit around `TaskManager._execute_pipeline` while excluding task registry behavior, `WebStreamRedirector` internals, Web config mapping, Flask routes, real SocketIO transport, and real model inference (`rewrite-in-rust/records/0022-confirm-web-pipeline-execution-events-boundary.md:17`, `rewrite-in-rust/records/0022-confirm-web-pipeline-execution-events-boundary.md:23`).

Capability coverage is otherwise appropriate for dependency bootstrap. The dependency record covers execution logs/progress events, terminal status mapping, and extension-order output-file collection (`rewrite-in-rust/dependencies/web_pipeline_execution_events.yaml:3`). The bootstrap narrows the compatibility surface to config handoff, cancel-checker injection, stdout/stderr restoration, startup/progress/log events, fake pipeline outcomes, output collection, cancellation/failure distinctions, and manager-error fallback (`rewrite-in-rust/bootstrap/web_pipeline_execution_events.md:11`).

Kept-legacy decisions are sound. Real `run_auto_lyric_job` model inference, task registry start/stop/list behavior, `WebStreamRedirector` internals, real SocketIO transport, and Web config mapping stay legacy-owned or covered by separate units (`rewrite-in-rust/dependencies/web_pipeline_execution_events.yaml:39`). This matches the Stage 1 exclusion of model execution and frontend/Web-server replacement.

The seam choice is acceptable: independent Rust library, legacy runtime owner, and no bridge dependencies (`rewrite-in-rust/dependencies/web_pipeline_execution_events.yaml:16`; `rewrite-in-rust/bootstrap/web_pipeline_execution_events.md:46`). The hand-written replacement rationale is also acceptable because a narrow event/state model over fake SocketIO/config/pipeline/clock/output inputs is smaller and more verifiable than replacing Flask-SocketIO, Python threading, or model inference (`rewrite-in-rust/dependencies/web_pipeline_execution_events.yaml:32`).

No missing production crate risk found. `v2m-core` has no normal dependencies, and `serde_json` is present only as a dev-dependency for JSONL fixture tests (`rewrite-in-rust/rust/crates/v2m-core/Cargo.toml:12`).

## Checks

- `git status --short`: reviewed dirty tree and untracked rewrite artifacts; report write scope stayed limited to this review file.
- `git diff --stat` and focused `git diff`: reviewed current tracked manifest/resource/Cargo/lib changes, with untracked unit records/dependencies/bootstrap/fixture/Rust files read directly.
- `git diff --check`: passed.
- `uv run python rewrite-in-rust/bootstrap/check_web_pipeline_execution_events.py`: passed.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml web_pipeline_events` with `CARGO_TARGET_DIR` in `/tmp`: passed; 1 `web_pipeline_events` test passed.
- `cargo tree --manifest-path rewrite-in-rust/rust/Cargo.toml -p v2m-core --edges normal`: passed; no normal dependencies.
- `cargo tree --manifest-path rewrite-in-rust/rust/Cargo.toml -p v2m-core --edges dev`: passed; only `serde_json` appears under dev-dependencies.

## Residual Risk

This role did not perform behavior, error-tracing, product, Rust style, or architecture review. The remaining dependency-bootstrap risk is fixture breadth around full SocketIO payload shape and fake-clock timestamp assertions. Status-change emit failure behavior is also not covered here and should stay with the required error-tracing review unless the unit scope is expanded.

## Promotion Note

This dependency/bootstrap role does not block the confirmed inventory boundary or seam. It should not be used as clean final promotion evidence until the medium fake SocketIO payload follow-up is fixed or explicitly accepted by the coordinator and later reviewers. Runtime ownership and rollback stay with `web_task_manager.TaskManager._execute_pipeline`; do not mark the manifest verified from this report alone.
