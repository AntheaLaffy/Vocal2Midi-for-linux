# web_model_download_task_lifecycle_contract Bootstrap

## Boundary

`web_model_download_task_lifecycle_contract` covers the deterministic manager
state behavior in `web_model_download_manager.py`:

```text
ModelDownloadManager.create_task
ModelDownloadManager.get_task
ModelDownloadManager.active_task
ModelDownloadManager.start_task
ModelDownloadManager.stop_task
```

The compatibility surface is:

- `create_task` stores a UUID string, copies `selected_models`, stores the qwen
  source and force flag, trims proxy URL whitespace, starts at
  `pending`/`queued`/`0`, records `created_at`, creates a stop event, and
  registers the task;
- mutating the caller's selected-model list after `create_task` does not mutate
  the task;
- `get_task` returns the matching task or `None`;
- `active_task` returns `None` when there is no active id, the active id is
  missing, or the active task is not `pending`, `running`, or `stopping`;
- `start_task` raises `RuntimeError("A model download task is already running.")`
  when the current active id points at a pending/running/stopping task;
- `start_task` creates a daemon thread named `ModelDownload-<first 8 id chars>`,
  sets `active_task_id`, status `running`, and `started_at`, then calls the
  thread's `start` hook;
- a completed previous active task does not block a new start;
- `stop_task` returns false for missing tasks and non-pending/non-running
  statuses;
- `stop_task` sets pending/running tasks to `stopping`, sets the stop event when
  present, and returns true when there is no live process.

The unit does not cover Web route response mapping, command/env planning,
stdout parsing, `_execute_download`, real thread target execution, real
subprocesses, process waiting, POSIX/Windows process termination, SocketIO
delivery, network downloads, package installation, archive extraction, or model
marker safety.

## Dependency Expansion

The selected behavior uses:

- stdlib: `uuid.uuid4`, `datetime.now`, `threading.Lock`,
  `threading.Event`, and `threading.Thread`;
- local dataclass: `ModelDownloadTask`.

Fixture parity injects UUIDs, clock values, fake event objects, fake thread
objects, and pre-existing manager tasks. No real thread target, subprocess,
network, SocketIO, or filesystem model asset is invoked.

## Seam

Use the default rewrite seam:

- kind: independent Rust library
- crate: `rewrite-in-rust/rust/crates/v2m-core`
- runtime owner: legacy Python
- bridge dependencies: none

The Rust side models task lifecycle state from explicit JSON fixture inputs. No
production bridge is introduced.

## Fixture Harness

Python and Rust tests consume:

```text
rewrite-in-rust/fixtures/web_model_download_task_lifecycle_contract.jsonl
```

The legacy Python side is checked by:

```bash
uv run python rewrite-in-rust/bootstrap/check_web_model_download_task_lifecycle_contract.py
```

## Rollback

Rollback is keeping production ownership unchanged:

```text
web_model_download_manager.ModelDownloadManager
```

No Web caller should import Rust output for this unit until a later promotion
record chooses and verifies a bridge.
