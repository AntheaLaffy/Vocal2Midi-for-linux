# Vocal2Midi Rust Workspace

This Cargo workspace contains the Rust side of the gradual Vocal2Midi library
rewrite. It is nested under `rewrite-in-rust/` because the Python application is
still the user-facing runtime and the migration control plane owns promotion
state.

## Status

The Rust workspace contains verified compatibility implementations, but it is
not the default runtime for the whole application.

- Python remains the default owner for GUI, Web handlers, model inference,
  slicing orchestration, ASR, forced alignment, and most file I/O.
- `v2m-core` contains fixture-backed Rust implementations for all 66 units in
  the current migration inventory.
- `v2m-quant-bridge` can run quantization through Rust when Python explicitly
  selects the `rust-json` backend.
- Promotion decisions are recorded in `../manifest.yaml`, `../records/`, and
  `../reviews/`.

## Toolchain

- Rust edition: 2024
- MSRV: 1.85
- Workspace manifest: `rewrite-in-rust/rust/Cargo.toml`

Install the standard components:

```bash
rustup toolchain install stable
rustup component add rustfmt clippy
```

Run commands from the repository root unless a document says otherwise.

## Crates

### `v2m-core`

Library crate for compatibility behavior that can be tested without Python
runtime objects, model assets, GUI state, or live external effects.

Public modules are grouped by compatibility boundary:

| Area | Modules |
| --- | --- |
| Application and runtime | `application`, `slice_bounds`, `device` |
| Web contracts | `web_config`, `web_task`, `web_stream`, `web_pipeline_events`, `web_settings`, `web_filesystem_picker`, `web_output_download`, `web_model_download*` |
| Batch and model assets | `batch_cli_*`, `download_models_*` |
| Export and quantization | `export`, `midi_export`, `ustx_*`, `quant` |
| Slicing | `slice_method`, `slicer_*` |
| Lyrics and HubertFA | `lyric_*`, `zh_g2p`, `ja_g2p`, `hfa_*`, `game` |
| ASR preprocessing | `asr_*` |

These modules do not own GUI, Flask/SocketIO routing, filesystem or network
effects, model sessions, or inference orchestration unless a module contract
explicitly models an injected effect result. Public APIs must name the Python
compatibility source and keep caller-input failures recoverable.

### `v2m-quant-bridge`

Binary bridge used by Python when quantization is explicitly routed to Rust.
The bridge reads one JSON request from stdin and writes one JSON response to
stdout.

Request shape:

```json
{
  "version": 1,
  "mode": "simple",
  "tempo": 120.0,
  "quantization_step": 16,
  "notes": [
    {"index": 0, "onset": 0.0, "offset": 0.5, "pitch": 60.0, "lyric": "la"}
  ]
}
```

Success response shape:

```json
{
  "ok": true,
  "applied": true,
  "notes": [
    {"index": 0, "onset": 0.0, "offset": 0.5}
  ]
}
```

Failure response shape:

```json
{
  "ok": false,
  "notes": [],
  "error": {
    "code": "invalid_json",
    "message": "invalid request JSON: ..."
  }
}
```

The bridge contract is intentionally small: no long-running server process, no
global runtime state, and no direct GUI/Web dependency.

## Development Commands

Format check:

```bash
cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check
```

Lint:

```bash
cargo clippy --manifest-path rewrite-in-rust/rust/Cargo.toml --workspace --all-targets --all-features -- -D warnings
```

Test:

```bash
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml --workspace --all-features
```

Build the quantization bridge:

```bash
cargo build --manifest-path rewrite-in-rust/rust/Cargo.toml --bin v2m-quant-bridge
```

Generate docs with rustdoc warnings as errors:

```bash
RUSTDOCFLAGS="-D warnings" cargo doc --manifest-path rewrite-in-rust/rust/Cargo.toml --workspace --all-features --no-deps
```

Run the Python bridge bootstrap check:

```bash
uv run python rewrite-in-rust/bootstrap/check_quantization_bridge_bootstrap.py
```

## Documentation Rules

- Start crate and module docs with the compatibility boundary.
- For public `Result` APIs, include an `# Errors` section.
- For APIs that can panic by contract, include a `# Panics` section.
- Prefer small examples when they do not require Python, model assets, or
  generated files.
- Do not document verified units as default runtime owners until the manifest
  promotes them.
- Keep README examples copyable from the repository root.
- Use intra-doc links for Rust items and keep broken links as hard failures.
- Treat every public type, field, variant, constant, and function as a
  maintained compatibility API.

The repository-wide policy and exact documentation checks are in
[`docs/documentation.md`](../../docs/documentation.md). The normal `cargo doc`
gate rejects rustdoc warnings, including the crate-level `missing_docs` lint.
Run the explicit audit when changing documentation lint policy:

```bash
RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc \
  --manifest-path rewrite-in-rust/rust/Cargo.toml \
  --workspace --all-features --no-deps
```

## Safety

Migration crates should avoid `unsafe`. If a future unit needs `unsafe`, add a
record under `../records/` that states the invariant, why a safe API is
insufficient, which tests cover it, and which review role accepted it.

## Release Boundary

These crates are marked `publish = false`. They are internal implementation
artifacts for Vocal2Midi until the project chooses a separate crate release
policy.
