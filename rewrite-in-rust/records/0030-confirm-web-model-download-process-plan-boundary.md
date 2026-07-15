# 0030 - Confirm Web Model Download Process Plan Boundary

## Context

Record 0029 split the broad Web model-download manager unit into
request/catalog, process-planning, lifecycle/termination, and asset-safety
work. The request/catalog slice is fixture-bound and does not cover the
subprocess plan or output-line parser.

The next deterministic slice is the part of `ModelDownloadManager` that plans
the child process and interprets its text output, without actually starting or
terminating a child process.

## Decision

Confirm `web_model_download_process_plan_contract` as one fixture-bound unit
covering:

- `_build_command` ordering for `--only`, qwen source inclusion, and `--force`;
- `_build_process_env` proxy inheritance/clearing/manual override behavior;
- `_popen_process_group_kwargs` POSIX and Windows process-boundary kwargs;
- `_read_process_output` character buffering, newline/carriage-return splitting,
  and final-buffer handling using fake process/stdout objects;
- `_handle_output_line`, `_guess_model_from_line`,
  `_emit_progress_for_model`, `_emit_log`, and `_emit_progress` state/payload
  effects;
- legacy percent-regex behavior, including the surprising fact that
  `\b(\d{1,3})%\b` does not match a line ending in `50%`;
- log level classification and the 500-entry log cap.

Do not include:

- real `subprocess.Popen` calls;
- `create_task` UUID/clock assignment, stop-event creation, lock insertion, and
  task registry ownership;
- `download_models.py` execution;
- SocketIO delivery guarantees or WebSocket error print behavior;
- active-task locking, thread start semantics, cancellation transitions, task
  completion/failure handling, or process-tree termination;
- network downloads, model package installation, archive extraction, or asset
  marker safety.

## Consequences

- The Rust unit can verify the subprocess plan and output parser without a Web
  server, child process, network, or model runtime.
- Task creation and registry behavior stay isolated in
  `web_model_download_lifecycle_termination_contract`.
- Termination and lifecycle behavior stay isolated in
  `web_model_download_lifecycle_termination_contract`.
- Actual asset download behavior remains isolated in
  `download_models_asset_safety`.

## Reversal

Rollback is keeping `web_model_download_manager.ModelDownloadManager` as the
runtime owner for process planning and output parsing. No Rust bridge is
introduced by this unit.
