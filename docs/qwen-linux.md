# Linux Qwen3-ASR Environment

This is the official Linux setup for `Qwen/Qwen3-ASR-1.7B`.

Vocal2Midi now uses this official `qwen-asr` backend on Linux/macOS. If you want
the repository-level Linux setup, start from [`docs/linux.md`](linux.md).

## Recommended Setup

Create a clean Python 3.12 environment with `uv`:

```bash
uv python install 3.12
uv venv ~/.venv/qwen3-asr --python 3.12
source ~/.venv/qwen3-asr/bin/activate.fish
python --version
pip install -U pip setuptools wheel
```

If you want to replace your current `~/.venv` in place:

```bash
uv python install 3.12
uv venv ~/.venv --python 3.12 --clear
source ~/.venv/bin/activate.fish
python --version
```

If `fish` cannot find commands installed into the virtual environment, activate
the environment first:

```fish
source ~/.venv/bin/activate.fish
which python
which modelscope
```

If you want `fish` to expose `~/.venv/bin` in every new shell, add it once:

```fish
fish_add_path -p ~/.venv/bin
```

You can also bypass shell activation and call the tools by absolute path:

```fish
~/.venv/bin/python -m pip show modelscope
~/.venv/bin/modelscope --help
```

Install the official runtime:

```bash
pip install -U qwen-asr
```

If you want the faster vLLM backend:

```bash
pip install -U qwen-asr[vllm]
```

If you also want timestamp support with better throughput:

```bash
pip install -U flash-attn --no-build-isolation
```

For smaller machines:

```bash
MAX_JOBS=4 pip install -U flash-attn --no-build-isolation
```

## Model Download

The model card for `Qwen/Qwen3-ASR-1.7B` supports automatic download during
loading. If you need an offline local copy, download it first:

```bash
pip install -U "huggingface_hub[cli]"
huggingface-cli download Qwen/Qwen3-ASR-1.7B --local-dir ./Qwen3-ASR-1.7B
```

For users in Mainland China, the model card also lists ModelScope as an
alternative download path:

```bash
pip install -U modelscope
modelscope download --model Qwen/Qwen3-ASR-1.7B --local_dir ./Qwen3-ASR-1.7B
```

For this repository layout, a convenient local target is:

```bash
~/.venv/bin/python -m pip install -U modelscope qwen-asr
~/.venv/bin/modelscope download \
  --model Qwen/Qwen3-ASR-1.7B \
  --local_dir experiments/Qwen3-ASR-1.7B
```

## Quick Test

```python
import torch
from qwen_asr import Qwen3ASRModel

model = Qwen3ASRModel.from_pretrained(
    "Qwen/Qwen3-ASR-1.7B",
    dtype=torch.bfloat16,
    device_map="cuda:0",
    max_inference_batch_size=32,
    max_new_tokens=256,
    forced_aligner="Qwen/Qwen3-ForcedAligner-0.6B",
    forced_aligner_kwargs=dict(
        dtype=torch.bfloat16,
        device_map="cuda:0",
    ),
)

results = model.transcribe(
    audio="test.wav",
    language=None,
)

print(results[0].language)
print(results[0].text)
```

From the repository root, test the sample audio without using the project-local
`llama.cpp`/GGUF runtime by loading the official Transformers-format model
downloaded above:

```bash
~/.venv/bin/python - <<'PY'
import torch
from qwen_asr import Qwen3ASRModel

model = Qwen3ASRModel.from_pretrained(
    "experiments/Qwen3-ASR-1.7B",
    dtype=torch.float32,
    device_map="cpu",
    max_inference_batch_size=1,
    max_new_tokens=160,
)

results = model.transcribe(
    audio="tests/zh-bpm-98.flac",
    language="Chinese",
)

print(results[0].language)
print(results[0].text)
PY
```

## Notes

- This upstream path is GPU-oriented, but Vocal2Midi can still use it on CPU.
- The official `qwen-asr` path does not use this repository's
  `experiments/Qwen3-ASR-1.7B-dml` GGUF files and does not require
  `libllama.so`, `libggml.so`, or `libggml-base.so`.
- The official Qwen package provides both transformers and vLLM backends.
- Do not reuse a Python 3.13 venv for this setup; Qwen3-ASR should be pinned to Python 3.12 here.
- If your `fish` config replaces `python` and `pip`, verify `python --version` before installation.
- If your shell wraps `pip`, this document assumes that `pip install ...` is the supported entrypoint.
