# 0032 - Confirm Web Model Download Task Lifecycle Boundary

## Context

Record 0031 split the remaining model-download lifecycle/termination work into
task lifecycle, execution-result handling, and process termination. The first
of those is manager state before any real subprocess behavior is required.

`ModelDownloadManager` lifecycle setup has a small deterministic core, but it
touches UUIDs, clocks, thread objects, events, and manager registry state. Those
unstable values must be injected by fixtures.

## Decision

Confirm `web_model_download_task_lifecycle_contract` as one unit covering:

- `create_task` defaults, UUID/clock assignment, selected-model copying, proxy
  URL trimming, stop-event creation, and task registration;
- `get_task` found/not-found lookup;
- `active_task` returning only active ids whose status is `pending`, `running`,
  or `stopping`;
- `start_task` active conflict behavior;
- `start_task` success behavior: task creation, thread metadata, active task id,
  status transition to `running`, `started_at`, and thread start hook;
- starting a new task when the previous active id points at a completed task;
- `stop_task` behavior for missing tasks, non-running/non-pending statuses,
  pending/running tasks without a process, and missing stop events.

Do not include:

- route request parsing or response mapping;
- process command/env planning or stdout parsing;
- `_execute_download` result transitions;
- real `subprocess.Popen` execution;
- process waiting or cancellation timeout escalation;
- POSIX/Windows process-tree termination;
- SocketIO delivery behavior;
- real downloads, package installation, archive extraction, or asset marker
  checks.

## Consequences

- The unit can be verified with fake UUID, clock, event, thread, and manager
  state fixtures.
- OS termination remains isolated in
  `web_model_download_process_termination_contract`.
- `_execute_download` status/result behavior remains isolated in
  `web_model_download_execution_result_contract`.

## Reversal

Rollback is keeping `web_model_download_manager.ModelDownloadManager` as the
runtime owner for task lifecycle state. No Rust bridge is introduced.
