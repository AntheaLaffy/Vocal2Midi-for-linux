# 0018 - Confirm Web Pipeline Config Mapping Boundary

## Context

After `application_job_contract`, the next Stage 1 unit is
`web_pipeline_config_mapping`. The selected source is
`web_task_manager.py::TaskManager._build_config`.

Source inspection showed that this function is not the Web task lifecycle. It
does not create tasks, emit SocketIO events, run Flask request parsing, or start
model inference. It maps a frontend config dictionary into `PipelineConfig`,
creates the output directory, normalizes the device field, applies slice-bound
fallback, builds output formats, and generates timestamps.

The mapping also preserves a current product quirk: Web quantization settings
remain visible/persisted elsewhere but are ignored by `_build_config`, so
`PipelineConfig` keeps `quantization_step=16` and `quantization_mode="bayes"`.

## Decision

Confirm `web_pipeline_config_mapping` as a narrow Web-to-application mapping
unit.

Implement only the supported JSON-compatible mapping path and keep these
capabilities out of the unit:

- persisted settings merge in `web_server.py`;
- Flask request parsing and validation;
- task creation, threading, cancellation, stdout/stderr redirection, SocketIO
  events, and output collection;
- GUI config construction;
- model downloads;
- `auto_lyric_hybrid_pipeline` and all model inference.

Use an independent Rust library function with JSONL fixtures. `serde_json` is
allowed only as a Rust test/dev dependency for reading those fixtures.

## Consequences

- The Web mapping can be tested without launching Flask, SocketIO, or model
  runtimes.
- Later Web settings and task lifecycle units must not silently reuse this
  unit's scope as permission to rewrite Flask or threading behavior.
- If product owners later make Web quantization controls effective, they must
  update the verified caller/default contract before changing this mapping.

## Reversal

If a future promotion needs runtime Web config mapping through Rust, add a
separate bridge/promotion record. Until then, rollback is keeping
`web_task_manager.TaskManager._build_config` as the production owner.
