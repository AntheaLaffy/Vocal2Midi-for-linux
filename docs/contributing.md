# Development Guide

This guide is for maintainers and contributors working on Vocal2Midi code.
User-facing setup instructions live in the platform documents under `docs/`.

The documentation style follows the habits common in Rust projects:

- start with the contract a caller can rely on
- document limits and errors before edge cases surprise users
- keep examples copyable from the repository root
- update tests and docs in the same change when behavior changes

## Supported Environment

The Linux development path is the primary target for this repository snapshot.

- Python: `>=3.12,<3.13`
- Environment manager: `uv`
- GUI: PyQt5 / pyqt-fluent-widgets
- Web backend: Flask + Flask-SocketIO
- Inference runtime on Linux: ONNX Runtime CPU plus the official `qwen-asr`
  package for Qwen3-ASR

Do not assume CUDA is available on Linux. Device names accepted by legacy
interfaces are normalized by `inference/device_utils.py`.

## Setup

Run these commands from the repository root:

```bash
uv python install 3.12
uv python pin 3.12
uv sync
uv run python --version
```

Optional source mirror audit:

```bash
uv run python scripts/vendor_sources.py --force
uv run python scripts/vendor_native_sources.py --force
uv run python scripts/audit_vendored_sources.py
```

## Run Locally

Desktop GUI:

```bash
uv run python app_fluent.py
```

Web backend:

```bash
V2M_WEB_PORT=5001 uv run python web_server.py
```

Batch CLI:

```bash
uv run python scripts/slice_asr_cli.py <input_dir> <output_dir> \
  --asr-model experiments/Qwen3-ASR-1.7B \
  --device cpu \
  --language zh
```

## Dependency Boundaries

The intended direction is:

```text
gui -> application -> inference
web -> application -> inference
```

Keep these rules intact:

- GUI and Web handlers collect input, report progress, and translate UI state.
- `application/` validates configuration and owns user-facing errors.
- `inference/` owns model loading, audio processing, alignment, note extraction,
  quantization, and exports.
- Shared runtime naming belongs in `inference/device_utils.py`.
- Public entrypoints should accept stable dataclasses or dictionaries rather
  than long positional parameter lists.

When adding a new feature, prefer placing validation at the application boundary
instead of duplicating it separately in every UI.

## Documentation Standard

For Markdown documents:

- begin with the document purpose and target reader
- state current limits in their own section
- use repository-root-relative paths
- mark commands with `bash`, `fish`, or `python`
- include the command that verifies the documented behavior
- avoid describing planned behavior as if it already exists

For public Python functions, classes, and dataclasses:

- first sentence: what contract the item provides
- `Args`: caller-provided values and accepted shapes
- `Returns`: stable return shape
- `Raises`: user-visible or boundary-crossing errors
- mention cancellation, filesystem writes, subprocesses, or network access when
  the function performs them

For endpoints and event streams, document:

- request method and content type
- required fields
- success response shape
- non-success status codes
- emitted WebSocket events, if any

## Error Handling

User-facing pipeline errors should flow through `application.exceptions`.

- `ModelNotFoundError`: required model paths are missing or invalid.
- `CancellationError`: the user cancelled the operation.
- `Vocal2MidiError`: base application error for pipeline failures.

REST endpoints should return JSON with `success: false` and an `error` string.
Use HTTP status codes to separate caller errors from server failures.

## Testing

Fast Web API tests:

```bash
uv run pytest tests/test_web_api.py
```

Full test suite:

```bash
uv run pytest
```

Manual integration script. Start `web_server.py` first, then run:

```bash
uv run python tests/test_api_integration.py
```

The integration script uses real audio files under `tests/` and requires the
server to be reachable at `http://localhost:5000` unless the script is changed.

## Review Checklist

Before handing off a change:

- run the narrowest relevant test command
- update `docs/web-api.md` when REST or SocketIO behavior changes
- update `docs/architecture.md` when a module boundary changes
- update `docs/linux.md` when setup, model paths, or runtime assumptions change
- keep generated model files, local settings, and output artifacts out of review
- do not weaken path traversal checks for filesystem or download endpoints
