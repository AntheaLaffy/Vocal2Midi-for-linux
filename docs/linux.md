# Linux Setup

This project supports a Linux CPU/ONNX workflow for the GUI and batch CLI.
It is still not a full Linux-native port.

## Choose the Right Linux Path

There are two different Linux setups in this repository:

| Goal | Recommended path |
| --- | --- |
| Run the Vocal2Midi GUI, batch CLI, HubertFA, GAME, RMVPE, Japanese mora ASR, and Qwen3-ASR on Linux | This document |
| Manually test or benchmark the upstream `Qwen/Qwen3-ASR-1.7B` package on Linux | [`docs/qwen-linux.md`](qwen-linux.md) |

Do not mix them by accident:

- this document is for the repository's Linux workflow, which now uses the official `qwen-asr` backend for Qwen3-ASR
- `docs/qwen-linux.md` is for manual standalone setup and direct package usage

## What Works

- GUI startup through `app_fluent.py`
- batch CLI in `scripts/slice_asr_cli.py`
- Qwen3-ASR through the official `qwen-asr` Transformers backend
- ONNX Runtime CPU execution for alignment, GAME, RMVPE, and romaji ASR
- folder opening from the GUI via `xdg-open` or `open`

## Current Limits

- DirectML is Windows-only
- the portable `python/` folder in this repository is Windows-specific
- official `qwen-asr` on CPU is much slower than the Windows DML/GGUF path for long slices
- the project-local Qwen3-ASR DML/GGUF compatibility layer remains Windows-oriented

## Recommended Python Environment

Use the repository-local `uv` environment. The project pins Python 3.12 in
`.python-version` and locks dependencies in `uv.lock`.

```bash
uv python install 3.12
uv python pin 3.12
uv sync
uv run python --version
```

Target Python version on Linux:

- required by this repository: Python 3.12
- avoid using Python 3.13+ for this repository unless you have already verified the full stack yourself

The lock file selects the CPU PyTorch index so Linux installs do not pull CUDA
runtime packages into the local environment.

Vendored third-party sources are managed under `third_party/`. This includes
Python sdists, upstream fallbacks for packages without sdists, native/FFI source
trees, and vendored Rust crates for Python native extensions. Regenerate them
after `uv sync`:

```bash
uv run python scripts/vendor_sources.py --force
uv run python scripts/vendor_native_sources.py --force
uv run python scripts/audit_vendored_sources.py
```

## System Packages

The default Linux target for this repository is now Arch Linux.
If you are on another distribution, use the package list below as a reference and map it to your distro.

Install these packages first:

- Python 3.12
- `pip`
- `ffmpeg`
- Qt runtime libraries
- build tools if you plan to compile `pyopenjtalk` or `llama.cpp`

Recommended Arch Linux packages:

```bash
sudo pacman -S --needed python python-pip ffmpeg libsndfile \
  mesa libxkbcommon xcb-util-cursor
```

If your Python is fully managed by `uv`, the system `python` and `python-pip` packages are less important,
but `ffmpeg`, `libsndfile`, and the Qt/X11 runtime libraries are still required.

Common optional Arch packages if PyQt5 startup is incomplete on your desktop:

```bash
sudo pacman -S --needed qt5-base gtk3
```

Fallback example for Debian/Ubuntu:

```bash
sudo apt update
sudo apt install -y python3 python3-pip python3-venv ffmpeg libsndfile1 \
  libgl1 libxkbcommon-x11-0 libxcb-cursor0
```

If PyQt5 still fails to start, you may need extra distro-specific X11 or Wayland runtime packages.

## Install

From the repository root:

```bash
uv python pin 3.12
uv sync
```

`uv.lock` installs the base dependencies plus `qwen-asr` for the official Linux
Qwen backend. `pyopenjtalk` remains optional on Linux and is only needed for the
Japanese G2P path.

If you use the repository helper scripts:

- `install.sh` and `run.sh` are `bash` scripts
- `install.sh` uses `uv sync` by default when `uv` is available
- set `VENDOR_SOURCES=1` with `install.sh` to refresh and audit all `third_party/` source mirrors
- explicit `PYTHON_BIN` / `PIP_BIN` values force the legacy pip fallback

```bash
chmod +x install.sh run.sh
./install.sh
```

## Run

```bash
./run.sh
```

Or:

```bash
uv run python app_fluent.py
```

## CLI

```bash
uv run python scripts/slice_asr_cli.py <input_dir> <output_dir> \
  --asr-model experiments/Qwen3-ASR-1.7B \
  --device cpu \
  --language zh
```

For Linux, start with `--device cpu`. The GUI also defaults to CPU on non-Windows systems.
The Linux Qwen path uses `experiments/Qwen3-ASR-1.7B` and the official `qwen-asr`
package, not the Windows DML/GGUF directory.

## Recommended Model Paths

Use the same model directories as Windows, but verify the files exist on disk:

- `experiments/GAME-1.0.3-medium-onnx`
- `experiments/1218_hfa_model_new_dict`
- `experiments/Qwen3-ASR-1.7B`
- `experiments/romajiASR`
- `experiments/RMVPE/rmvpe.onnx`

If the Japanese path is used without `pyopenjtalk`, the code falls back to a weaker internal G2P path.

## Troubleshooting

- `onnxruntime` import fails: reinstall in a clean virtual environment.
- `ffmpeg` read errors: install `ffmpeg` and verify `ffmpeg -version`.
- GUI opens but no models load: check the configured model paths in the settings panel.
- `install.sh` or `run.sh` uses the wrong Python: avoid setting `PYTHON_BIN`, or pass the exact interpreter path intentionally.
- `python --version` shows `3.13+`: run commands through `uv run python`, or rebuild the environment with `uv sync`.
- `fish: Unknown command: modelscope`: activate the venv with `source ~/.venv/bin/activate.fish`,
  add it with `fish_add_path -p ~/.venv/bin`, or call `~/.venv/bin/modelscope` directly.
- Qwen3-ASR import fails on Linux: confirm `qwen-asr`, `torch`, and the official model directory are installed.
