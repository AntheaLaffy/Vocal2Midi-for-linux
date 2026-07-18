# Development Guide

This guide is for maintainers and contributors working on Vocal2Midi code.
User-facing setup instructions live in the platform documents under `docs/`.
The shorter repository entrypoint is [`CONTRIBUTING.md`](../CONTRIBUTING.md),
and the full documentation policy is
[`docs/documentation.md`](documentation.md).

The documentation style follows the habits common in Rust projects:

- start with the contract a caller can rely on
- document limits and errors before edge cases surprise users
- keep examples copyable from the repository root
- update tests and docs in the same change when behavior changes

## Supported Environment

The Linux development path is the primary target for this repository snapshot.

- Python: `>=3.12,<3.13`
- Environment manager: `uv`
- Rust: stable toolchain with MSRV `1.85`
- Cargo workspace: `rewrite-in-rust/rust/Cargo.toml`
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

Rust workspace setup:

```bash
rustup toolchain install stable
rustup component add rustfmt clippy
cargo --version
rustc --version
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

## Rust Contributions

Rust code lives under `rewrite-in-rust/rust/` and follows the same owner model
as the migration manifest:

- `v2m-core` holds fixture-backed application/Web contracts, deterministic
  processing and export helpers, and inference-adjacent preprocessing that does
  not create model sessions.
- `v2m-quant-bridge` is a JSON stdin/stdout bridge for explicitly selected
  quantization runs.
- Python remains the default runtime owner until a manifest unit is promoted.
- Public Rust APIs should document their compatibility source, accepted inputs,
  return shape, and error behavior.
- Use `Result<T, E>` for recoverable boundary failures; do not panic on caller
  input.
- Keep `unsafe` out of migration units unless a record explains the exact
  invariant and review path.
- Do not add business logic to `rewrite-in-rust/` control-plane documents,
  skills, manifests, or review records.

Run Rust commands from the repository root with `--manifest-path` so the nested
workspace is explicit:

```bash
cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check
cargo clippy --manifest-path rewrite-in-rust/rust/Cargo.toml --workspace --all-targets --all-features -- -D warnings
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml --workspace --all-features
RUSTDOCFLAGS="-D warnings" cargo doc --manifest-path rewrite-in-rust/rust/Cargo.toml --workspace --all-features --no-deps
```

For changes crossing the Python/Rust seam, also run the narrow Python parity
tests named by the manifest unit. Keep fixture updates, Rust code, Python seam
code, and docs in the same change when public behavior changes.

## Documentation Standard

[`docs/documentation.md`](documentation.md) is authoritative for document
ownership, historical migration evidence, Markdown structure, rustdoc, and
verification. The summary below applies to code review.

For Markdown documents:

- begin with the document purpose and target reader
- state current limits in their own section
- use repository-root-relative paths
- mark commands with `bash`, `fish`, or `python`
- include the command that verifies the documented behavior
- avoid describing planned behavior as if it already exists

Validate repository-local Markdown targets after moving files or changing
links:

```bash
uv run python scripts/check_markdown_links.py
```

For public Python functions, classes, and dataclasses:

- first sentence: what contract the item provides
- `Args`: caller-provided values and accepted shapes
- `Returns`: stable return shape
- `Raises`: user-visible or boundary-crossing errors
- mention cancellation, filesystem writes, subprocesses, or network access when
  the function performs them

For public Rust modules, types, and functions:

- start crate and module docs with the compatibility boundary
- document the Python source being mirrored while Python remains runtime owner
- include `# Errors` when a function returns `Result`
- include `# Panics` only when panics are part of the contract; otherwise handle
  caller input as data
- include `# Examples` for stable public APIs when a short copyable example is
  possible
- avoid documenting future promotion as current default behavior

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

Rust migration checks:

```bash
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml --workspace --all-features
uv run python rewrite-in-rust/bootstrap/check_quantization_bridge_bootstrap.py
```

Run the full Rust style gate before promoting or reviewing a Rust unit:

```bash
cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check
cargo clippy --manifest-path rewrite-in-rust/rust/Cargo.toml --workspace --all-targets --all-features -- -D warnings
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml --workspace --all-features
RUSTDOCFLAGS="-D warnings" cargo doc --manifest-path rewrite-in-rust/rust/Cargo.toml --workspace --all-features --no-deps
```

## Review Checklist

Before handing off a change:

- run the narrowest relevant test command
- run `cargo fmt`, `cargo clippy`, `cargo test`, and `cargo doc` for Rust
  changes
- update `docs/web-api.md` when REST or SocketIO behavior changes
- update `docs/architecture.md` when a module boundary changes
- update `docs/linux.md` when setup, model paths, or runtime assumptions change
- update `docs/documentation.md` when document ownership or verification rules
  change
- update `rewrite-in-rust/manifest.yaml`, `rewrite-in-rust/records/`, and the
  relevant review report when a Rust migration state changes
- keep generated model files, local settings, and output artifacts out of review
- do not weaken path traversal checks for filesystem or download endpoints
