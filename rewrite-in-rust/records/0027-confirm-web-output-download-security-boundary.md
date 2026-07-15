# 0027 - Confirm Web Output Download Security Boundary

## Context

Record 0025 split the provisional filesystem/download unit. The remaining
`web_output_download_security` behavior is the authorization boundary for files
served by `GET /api/download/<path>`.

The selected code is small but security-sensitive. It mixes URL path validation,
project-root resolution, task output registration, canonical path comparison,
and Flask `send_file` response metadata.

## Decision

Confirm `web_output_download_security` as one output-download authorization
unit covering:

- `WINDOWS_DRIVE_RE`;
- `_safe_requested_download_path`;
- `_authorized_output_file`;
- `GET /api/download/<path>`.

Use temporary project roots, synthetic files/symlinks, synthetic registered task
outputs, and Flask test-client route calls in the Python checker. The Rust side
models the same path validation, canonical matching, 404 error shapes, and
successful `download_name`/body fixture result.

Do not include:

- filesystem picker path/listing behavior;
- task registry start/stop/list behavior;
- pipeline output collection;
- Flask route replacement or real file streaming;
- browser UI behavior.

## Consequences

- Download authorization can be verified without starting a Web server or
  running a pipeline.
- Registered output matching remains exact after canonical path normalization.
- URL paths remain project-relative only: absolute paths, traversal, NUL,
  backslashes, and Windows drive prefixes are rejected before authorization.
- Encoded absolute POSIX route paths preserve the legacy Flask app-level JSON
  404 shape rather than the download route's `File not found` shape.

## Reversal

Rollback is keeping `web_server.py` download helpers/routes and
`web_task_manager.Task.output_files` as the production owners. No Rust bridge is
introduced by this unit.
