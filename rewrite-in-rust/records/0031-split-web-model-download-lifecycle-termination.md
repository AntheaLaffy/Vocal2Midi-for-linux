# 0031 - Split Web Model Download Lifecycle and Termination

## Context

Record 0029 split the original broad Web model-download contract into
request/catalog, process planning, and lifecycle/termination work. After
implementing request/catalog and process planning, the remaining
`web_model_download_lifecycle_termination_contract` still mixed several
independent risk areas:

- task registry creation and active-task state;
- thread/event lifecycle behavior;
- `_execute_download` result transitions and SocketIO status payloads;
- cancellation handoff to termination;
- POSIX and Windows process-tree termination.

One fixture harness covering all of those would need to fake locks, threads,
events, subprocesses, waits, clocks, SocketIO, and OS kill APIs at once. That is
too broad for a small migration unit.

## Decision

Re-cut the remaining lifecycle/termination work into three planned units:

- `web_model_download_task_lifecycle_contract`;
- `web_model_download_execution_result_contract`;
- `web_model_download_process_termination_contract`.

`web_model_download_task_lifecycle_contract` covers manager state and lifecycle
setup:

- `create_task` defaults, UUID/clock injection, proxy URL trimming, stop-event
  creation, and task registration;
- `get_task` lookup;
- `active_task` returning only pending/running/stopping active tasks;
- `start_task` active conflict, thread metadata, status transition to running,
  `started_at`, `active_task_id`, and thread start hook;
- `stop_task` behavior for missing tasks, non-running statuses, and running
  tasks without a live process.

`web_model_download_execution_result_contract` covers `_execute_download`
outcome handling with fake process/socket fixtures:

- initial command/log/progress behavior;
- process assignment and output-reader handoff;
- return code 0 completion;
- nonzero return code failure;
- cancellation path and timeout escalation callback;
- exception path logging/status;
- status-change payload emission and active-task cleanup.

`web_model_download_process_termination_contract` covers OS-specific process
termination:

- already-exited no-op;
- POSIX `os.killpg` SIGTERM/SIGKILL behavior;
- POSIX `ProcessLookupError` no-op;
- POSIX `OSError` fallback to `terminate` or `kill`;
- Windows `taskkill /PID <pid> /T` and `/F` behavior;
- Windows `OSError` fallback to `terminate` or `kill`;
- `stop_task` process-present success and OSError false result.

Do not include actual subprocess execution, network downloads, package
installation, archive extraction, or model asset marker safety in any of these
units. Those remain legacy-owned or part of `download_models_asset_safety`.

## Consequences

- The next writer can choose a small fixture harness instead of one large fake
  runtime.
- OS kill behavior can be reviewed separately from Web/task lifecycle state.
- `_execute_download` status transitions can use the already-confirmed
  process-plan parser as a dependency without re-opening command/env parsing.

## Reversal

Rollback is keeping `web_model_download_manager.ModelDownloadManager` as the
runtime owner for all lifecycle, execution, and termination behavior. No Rust
bridge is introduced by this split.
