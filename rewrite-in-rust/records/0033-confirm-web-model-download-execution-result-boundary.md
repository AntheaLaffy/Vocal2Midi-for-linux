# 0033 - Confirm Web Model Download Execution Result Boundary

## Context

Record 0031 split the remaining model-download work into task lifecycle,
execution-result handling, and process termination. The task lifecycle unit is
now independently fixture-backed and reviewed.

`ModelDownloadManager._execute_download` still combines deterministic task
state transitions with high-risk runtime operations:

- subprocess spawn arguments and environment handoff;
- output reader handoff;
- terminal status mapping for success, failure, cancellation, and exceptions;
- SocketIO log/progress/status payloads;
- active task cleanup;
- process-tree termination during cancellation.

The process spawn command/env/parser behavior is already covered by
`web_model_download_process_plan_contract`; OS-specific process killing remains
reserved for `web_model_download_process_termination_contract`.

## Decision

Confirm `web_model_download_execution_result_contract` as one unit covering
only `_execute_download` orchestration over fake process/socket fixtures:

- initial "prepare" and command log emission;
- initial progress emission at `2` / `starting`;
- fake `subprocess.Popen` argument capture and task process assignment;
- handoff to `_read_process_output` without re-opening parser semantics;
- return code `0` mapping to `completed` / `done` / `100`;
- nonzero return code mapping to `failed` / `failed` with legacy error text;
- cancellation mapping to `cancelled` / `cancelled`, including timeout
  escalation callback recording;
- exception mapping to `failed` with error and traceback logs;
- `status_change` payload emission through `serialize_task`;
- `active_task_id` cleanup only when it still points to the task.

The unit excludes real `download_models.py` execution, network access, package
installation, archive extraction, model marker safety, and OS process-tree
termination semantics.

## Consequences

Fixtures inject fake stdout, return codes, wait plans, stop-event state,
environment maps, process-group kwargs, and active-task ids. The Rust side uses
the process-plan helpers for command/env/output parsing and models only the
execution-result state machine.

The remaining planned model-download unit is
`web_model_download_process_termination_contract`.

## Reversal

Rollback is keeping `web_model_download_manager.ModelDownloadManager` as the
runtime owner for `_execute_download`. No production bridge is introduced by
this record.
