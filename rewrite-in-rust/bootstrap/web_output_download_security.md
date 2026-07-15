# web_output_download_security Bootstrap

## Boundary

`web_output_download_security` covers only output download authorization in
`web_server.py`:

```text
WINDOWS_DRIVE_RE
_safe_requested_download_path
_authorized_output_file
GET /api/download/<path>
```

The public compatibility surface is:

- empty requested paths are rejected;
- NUL bytes are rejected;
- backslashes are rejected;
- Windows drive prefixes using `/` or `\` are rejected on every platform;
- absolute POSIX paths are rejected;
- any `..` traversal part is rejected;
- accepted URL paths resolve under `PROJECT_ROOT`;
- Flask percent-decodes route path captures before `download_file(filepath)` is
  invoked;
- encoded absolute POSIX paths can miss the Flask `<path>` route and return the
  app-level `{"success": false, "error": "Resource not found"}` 404 shape;
- requested files must exist before task output authorization;
- a request is authorized only when its resolved path exactly equals one
  registered task output path after canonical resolution;
- relative task outputs resolve from `PROJECT_ROOT`;
- absolute task outputs are compared by canonical path;
- failed downloads return status `404` with `{"error": "File not found"}`;
- successful downloads use `send_file(..., as_attachment=True,
  download_name=safe_path.name)`.

The unit does not cover filesystem picker behavior, pipeline output collection,
task registry lifecycle, Flask route replacement, real file streaming internals,
or browser UI behavior.

## Dependency Expansion

The selected source uses:

- stdlib: `re`, `pathlib.PurePosixPath`, `pathlib.Path.resolve`, file existence
  checks, and path absoluteness;
- local: `web_task_manager.Task.output_files` and the task manager lock;
- Flask route wrappers: `jsonify` and `send_file`.

Fixture parity uses temporary project roots, synthetic files, synthetic
symlinks, synthetic task outputs, and Flask's test client. No live Web server,
SocketIO, model runtime, pipeline execution, network, or browser frontend is
needed.

## Seam

Use the default rewrite seam:

- kind: independent Rust library
- crate: `rewrite-in-rust/rust/crates/v2m-core`
- runtime owner: legacy Python
- bridge dependencies: none

The Rust side models path validation, registered-output authorization, and the
route response metadata from explicit fixture inputs. No production bridge is
introduced.

## Fixture Harness

Python and Rust tests consume:

```text
rewrite-in-rust/fixtures/web_output_download_security.jsonl
```

The legacy Python side is checked by:

```bash
uv run python rewrite-in-rust/bootstrap/check_web_output_download_security.py
```

## Rollback

Rollback is keeping production ownership unchanged:

```text
web_server.py download helpers/routes and web_task_manager.Task.output_files
```

No Web caller should import Rust output for this unit until a later promotion
record chooses and verifies a bridge.
