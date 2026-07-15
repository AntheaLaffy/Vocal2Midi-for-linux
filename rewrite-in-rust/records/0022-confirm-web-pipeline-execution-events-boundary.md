# 0022 - Confirm Web Pipeline Execution Events Boundary

## Context

After splitting the broad Web lifecycle unit, `web_pipeline_execution_events`
is the remaining `_execute_pipeline` behavior. The registry and stream
redirector contracts are already separated, which leaves event ordering,
terminal task state, status payloads, stdout/stderr installation/restoration,
output-file collection, and exception/cancellation distinctions.

The production code imports `run_auto_lyric_job`, but the selected boundary does
not require model inference. It only needs to prove how `_execute_pipeline`
reacts to fake application-pipeline outcomes.

## Decision

Confirm `web_pipeline_execution_events` as one event/state unit around
`TaskManager._execute_pipeline`.

Use fake SocketIO, fake config builder, fake stop event, fake clock, fake output
files, fake stdout/stderr streams, and patched `run_auto_lyric_job` fixtures.
The fake SocketIO evidence should preserve one ordered emit trace, including
event name, room, `task_id`, log timestamps, status payload shape, and terminal
result payloads.

Do not include:

- task registry creation/start/stop/list behavior;
- WebStreamRedirector internals;
- Web config mapping;
- Flask route/request behavior;
- real SocketIO transport;
- real model inference.

## Consequences

- `_execute_pipeline` can be verified without starting a Web server or loading
  models.
- Behavior review should focus on ordered SocketIO emit payloads, terminal
  status, output-file order, traceback/error logs, and stdout/stderr
  restoration.
- Error tracing review should decide whether current traceback and swallowed
  emit behavior are acceptable before any future promotion.

## Reversal

If later review finds this unit too broad, split output-file collection from
terminal event handling in a new record. Until then, rollback remains keeping
`web_task_manager.TaskManager._execute_pipeline` as the production owner.
