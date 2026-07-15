# web_pipeline_execution_events - behavior_reviewer

Date: 2026-07-16
Decision: fail

## Findings

- Severity: medium
- Location: rewrite-in-rust/rust/crates/v2m-core/src/web_pipeline_events.rs:30
- Issue: stdout/stderr restoration is part of this unit's stated behavior, but the Rust parity surface does not model or assert it. The fixture rows include `stdout_restored` and `stderr_restored`, and the Python checker hard-checks restoration after `_execute_pipeline`, but the Rust outcome has no restoration fields and the Rust fixture test ignores those fixture keys.
- Evidence: The boundary record lists stdout/stderr restoration as public compatibility behavior (`rewrite-in-rust/bootstrap/web_pipeline_execution_events.md:15`). Python restores both streams in the inner `finally` (`web_task_manager.py:298`) and the checker verifies the streams are back to the pre-call objects (`rewrite-in-rust/bootstrap/check_web_pipeline_execution_events.py:234`). The fixture declares restoration expectations (`rewrite-in-rust/fixtures/web_pipeline_execution_events.jsonl:1`). Rust only exposes task state, logs, progress events, status changes, and output files (`rewrite-in-rust/rust/crates/v2m-core/src/web_pipeline_events.rs:32`), and the test assertions stop at those modeled fields (`rewrite-in-rust/rust/crates/v2m-core/src/web_pipeline_events.rs:395`).
- Required fix: Add Rust-side restoration fields/assertions or explicitly re-cut the unit boundary so restoration is not claimed by this unit. If restoration stays in scope, the shared fixture should fail Rust tests when `stdout_restored` or `stderr_restored` does not match.

- Severity: medium
- Location: rewrite-in-rust/rust/crates/v2m-core/src/web_pipeline_events.rs:7
- Issue: The SocketIO payload and ordering contract is under-modeled. Python emits `task_id` in log, progress, and status payloads and uses `room=task.id`, but the Rust event structs omit `task_id`, room, log timestamps, and a unified emit sequence. The Python checker normalizes progress/status payloads by dropping `task_id`, compares task logs instead of log emit payloads, and only checks room separately. The Rust test parses `task_id` into input but never asserts it in any output.
- Evidence: Python log/progress/status payloads include `task_id` and rooms (`web_task_manager.py:179`, `web_task_manager.py:186`, `web_task_manager.py:195`, `web_task_manager.py:198`, `web_task_manager.py:253`, `web_task_manager.py:259`). Rust log/progress/status structs contain only message/level, progress/stage, and terminal result fields (`rewrite-in-rust/rust/crates/v2m-core/src/web_pipeline_events.rs:7`, `rewrite-in-rust/rust/crates/v2m-core/src/web_pipeline_events.rs:14`, `rewrite-in-rust/rust/crates/v2m-core/src/web_pipeline_events.rs:21`). The checker drops `task_id` while normalizing progress and status events (`rewrite-in-rust/bootstrap/check_web_pipeline_execution_events.py:109`, `rewrite-in-rust/bootstrap/check_web_pipeline_execution_events.py:120`), while the Rust test consumes `task_id` but does not verify it (`rewrite-in-rust/rust/crates/v2m-core/src/web_pipeline_events.rs:361`). Because logs, progress events, and status changes are separate vectors, a regression in global emit order could still pass as long as each per-channel list matched.
- Required fix: Extend the shared fixture and both checkers to compare a unified emitted-event trace, or at minimum assert `task_id` and room for each log/progress/status event on both Python and Rust sides. If timestamps remain intentionally nondeterministic, assert their presence/format separately or document them as outside the Rust model.

