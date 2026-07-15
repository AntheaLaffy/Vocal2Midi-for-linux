# web_model_download_request_catalog_contract Bootstrap

## Boundary

`web_model_download_request_catalog_contract` covers only the public request and
catalog behavior around the Web model-download endpoints:

```text
GET /api/models/status
POST /api/models/download
GET /api/models/download/status/<task_id>
POST /api/models/download/stop
ModelDownloadManager.model_statuses
ModelDownloadManager.serialize_task
validate_model_request
_redact_proxy_url
```

The compatibility surface is:

- known model ids are `game`, `hfa`, `rmvpe`, `romaji`, and `qwen`;
- catalog entries expose `id`, `name`, `role`, `description`, `target_path`,
  `marker`, `source`, `installed`, and `required`;
- installed and missing counts are derived from catalog entries;
- active tasks serialize as `task_type: model_download` with task timestamps,
  selected models, qwen source, force, proxy mode, redacted proxy URL, error,
  return code, and logs;
- missing `models` or JSON `null` defaults to all currently missing models;
- `models` must be a list of strings;
- duplicate model ids are deduplicated while preserving first-seen order;
- empty selections are rejected;
- qwen source, proxy mode, and manual proxy URL validation preserve legacy error
  strings;
- `force` follows Python truthiness for JSON values because the route calls
  `bool(data.get("force", False))`;
- active download conflicts return status `409`;
- missing status tasks return status `404`;
- stop requests preserve missing id, missing task, cannot stop, and success
  response shapes.

The unit does not cover subprocess command construction, proxy environment
shaping, output parsing, progress math, SocketIO event emission, active-task
lock correctness, thread creation, cancellation transitions, process-tree
termination, actual downloads, or archive safety.

## Dependency Expansion

The selected behavior uses:

- stdlib: dictionaries, dataclasses, datetime ISO formatting, URL string
  splitting, and Python truthiness;
- local metadata: `download_models.GITHUB_MODELS`, `QWEN_LOCAL_DIR`, and
  `QWEN_MODEL_ID`;
- local helpers: `target_has_model`, `qwen_has_weights`,
  `validate_model_request`, and `_redact_proxy_url`;
- Flask route wrappers for JSON status codes.

Fixture parity injects install states, fake active tasks, and fake
task-manager outcomes. No live Web server, SocketIO transport, model runtime,
download process, network, or archive extraction is needed.

## Seam

Use the default rewrite seam:

- kind: independent Rust library
- crate: `rewrite-in-rust/rust/crates/v2m-core`
- runtime owner: legacy Python
- bridge dependencies: none

The Rust side models the request/catalog contract from explicit JSON fixture
inputs. No production bridge is introduced.

## Fixture Harness

Python and Rust tests consume:

```text
rewrite-in-rust/fixtures/web_model_download_request_catalog_contract.jsonl
```

The legacy Python side is checked by:

```bash
uv run python rewrite-in-rust/bootstrap/check_web_model_download_request_catalog_contract.py
```

## Rollback

Rollback is keeping production ownership unchanged:

```text
web_server.py model download routes
web_model_download_manager.ModelDownloadManager
download_models.py
```

No Web caller should import Rust output for this unit until a later promotion
record chooses and verifies a bridge.
