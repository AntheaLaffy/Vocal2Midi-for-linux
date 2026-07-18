# Vocal2Midi

[English](README.md) | [日本語](README.ja.md) | [简体中文](README.zh-CN.md)

Vocal2Midi turns vocal recordings into lyric-aligned MIDI, USTX, TextGrid, and
supporting editing artifacts. It includes a Fluent desktop GUI, a local Web UI,
a batch slice + ASR CLI, and an ONNX-first inference pipeline.

## Project Status

- The active runtime is ONNX-first.
- A gradual Rust rewrite is underway under `rewrite-in-rust/`. The current
  user-facing application is still Python-led; verified Rust units are promoted
  only behind explicit compatibility and rollback checks.
- Windows uses DirectML for ONNX models when available.
- Windows Qwen3-ASR uses the project-local ONNX encoder + GGUF/`llama.cpp`
  decoder path.
- Linux and macOS Qwen3-ASR use the official `qwen-asr` Transformers backend.
- Linux and macOS use standard ONNX Runtime CPU execution for the non-Qwen
  ONNX models.
- Model assets are expected to live under `experiments/` or another configured
  local path.
- Some legacy function names and compatibility branches remain while the
  codebase is being cleaned up.

## For Users

### Install

The preferred development environment uses Python 3.12 and `uv`:

```bash
uv python install 3.12
uv python pin 3.12
uv sync
```

Platform helpers are also available:

```bash
./install.sh
./run.sh
```

Windows portable setup:

```bat
install.bat
run.bat
```

Linux-specific setup notes are in [docs/linux.md](docs/linux.md). Standalone
upstream Qwen3-ASR setup notes are in [docs/qwen-linux.md](docs/qwen-linux.md).

### Download Models

Show the model download plan:

```bash
uv run python download_models.py --list
```

Download missing model assets:

```bash
uv run python download_models.py
```

Choose the Qwen3-ASR source explicitly when needed:

```bash
uv run python download_models.py --qwen-source modelscope
uv run python download_models.py --qwen-source huggingface
```

Default model paths:

| Component | Default path |
| --- | --- |
| GAME | `experiments/GAME-1.0.3-medium-onnx` |
| HubertFA | `experiments/1218_hfa_model_new_dict` |
| Qwen3-ASR on Linux/macOS/Web | `experiments/Qwen3-ASR-1.7B` |
| Qwen3-ASR on Windows desktop | `experiments/Qwen3-ASR-1.7B-dml` |
| Japanese mora ASR | `experiments/romajiASR` |
| RMVPE | `experiments/RMVPE/rmvpe.onnx` |

You can change model paths in the desktop GUI settings panel or the Web UI
settings page.

### Desktop GUI

Start the desktop GUI:

```bash
uv run python app_fluent.py
```

The desktop GUI is the primary interactive workflow. It lets you choose model
paths, select runtime device, configure slicing, set language and lyric mode,
provide optional reference lyrics, and export MIDI/USTX/debug artifacts.

### Web UI

Start the local Web backend:

```bash
uv run python web_server.py
```

Then open:

```text
http://localhost:5000
```

Use a custom port when needed:

```bash
V2M_WEB_PORT=5001 uv run python web_server.py
```

The Web API contract is documented in [docs/web-api.md](docs/web-api.md).

### Batch CLI

Run folder-based slice + ASR processing:

```bash
uv run python scripts/slice_asr_cli.py <input_dir> <output_dir> \
  --asr-model experiments/Qwen3-ASR-1.7B \
  --device cpu \
  --language zh
```

On Windows desktop setups, use the Windows Qwen path and DirectML device when
that model directory is available:

```bash
uv run python scripts/slice_asr_cli.py <input_dir> <output_dir> \
  --asr-model experiments/Qwen3-ASR-1.7B-dml \
  --device dml \
  --language zh
```

Useful options:

```text
--no-slice              bypass slicing and send the whole file to ASR
--asr-batch-size        ASR batch size
--file-batch-size       number of audio files per batch
--rmvpe-model           enable RMVPE-assisted smart slicing
--rmvpe-batch-size      RMVPE batch size
--keep-model            keep the ASR runtime alive across the batch
--keep-rmvpe            keep the RMVPE runtime alive across the batch
--save-json             save slice timing and ASR outputs as JSON
--no-recursive          scan only the top level
--no-skip-existing      force reprocessing of existing outputs
```

### Outputs

Depending on the selected workflow, Vocal2Midi can export:

- `.mid`
- `.ustx`
- `.txt`
- `.csv`
- `TextGrid`
- chunk `.wav` files
- `.lab`
- ASR matching logs

## For Developers

### Architecture

The intended dependency direction is:

```text
gui -> application -> inference
web -> application -> inference
```

Key areas:

- `application/`: application-layer configuration, validation, and job entrypoints
- `gui/`: PyQt5 + qfluentwidgets desktop UI
- `web_server.py`: Flask + SocketIO local Web backend
- `inference/`: ASR, alignment, pitch extraction, slicing, quantization, export
- `scripts/`: model/source maintenance and batch CLI helpers
- `tests/`: automated tests

Architecture notes are in [docs/architecture.md](docs/architecture.md).
Development workflow and documentation rules are in
[docs/contributing.md](docs/contributing.md).

### Rust Workspace

Rust migration work lives in [rewrite-in-rust/](rewrite-in-rust/). The Cargo
workspace is intentionally nested at [rewrite-in-rust/rust/](rewrite-in-rust/rust/)
so Rust library units can be tested without starting the desktop GUI, Web
backend, or full model pipeline.

Common Rust checks:

```bash
cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check
cargo clippy --manifest-path rewrite-in-rust/rust/Cargo.toml --workspace --all-targets --all-features -- -D warnings
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml --workspace --all-features
RUSTDOCFLAGS="-D warnings" cargo doc --manifest-path rewrite-in-rust/rust/Cargo.toml --workspace --all-features --no-deps
```

The Rust workspace README documents MSRV, crate boundaries, JSON bridge
contracts, and migration-owner rules:
[rewrite-in-rust/rust/README.md](rewrite-in-rust/rust/README.md).

### Tests

Run the focused Web/API test suite:

```bash
uv run pytest tests/test_web_api.py
```

Run all automated tests:

```bash
uv run pytest
```

Manual integration test. Start `web_server.py` first, then run:

```bash
uv run python tests/test_api_integration.py
```

### Source Mirrors

Vendored third-party source mirrors live under `third_party/`. Refresh and audit
them with:

```bash
uv run python scripts/vendor_sources.py --force
uv run python scripts/vendor_native_sources.py --force
uv run python scripts/audit_vendored_sources.py
```

## Documentation

- Linux setup: [docs/linux.md](docs/linux.md)
- Qwen3-ASR Linux notes: [docs/qwen-linux.md](docs/qwen-linux.md)
- Architecture: [docs/architecture.md](docs/architecture.md)
- Development guide: [docs/contributing.md](docs/contributing.md)
- Contribution entrypoint: [CONTRIBUTING.md](CONTRIBUTING.md)
- Documentation policy: [docs/documentation.md](docs/documentation.md)
- Web API contract: [docs/web-api.md](docs/web-api.md)
- Security policy: [SECURITY.md](SECURITY.md)
- Third-party credits: [ACKNOWLEDGEMENTS.md](ACKNOWLEDGEMENTS.md)

## License

Vocal2Midi is distributed under the Apache License 2.0. See [LICENSE](LICENSE).

Third-party components, vendored code, model assets, dictionaries, and embedded
materials may carry their own original licenses, notices, or attribution
requirements. Those notices remain applicable to the corresponding materials.
