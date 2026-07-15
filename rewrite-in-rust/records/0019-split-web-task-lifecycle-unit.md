# 0019 - Split Web Task Lifecycle Unit

## Context

The provisional `web_task_lifecycle_contract` unit pointed at
`web_task_manager.py`, `web_stream_redirector.py`, and `application/pipeline.py`.
Dependency inspection showed that the unit combined several independent
capability shapes:

- task registry state: task creation, pending/running transitions, stop events,
  lookup, and list summaries;
- stream redirection: callback emission, whitespace filtering, callback failure
  swallowing, underlying stream writes, flush, and attribute delegation;
- pipeline execution events: fake WebSocket emissions, config building,
  cancel-checker injection, stdout/stderr restoration, output file collection,
  success/cancel/failure distinctions, and manager-error fallback.

Keeping these together would make the first writer pass larger than needed and
would couple simple deterministic stream behavior to task execution and fake
SocketIO fixtures.

## Decision

Replace `web_task_lifecycle_contract` with three smaller planned units:

- `web_task_registry_contract`
- `web_stream_redirector_contract`
- `web_pipeline_execution_events`

Each unit remains legacy-owned at runtime and must go through its own
dependency/bootstrap pass before writer work.

The execution-events unit may depend on the already reimplemented
`web_pipeline_config_mapping`, but it must still use fake config and fake
pipeline call fixtures. It must not run model inference or a real SocketIO
server.

## Consequences

- Stage 1 can implement the registry and stream redirector contracts with much
  smaller fixture tables before tackling the broader pipeline event behavior.
- Review roles can be chosen per capability: stream redirection is mostly
  behavior/error tracing; execution events need stronger error tracing and
  product ergonomics attention.
- The original broad unit remains documented only as the split discovery record,
  not as an implementation target.

## Reversal

If later implementation shows the split creates artificial duplication, merge
only the affected units in a new record. Until then, production rollback is
unchanged: `web_task_manager.TaskManager` and `web_stream_redirector` remain the
runtime owners.
