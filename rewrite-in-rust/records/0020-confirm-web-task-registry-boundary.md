# 0020 - Confirm Web Task Registry Boundary

## Context

After splitting `web_task_lifecycle_contract`, the first resulting unit is
`web_task_registry_contract`. It covers the `TaskManager` methods that manage
task registry state before the pipeline execution body runs.

The source still uses Python threading, UUID generation, and wall-clock time,
but those are environmental inputs. The behavior to preserve can be proven with
fake UUID, fake clock, fake thread, and fake event objects.

## Decision

Confirm `web_task_registry_contract` as a narrow state-machine unit covering:

- `Task` creation defaults;
- `create_task`;
- `start_task` pending-only transition and thread metadata;
- `stop_task` running-only stop event behavior;
- `get_task`;
- `list_tasks`.

Do not include `_execute_pipeline`, `WebStreamRedirector`, live SocketIO
transport, Flask routes, config mapping, output scanning, or model inference.

## Consequences

- The first lifecycle split can be implemented without running threads or model
  inference.
- A later `web_pipeline_execution_events` unit remains responsible for
  completed/cancelled/failed execution outcomes and event payloads.
- The Rust model accepts injected task IDs and timestamps instead of adding UUID
  or time crates.

## Reversal

If later implementation shows registry and execution events must share a Rust
task data model, extract that model in a new record. Until then, rollback is
unchanged: `web_task_manager.TaskManager` remains the production owner.