- Severity: low
- Location: rewrite-in-rust/rust/crates/v2m-core/src/web_pipeline_events.rs:223
- Issue: Output-file ordering is only proven for one matching file per extension. Python groups extensions in `.mid`, `.ustx`, `.txt`, `.csv` order but uses `Path.glob` order within each extension. Rust groups by extension and preserves fixture input order within each group. The current fixture cannot prove what should happen with multiple `.mid`, `.ustx`, `.txt`, or `.csv` files.
- Evidence: Python loops by extension and extends with `output_dir.glob(ext)` results (`web_task_manager.py:241`). Rust loops extensions and then fixture-provided filenames (`rewrite-in-rust/rust/crates/v2m-core/src/web_pipeline_events.rs:225`). The completed fixture has one file for each collected extension plus one ignored file (`rewrite-in-rust/fixtures/web_pipeline_execution_events.jsonl:1`).
- Required fix: Add a fixture with multiple matching files for at least one extension and document whether same-extension order is part of compatibility or only extension grouping is guaranteed.

## Behavior Evidence

- Unit and role: reviewed exactly `web_pipeline_execution_events` as `behavior_reviewer`.
- Boundary: records 0019 and 0022 split this unit away from task registry behavior, stream redirector internals, Web config mapping, Flask routes, real SocketIO transport, and model inference (`rewrite-in-rust/records/0019-split-web-task-lifecycle-unit.md:24`, `rewrite-in-rust/records/0022-confirm-web-pipeline-execution-events-boundary.md:15`).
- Writer/reviewer separation: I only reviewed and wrote this report. I did not edit production code, manifest, records, fixtures, bootstrap scripts, or Rust/Python source.
- Modeled task state paths match the current fixture table for completed, stop-after-run cancellation, `KeyboardInterrupt`, stopped error after cancellation, generic failure, and manager/config error (`rewrite-in-rust/fixtures/web_pipeline_execution_events.jsonl:1`).
- Python source behavior checked: `_execute_pipeline` logs before `_build_config`, injects `config.cancel_checker`, installs `WebStreamRedirector`, emits loading progress before calling `run_auto_lyric_job`, restores stdout/stderr, collects output files by extension, and maps completed/cancelled/failed/manager-error terminal states (`web_task_manager.py:203`, `web_task_manager.py:207`, `web_task_manager.py:213`, `web_task_manager.py:226`, `web_task_manager.py:229`, `web_task_manager.py:241`, `web_task_manager.py:270`, `web_task_manager.py:279`, `web_task_manager.py:303`).
- Application cancellation/error context checked: `run_auto_lyric_job` validates model paths, checks `cfg.cancel_checker` before starting, maps `InterruptedError` to `CancellationError`, passes `Vocal2MidiError`, and wraps other exceptions (`application/pipeline.py:46`, `application/pipeline.py:49`, `application/pipeline.py:54`, `application/pipeline.py:56`, `application/pipeline.py:58`).
- Rollback is documented as keeping `web_task_manager.TaskManager._execute_pipeline` as the production owner, with no Rust bridge introduced for this unit (`rewrite-in-rust/bootstrap/web_pipeline_execution_events.md:88`, `rewrite-in-rust/manifest.yaml:493`).

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_web_pipeline_execution_events.py`: passed with no output.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml web_pipeline_events`: passed; 1 `web_pipeline_events` test passed, 45 filtered out in `v2m-core`, and 0 matching tests in `v2m-quant-bridge`.
- `uv run pytest tests/test_web_api.py`: passed; 53 tests passed.
- `git diff --check`: passed.
- Current git diff/status inspected. The working tree contains active tracked and untracked migration changes by others; this review only added this report.

## Residual Risk

Real SocketIO transport, real model inference, Flask routes, task registry lifecycle, Web config mapping, and `WebStreamRedirector` internals remain outside this unit by record 0022. Those exclusions are appropriate, but the currently modeled Rust behavior is not yet sufficient durable evidence for the claimed stdout/stderr restoration and full SocketIO payload/order surface.

## Promotion Note

This behavior role blocks coordinator verification for `web_pipeline_execution_events` until the findings are fixed or the unit boundary is explicitly re-cut. Do not mark the manifest `verified` from this report.
