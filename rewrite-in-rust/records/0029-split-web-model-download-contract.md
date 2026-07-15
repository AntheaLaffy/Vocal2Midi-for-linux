# 0029 - Split Web Model Download Contract

## Context

The provisional `web_model_download_contract` bundled several independent
behaviors:

- model catalog/status serialization;
- Web request defaults, validation, conflict mapping, and stop/status route
  responses;
- subprocess command and proxy environment planning;
- output parsing, progress mapping, log emission, and log retention;
- active-task locking, cancellation, and process-tree termination.

That boundary was too broad for one fixture set. It mixed pure request/catalog
logic with effectful thread, subprocess, SocketIO, and OS termination behavior.

## Decision

Re-cut the provisional unit into three smaller units:

- `web_model_download_request_catalog_contract`;
- `web_model_download_process_plan_contract`;
- `web_model_download_lifecycle_termination_contract`.

Confirm and implement only `web_model_download_request_catalog_contract` in this
pass. It covers:

- `GET /api/models/status` model catalog fields and installed/missing counts;
- active model-download task serialization, including proxy URL redaction;
- `POST /api/models/download` request normalization, missing-model defaulting,
  model-id deduplication, validation errors, and active-task conflict mapping;
- `GET /api/models/download/status/<task_id>` found/not-found mapping;
- `POST /api/models/download/stop` missing id, not found, cannot stop, and
  success response mapping.

Do not include:

- subprocess command construction;
- proxy environment shaping for the child process;
- stdout/stderr parsing and per-model progress math;
- log cap behavior and SocketIO emission failures;
- active-task lock ownership, thread creation, cancellation transitions, or
  process-tree termination;
- actual `download_models.py` execution, network downloads, or archive safety.

## Consequences

- The first model-download Rust unit stays a pure library data contract with no
  Web server, subprocess, SocketIO, network, or model runtime bridge.
- Process planning and lifecycle/termination remain visible as planned units
  instead of being hidden inside a broad task-manager bucket.
- The existing Python Web routes and `ModelDownloadManager` remain runtime
  owners until a later explicit promotion record.

## Reversal

Rollback is keeping the original Python runtime ownership unchanged:
`web_server.py` model download routes, `web_model_download_manager.py`, and
`download_models.py`.
