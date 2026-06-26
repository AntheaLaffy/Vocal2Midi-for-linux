# exp12 ONNX DML Inference Bundle

This folder is a self-contained DirectML deployment bundle for the fastest current exp12 ONNX model.

Tested on Windows with Python 3.12.

## Files

- `model.onnx`
- `model.meta.json`
- `phoneme_vocab.json`
- `infer_dml.py`
- `benchmark_dml.py`
- `common.py`
- `requirements.txt`

## Install

```powershell
pip install -r requirements.txt
```

## Single-file inference

```powershell
python infer_dml.py --audio "your_audio.wav" --provider dml
```

## Manifest batch inference

```powershell
python infer_dml.py --manifest "dev.jsonl" --provider dml --batch_size 2
```

## Benchmark

```powershell
python benchmark_dml.py --manifest "dev.jsonl" --manifest_n 50 --providers dml,cpu --batch_size 2
```

## Notes

- Default model is the bundled `model.onnx`.
- The bundled model already outputs `pred_ids`, so it is optimized for greedy decoding speed.
- `batch_size=2` is the recommended deployment setting on the current machine.
- Recommended input format is mono WAV at 16 kHz, although the script will resample other supported formats.
