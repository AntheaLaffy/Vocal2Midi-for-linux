# Romaji ASR Model Bundle

This directory is the default model-asset path for the Japanese mora/romaji ASR
runtime. Users normally populate it through `download_models.py`; inference
code lives in `inference/romaji_asr/`.

## Files

| Path | Purpose |
| --- | --- |
| `model.onnx` | ONNX model downloaded as a release asset; it may be absent from a source checkout. |
| `model.meta.json` | Input/output metadata used to prepare fixed or dynamic batches. |
| `phoneme_vocab.json` | Token ID to phoneme vocabulary for CTC decoding. |

## Download

From the repository root:

```bash
uv run python download_models.py --only romaji
```

## Verify

Use a local audio file and select the provider supported by the platform:

```bash
uv run python -m inference.romaji_asr.infer_dml \
  --model experiments/romajiASR \
  --audio tests/jp-bpm-126.flac \
  --provider cpu
```

DirectML is available only on supported Windows ONNX Runtime installations.
Use `--provider cpu` on Linux and macOS.

## Current Limits

- The directory contains model data, not a self-contained Python environment.
- The model expects mono audio and the runtime normalizes supported input into
  its required sample format.
- Batch shape and output type are read from ONNX metadata and
  `model.meta.json`; do not assume a fixed batch size in callers.
- Model files may have separate distribution or usage terms from Vocal2Midi
  source code.
