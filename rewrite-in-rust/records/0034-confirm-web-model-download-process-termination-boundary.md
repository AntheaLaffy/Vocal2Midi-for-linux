# 0034 - Confirm Web Model Download Process Termination Boundary

## Context

Records 0031, 0032, and 0033 split the original model-download lifecycle work
into task lifecycle, execution-result handling, and process termination. The
remaining behavior is the OS-facing termination logic plus the `stop_task`
branch that invokes it for a live child process.

`ModelDownloadManager._terminate_process_tree` is compact but platform
dependent. Mixing it back into `_execute_download` would force every execution
fixture to care about POSIX signals, Windows `taskkill`, and fallback methods.

## Decision

Confirm `web_model_download_process_termination_contract` as one unit covering:

- already-exited process no-op;
- POSIX `os.killpg(process.pid, SIGTERM)` and `SIGKILL` selection;
- POSIX `ProcessLookupError` no-op;
- POSIX `OSError` fallback to `process.terminate()` or `process.kill()`;
- Windows `taskkill /PID <pid> /T` and forced `/F` command construction;
- Windows `subprocess.run` `OSError` fallback to `terminate()` or `kill()`;
- `stop_task` process-present success path;
- `stop_task` process-present `OSError` false result after status/stop-event
  are already updated.

The unit excludes task creation/start state without a live process, request
routes, command/env planning, output parsing, `_execute_download` terminal
status handling, real process termination, network downloads, package
installation, archive extraction, and model marker safety.

## Consequences

Fixtures use fake process objects, fake `os.name`, fake `os.killpg`, fake
`subprocess.run`, and fake termination exceptions. No test sends a signal,
starts a child process, or runs `taskkill`.

The model-download runtime remains legacy Python until a later promotion record
chooses and verifies a bridge.

## Reversal

Rollback is keeping `web_model_download_manager.ModelDownloadManager.stop_task`
and `_terminate_process_tree` as runtime owners. No production bridge is
introduced by this record.
