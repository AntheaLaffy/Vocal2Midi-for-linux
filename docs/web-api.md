# Web API Contract

This document describes the current Flask + SocketIO backend implemented in
`web_server.py`, `web_task_manager.py`, and `web_model_download_manager.py`.
It is written as a maintenance contract for the local Web UI.

## Current Limits

- The backend has no authentication layer.
- The default bind host is `0.0.0.0`; use it on trusted local networks only.
- Uploads are limited to 500 MB by Flask configuration.
- Pipeline cancellation is cooperative and finishes at the next cancellation
  check inside the pipeline.
- Model downloads may start subprocesses and use network access.

See [`SECURITY.md`](../SECURITY.md) for private vulnerability reporting and the
supported security boundary.

## Run

```bash
V2M_WEB_PORT=5001 uv run python web_server.py
```

Default base URL:

```text
http://localhost:5000
```

## Response Shape

Successful JSON responses include:

```json
{
  "success": true
}
```

Failed JSON responses include:

```json
{
  "success": false,
  "error": "message"
}
```

Timestamps are ISO 8601 strings produced by Python `datetime.isoformat()`.

## Pipeline Endpoints

### `POST /api/pipeline/start`

Starts one audio processing task.

Content type: `multipart/form-data`

Fields:

| Field | Required | Description |
| --- | --- | --- |
| `audio_file` | yes | Audio upload with `.wav`, `.m4a`, `.flac`, `.mp3`, or `.ogg` extension. |
| `config` | no | JSON string with per-run pipeline overrides. Defaults to `{}`. |

The server merges persisted settings first, then applies the request `config`.

Success:

```json
{
  "success": true,
  "task_id": "uuid",
  "status": "running",
  "message": "Task started successfully. Processing will begin shortly..."
}
```

Errors:

| Status | Cause |
| --- | --- |
| `400` | Missing file, empty filename, unsupported extension, or invalid JSON. |
| `413` | Upload exceeds 500 MB. |
| `500` | Task creation or startup failed unexpectedly. |

### `POST /api/pipeline/stop`

Requests cooperative cancellation for a running pipeline task.

Body:

```json
{
  "task_id": "uuid"
}
```

Success:

```json
{
  "success": true,
  "status": "stopping",
  "message": "Stop request sent. Task will finish current operation then stop."
}
```

Errors:

| Status | Cause |
| --- | --- |
| `400` | Missing `task_id` or task is not stoppable. |
| `404` | Task does not exist. |

### `GET /api/pipeline/status/<task_id>`

Returns full task state.

Success fields:

| Field | Description |
| --- | --- |
| `task_id` | Task UUID. |
| `status` | `pending`, `running`, `completed`, `failed`, or `cancelled`. |
| `progress` | Integer percentage. |
| `stage` | Current pipeline stage label. |
| `created_at` | Creation timestamp. |
| `started_at` | Start timestamp, or `null`. |
| `completed_at` | Completion timestamp, or `null`. |
| `error` | Error string, or `null`. |
| `output_files` | Files registered for download. |

Errors:

| Status | Cause |
| --- | --- |
| `404` | Task does not exist. |

### `GET /api/pipeline/list`

Returns task summaries:

```json
{
  "success": true,
  "tasks": [],
  "count": 0
}
```

## Settings Endpoints

### `GET /api/settings`

Returns persisted Web settings merged with defaults.

Top-level sections:

- `models`
- `params`
- `debug`
- `pipeline`
- `downloads`

### `PUT /api/settings`

Updates known top-level settings sections and persists them to
`settings/web_settings.json` unless `V2M_WEB_SETTINGS_FILE` overrides the path.

Body example:

```json
{
  "params": {
    "seg_threshold": 0.2,
    "slice_min": 8.0
  },
  "debug": {
    "export_txt": true
  }
}
```

Known top-level sections must be JSON objects. Unknown top-level sections are
ignored by the current implementation.

Errors:

| Status | Cause |
| --- | --- |
| `400` | Request body is invalid JSON or a known section is not an object. |
| `500` | Settings could not be saved. |

### `POST /api/settings/reset`

Replaces current settings with defaults and persists them.

## Filesystem Picker Endpoints

These endpoints support the local Web UI path picker. They expose local
filesystem paths and should not be exposed to untrusted clients.

