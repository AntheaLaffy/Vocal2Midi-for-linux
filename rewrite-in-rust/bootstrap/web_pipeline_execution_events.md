# web_pipeline_execution_events Bootstrap

## Boundary

`web_pipeline_execution_events` covers only:

```text
web_task_manager.py::TaskManager._execute_pipeline
```

The public compatibility surface is:

- config-building log before `_build_config`;
- `config.cancel_checker` injection from `task.stop_event.is_set`;
- stdout/stderr replacement with `WebStreamRedirector` and restoration in the
  inner `finally`;
- startup log messages and loading progress event, including SocketIO
  `task_id`, room, log timestamp, and cross-event emit order;
- fake `run_auto_lyric_job(config)` handoff;
- completed status, progress, stage, completed timestamp, output file
  collection, success logs, and completed `status_change`;
- cancelled status when the stop event is set after the pipeline returns;
- `KeyboardInterrupt` mapping to cancelled status;
- generic exception mapping to cancelled when the stop event is set;
- generic exception mapping to failed status, `task.error`, error logs,
  traceback log, and failed `status_change` otherwise;
- outer task-manager error path when config building or setup fails.

The unit does not cover task registry start/stop/list behavior,
`WebStreamRedirector` internals, Web config mapping, Flask routes, real
SocketIO transport, or model inference.

## Dependency Expansion

The selected source uses:

- stdlib: `sys`, `pathlib`, `traceback`, and `datetime`
- local: `web_stream_redirector.WebStreamRedirector`
- local: `application.pipeline.run_auto_lyric_job`
- local: `TaskManager._build_config`

Fixture parity uses fake `SocketIO`, fake config objects, fake stop events, fake
clock values, fake output files, and patched `run_auto_lyric_job`. No real Web
server, thread scheduler, model runtime, ONNX Runtime, Qwen ASR, PyQt, or network
service is required.

## Seam

Use the default rewrite seam:

- kind: independent Rust library
- crate: `rewrite-in-rust/rust/crates/v2m-core`
- runtime owner: legacy Python
- bridge dependencies: none

The Rust side models the resulting task logs, progress events, status changes,
ordered SocketIO emit trace, stdout/stderr restoration, task state, and
collected output paths from explicit fixture inputs. No production bridge is
introduced.

## Fixture Harness

Python and Rust tests consume:

```text
rewrite-in-rust/fixtures/web_pipeline_execution_events.jsonl
```

The fixtures cover:

- completed run with stdout/stderr redirected logs and output file collection,
  including multiple files for each collected extension
- cancellation after pipeline return
- `KeyboardInterrupt` cancellation
- exception with stop event already set
- generic failure with traceback logging
- manager/config error before stream redirection

Each fixture also pins the ordered fake SocketIO trace. Log emits include
`task_id`, room, and deterministic fake-clock timestamps. Status emits preserve
the legacy distinction between omitted `error` fields and `error: null`.
Output-file fixtures pin current legacy `Path.glob` ordering within each
extension group and the legacy extension group order: `.mid`, `.ustx`, `.txt`,
then `.csv`.

The legacy Python side is checked by:

```bash
uv run python rewrite-in-rust/bootstrap/check_web_pipeline_execution_events.py
```

## Repeated-Call Behavior

For the same task state, fake config, fake pipeline result, fake clock, and fake
output directory, event order and terminal task state are deterministic. The
unit does not depend on live sockets, real model state, or network access.

## Rollback

Rollback is keeping production ownership unchanged:

```text
web_task_manager.TaskManager._execute_pipeline
```

No Web caller should import Rust output for this unit until a later promotion
record chooses and verifies a bridge.
