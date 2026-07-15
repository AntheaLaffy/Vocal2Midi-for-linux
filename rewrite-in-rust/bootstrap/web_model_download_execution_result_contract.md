# web_model_download_execution_result_contract Bootstrap

## Boundary

`web_model_download_execution_result_contract` covers the deterministic
task-visible behavior inside:

```text
ModelDownloadManager._execute_download
```

The compatibility surface is:

- startup emits two log messages, then progress `2` with stage `starting`;
- the fake child process is assigned to `task.process`;
- output reader handoff happens after process assignment;
- return code `0` sets `completed`, `done`, progress `100`, a success log,
  `completed_at`, `returncode`, and a `status_change`;
- nonzero return code sets `failed`, `failed`, legacy
  `download_models.py exited with code <n>` error text, `completed_at`,
  `returncode`, an error log, and a `status_change`;
- a set stop-event after output handling sets `cancelled`, `cancelled`,
  `completed_at`, warning log, and `status_change`;
- cancellation records the timeout escalation handoff but does not assert
  POSIX/Windows kill behavior;
- exceptions set `failed`, `failed`, `error`, `completed_at`, an error log,
  a traceback log, and a `status_change`;
- the final cleanup clears `active_task_id` only when it still points at the
  task id.

The unit does not cover Web route response mapping, task creation/start
lifecycle, subprocess command/env parsing details, stdout line parser edge
cases, real `download_models.py` execution, real SocketIO delivery failures,
POSIX/Windows process-tree termination, network downloads, package
installation, archive extraction, or model marker safety.

## Dependency Expansion

The selected behavior uses:

- stdlib: `subprocess.Popen`, `subprocess.TimeoutExpired`, `datetime.now`,
  and `traceback.format_exc`;
- local helpers: `_build_command`, `_build_process_env`,
  `_popen_process_group_kwargs`, `_read_process_output`, `_emit_log`,
  `_emit_progress`, `_emit_status`, and `serialize_task`;
- local dataclass: `ModelDownloadTask`.

Fixture parity injects fake Popen/process/stdout/socket/event objects, return
codes, wait plans, process-group kwargs, environment maps, and active-task ids.
Traceback and timestamp text are normalized.

## Seam

Use the default rewrite seam:

- kind: independent Rust library
- crate: `rewrite-in-rust/rust/crates/v2m-core`
- runtime owner: legacy Python
- bridge dependencies:
  - `web_model_download_process_plan_contract`
  - `web_model_download_request_catalog_contract`

The Rust side reuses process-plan helpers for command/env/output parsing and
models only the execution-result state transitions. No production bridge is
introduced.

## Fixture Harness

Python and Rust tests consume:

```text
rewrite-in-rust/fixtures/web_model_download_execution_result_contract.jsonl
```

The legacy Python side is checked by:

```bash
uv run python rewrite-in-rust/bootstrap/check_web_model_download_execution_result_contract.py
```

## Rollback

Rollback is keeping production ownership unchanged:

```text
web_model_download_manager.ModelDownloadManager
```

No Web caller should import Rust output for this unit until a later promotion
record chooses and verifies a bridge.