### `GET /api/filesystem/roots`

Returns useful root directories:

```json
{
  "success": true,
  "separator": "/",
  "roots": []
}
```

### `GET /api/filesystem/list`

Query parameters:

| Name | Default | Description |
| --- | --- | --- |
| `path` | project root | Directory to list. Relative paths resolve from the project root. |
| `mode` | `directory` | `directory` or `file`. |
| `extensions` | empty | Comma-separated file extensions used when `mode=file`. |

Directories are listed before files and entries are sorted by lowercase name.

Errors:

| Status | Cause |
| --- | --- |
| `400` | Invalid mode or unreadable directory. |
| `404` | Path does not exist. |

## System Endpoint

### `GET /api/system/info`

Returns backend version, Python version, platform name, visible runtime device
choices, and count of currently running pipeline tasks.

## Model Endpoints

### `GET /api/models/status`

Returns known model assets, local install status, install counts, and the active
download task if one exists.

Model ids currently include:

- `game`
- `hfa`
- `rmvpe`
- `romaji`
- `qwen`

### `POST /api/models/download`

Starts one background model download task.

Body fields:

| Field | Default | Description |
| --- | --- | --- |
| `models` | all missing models | List of model ids. |
| `qwen_source` | `auto` | `auto`, `modelscope`, or `huggingface`. |
| `force` | `false` | Redownload even if assets are present. |
| `proxy_mode` | `system` | `system`, `manual`, or `none`. |
| `proxy_url` | empty | Required when `proxy_mode=manual`; must include a scheme. |

Success:

```json
{
  "success": true,
  "task_id": "uuid",
  "status": "running",
  "message": "Model download task started"
}
```

Errors:

| Status | Cause |
| --- | --- |
| `400` | Invalid model list, source, proxy mode, or proxy URL. |
| `409` | Another model download task is already active. |

### `GET /api/models/download/status/<task_id>`

Returns serialized model download task state.

Fields include:

- `task_id`
- `task_type`
- `status`
- `progress`
- `stage`
- `selected_models`
- `qwen_source`
- `force`
- `proxy_mode`
- `proxy_url`
- `created_at`
- `started_at`
- `completed_at`
- `error`
- `returncode`
- `logs`

### `POST /api/models/download/stop`

Requests cancellation for a model download task and terminates the child
download process group when possible.

Body:

```json
{
  "task_id": "uuid"
}
```

Errors:

| Status | Cause |
| --- | --- |
| `400` | Missing `task_id` or task is not stoppable. |
| `404` | Download task does not exist. |

## Output Downloads

### `GET /api/download/<path>`

Downloads a file only when that exact path is registered in a pipeline task's
`output_files`.

Security properties:

- absolute paths are rejected
- `..` traversal is rejected
- backslashes and NUL bytes are rejected
- Windows drive prefixes are rejected
- unregistered files are rejected

## SocketIO Events

### Client to Server

| Event | Payload | Description |
| --- | --- | --- |
| `join_task` | `{ "task_id": "uuid" }` | Joins a task room and receives current status plus backlog logs. |
| `leave_task` | `{ "task_id": "uuid" }` | Leaves a task room. |
| `stop_task` | `{ "task_id": "uuid" }` | Stops either a pipeline task or model download task. |

### Server to Client

| Event | Payload | Description |
| --- | --- | --- |
| `connected` | server time and message | Emitted after WebSocket connection. |
| `joined` | task id | Confirms room join. |
| `left` | task id | Confirms room leave. |
| `log` | log entry | Streaming task log. |
| `progress` | task id, progress, stage | Streaming progress update. |
| `status_change` | task status payload | Terminal and intermediate status updates. |
| `backlogs` | task id and logs | Existing logs sent after joining a task room. |
| `stop_response` | task id, task type, success, message | Response to `stop_task`. |
| `error` | message | Socket-level request error. |

Pipeline log entries include `task_id`, `message`, `level`, and `timestamp`.
Model download log entries also include `task_type: "model_download"`.

## Compatibility Rules

When changing this API:

- keep response field names stable unless the frontend and tests change together
- add new fields instead of repurposing old fields
- update `tests/test_web_api.py` for unit-level behavior
- run the integration script when changing task lifecycle or SocketIO behavior
