# ASR Romaji Vocab CTC Decode Core Bootstrap

Date: 2026-07-18

## Unit

`asr_romaji_vocab_ctc_decode_core`

## Public Boundary

This unit owns the deterministic helper behavior in
`inference/romaji_asr/common.py`:

- `load_vocab`
- `decode_pred_ids`
- `decode_logits`
- `decode_outputs`
- `chunked`

It does not own `load_audio`, `create_session`, ONNX Runtime providers, session
metadata, audio file IO, resampling, padding/mask preparation, or model
execution.

## Source Evidence

Project source:

- `inference/romaji_asr/common.py`
- `inference/romaji_asr/runtime.py`
- `inference/romaji_asr/infer_dml.py`

Dependency evidence:

- `pyproject.toml` and `uv.lock` include `numpy`, `onnxruntime`, `soundfile`, and
  `scipy`.
- `third_party/sources/manifest.json` records the Python source snapshots for
  NumPy/SciPy/SoundFile.
- `third_party/sources/MISSING_SOURCES.md` records the ONNX Runtime upstream
  source fallback.

Only NumPy's public array behavior reaches this selected seam. ONNX Runtime,
SoundFile/libsndfile, and SciPy resampling remain outside this unit.

## Rust Boundary

Use an independent `v2m-core` module with:

- JSON vocab parsing and id inversion
- CTC duplicate collapse with blank reset semantics
- unknown token fallback to `<unk>`
- ndarray-backed 2D logits argmax with first-index tie and NumPy NaN behavior
- integer-vs-logit decode-output helpers
- chunking with `max(1, int(chunk_size))` semantics for fixture-backed JSON
  projections

`ndarray` is selected for array-shaped logits and batch outputs. It does not own
the Python-specific CTC/vocab/chunk policy.

## Fixture Harness

`fixtures/asr_romaji_vocab_ctc_decode_core.jsonl` contains 24 Python golden
cases:

- explicit `<blank>`, `PAD`, and default-zero blank lookup
- string, leading-zero, negative, and duplicate id vocab values
- CTC duplicate collapse and blank reset behavior
- unknown id fallback
- argmax tie-first behavior
- argmax NaN behavior
- integer and float `decode_outputs` batch dispatch
- `uint64` token ids above `i64::MAX`
- chunking for positive, zero, negative, float, string, and invalid `None`
  chunk sizes

`bootstrap/check_asr_romaji_vocab_ctc_decode_core.py` validates the fixture file
against the current uv Python environment.

## Kept Legacy

- ONNX Runtime sessions and provider selection
- model execution
- audio loading and resampling
- fake-session metadata and batch padding/mask preparation

## Writer Readiness

The unit is writer-ready after this bootstrap record. It should remain unwired
from production Python callers until a separate promotion unit changes runtime
ownership.
