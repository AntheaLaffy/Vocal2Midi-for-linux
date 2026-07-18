# Romaji ASR Test Bundle

This directory is a development-only model fixture. Production defaults use
`experiments/romajiASR`; no application, GUI, Web, or CLI setting points to this
directory automatically.

## Files

| Path | Purpose |
| --- | --- |
| `model.onnx` | Optional local ONNX model; it may be absent from a source checkout. |
| `model.meta.json` | Model input/output metadata. |
| `phoneme_vocab.json` | Token ID to phoneme vocabulary. |

## Verify

From the repository root:

```bash
uv run python -m inference.romaji_asr.infer_dml \
  --model experiments/romajiASR_test \
  --audio tests/jp-bpm-126.flac \
  --provider cpu
```

## Current Limits

- This duplicate asset directory is not part of the supported runtime path.
- Keep it only while a test or comparison explicitly names it.
- Do not update production documentation or defaults to use this directory.
- Model files may have separate distribution or usage terms from Vocal2Midi
  source code.
