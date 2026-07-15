# web_task_registry_contract Bootstrap

## Boundary

`web_task_registry_contract` covers the registry and state-transition part of
`web_task_manager.py::TaskManager`:

- `Task` default field values
- `TaskManager.create_task`
- `TaskManager.start_task`
- `TaskManager.stop_task`
- `TaskManager.get_task`
- `TaskManager.list_tasks`

The public compatibility surface is:

- `create_task` creates a UUID task with status `pending`, progress `0`, stage
  `idle`, retained config/audio path, `created_at=datetime.now()`, no
  `started_at` or `completed_at`, no error, empty output/log lists, no thread,
  and a clear stop event.
- `start_task` returns `False` for missing or non-`pending` tasks.
- starting a pending task sets status `running`, sets `started_at`, creates a
  daemon thread named `Pipeline-<first 8 chars of task_id>`, starts it, and
  returns `True`.
- `stop_task` returns `False` for missing or non-`running` tasks.
- stopping a running task sets its stop event and returns `True`.
- `get_task` returns the stored task object or `None`.
- `list_tasks` returns summaries with id, status, progress, stage, and ISO
  timestamp fields only.

This unit does not execute `_execute_pipeline`, redirect streams, emit real
SocketIO events, build configs, run the application pipeline, or collect output
files.

## Dependency Expansion

The selected boundary uses:

- stdlib: `uuid`, `datetime`, `threading`, and dictionary/list state
- local: `web_task_manager.Task`

The fixture harness injects deterministic UUIDs, deterministic clock values,
and fake thread/event classes. This proves the state contract without using the
real scheduler or running the pipeline body.

It does not require Flask, Flask-SocketIO, model runtimes, ONNX Runtime, Qwen
ASR, PyQt, WebStreamRedirector, or `application.pipeline`.

## Seam

Use the default rewrite seam:

- kind: independent Rust library
- crate: `rewrite-in-rust/rust/crates/v2m-core`
- runtime owner: legacy Python
- bridge dependencies: none

The Rust side models task registry state directly. It accepts explicit task IDs
and timestamps in tests rather than generating UUIDs or wall-clock time.

## Fixture Harness

Python and Rust tests consume:

```text
rewrite-in-rust/fixtures/web_task_registry_contract.jsonl
```

The fixtures cover:

- task creation defaults and list summary shape
- pending task start
- missing task start
- non-pending task start
- running task stop
- pending task stop
- missing task stop
- multi-task creation order, `get_task` hits, and missing lookup

The legacy Python side is checked by:

```bash
uv run python rewrite-in-rust/bootstrap/check_web_task_registry_contract.py
```

## Repeated-Call Behavior

For injected UUID and clock values, repeated registry operations produce the
same state transitions. The unit does not depend on real SocketIO rooms, real
threads completing, model availability, or filesystem output scanning.

## Rollback

Rollback is keeping production ownership unchanged:

```text
web_task_manager.TaskManager
```

No Web caller should import Rust output for this unit until a later promotion
record chooses and verifies a bridge.
