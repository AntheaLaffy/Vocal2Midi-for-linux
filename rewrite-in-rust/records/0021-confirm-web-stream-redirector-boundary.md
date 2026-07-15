# 0021 - Confirm Web Stream Redirector Boundary

## Context

After splitting `web_task_lifecycle_contract`, `web_stream_redirector_contract`
is the smallest Web lifecycle slice. The Python source is a standalone
callback-based stream wrapper used by the task execution path.

It does not know about task IDs, SocketIO payload shape, pipeline status, or
model execution. Those concerns belong to `web_pipeline_execution_events`.

## Decision

Confirm `web_stream_redirector_contract` as a narrow behavior unit covering:

- non-empty stripped callback writes;
- callback exception swallowing;
- original stream writes;
- flush delegation;
- delegated attribute access.

Use fake stream and fake callback fixtures. Do not introduce a real stream
bridge, SocketIO dependency, task manager dependency, or model-runtime
dependency.

## Consequences

- The stream wrapper can be verified independently of `_execute_pipeline`.
- Later execution-event fixtures can assume stream redirection itself is already
  covered and focus on when stdout/stderr are installed/restored.
- Error tracing review should check that callback failures remain deliberately
  swallowed to avoid breaking pipeline execution.

## Reversal

If execution-event work requires a shared logging model, add that model in a
new record. Until then, rollback is keeping
`web_stream_redirector.WebStreamRedirector` as the production owner.
