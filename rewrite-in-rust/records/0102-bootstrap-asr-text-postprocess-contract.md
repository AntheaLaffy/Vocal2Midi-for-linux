# 0102 - Bootstrap ASR Text Postprocess Contract

Date: 2026-07-18

## Decision

Split `asr_text_postprocess_contract` before writer work.

The provisional unit was too broad. It combined pure text normalization, small
schema/default contracts, WAV PCM decoding, SciPy resampling, romaji vocab/CTC
decode, and ONNX-session-adjacent batch metadata. Those surfaces should not
share one Rust writer unit because they have different dependency evidence,
fixture shapes, review risks, and rollback stories.

Python remains the runtime owner. This record does not introduce Rust
production/library code, fixtures, a bridge, or Python caller routing.

## New Unit Boundaries

- `asr_chinese_itn_core`: Chinese inverse text normalization. Writer-ready
  after golden fixtures.
- `asr_qwen_language_schema_contract`: Qwen language validation and schema DTO
  defaults. Writer-ready after golden fixtures.
- `asr_qwen_wav_pcm_decode_core`: WAV PCM fallback decode/channel mean/slicing
  for same-rate audio. Writer-ready after fixtures, but resampling is separate.
- `asr_resample_poly_contract`: prerequisite discovery for
  `scipy.signal.resample_poly` parity. Not writer-ready from this bootstrap.
- `asr_romaji_vocab_ctc_decode_core`: romaji vocab loading, CTC collapse,
  logits-vs-id dispatch, and chunking. Writer-ready after fixtures.
- `asr_romaji_batch_metadata_contract`: fake-session metadata and batch
  padding/mask assembly. Planned; avoid real ONNX Runtime sessions.

Rollback for every child remains keeping the corresponding Python module as
runtime owner because no production route changes.

## Dependency Evidence

The four source refs import standard library modules plus `numpy`,
`scipy.signal`, `pydub`, `soundfile`, and `onnxruntime`. The wider environment
also declares `librosa`, `qwen-asr`, `sentencepiece`, `torch`, and
`transformers`, but the selected deterministic helper seams do not call them.

First-layer source records are indexed under `third_party/sources/` for
`numpy-1.26.4`, `scipy-1.17.1`, `soundfile-0.14.0`, `pydub-0.25.1`,
`librosa-0.11.0`, `qwen-asr-0.0.6`, `sentencepiece-0.2.2`, and
`transformers-4.57.6`. `onnxruntime-1.27.0` and `torch-2.13.0+cpu` have
upstream fallbacks per `third_party/sources/MISSING_SOURCES.md`.
`third_party/source_audit.json` reports no errors.

No second-layer expansion is used. In particular, native OpenBLAS,
libsndfile/codecs, ONNX Runtime execution-provider internals, tokenizers,
safetensors, torch, transformers, sentencepiece, qwen-asr model code, and model
weights are not public-seam evidence for these child units.

## ndarray Policy

`ndarray` is already present in the Rust crate and is allowed for future
AI/model-adjacent work. For this split:

- Defer it for Chinese ITN and Qwen language/schema because those are string
  and DTO contracts.
- Defer it for Qwen WAV PCM decode because a `Vec<f32>` sample buffer is enough
  for fallback decode fixtures.
- Consider it for romaji logits decode and batch metadata if the writer exposes
  array-shaped Rust APIs; still do not claim broad NumPy compatibility.
- Do not use it to mask the unresolved SciPy `resample_poly` parity question.

## Writer Readiness

Writer-ready after fixture generation:

- `asr_chinese_itn_core`
- `asr_qwen_language_schema_contract`
- `asr_qwen_wav_pcm_decode_core` for same-rate WAV PCM fallback only
- `asr_romaji_vocab_ctc_decode_core`

Not writer-ready from this bootstrap:

- `asr_resample_poly_contract`
- `asr_romaji_batch_metadata_contract` if it tries to own file loading or real
  ONNX Runtime sessions instead of fake metadata and synthetic waveforms

## Reversal

If later discovery finds this split too fine-grained, merge only units that
share fixtures and review risk. Do not merge model sessions, arbitrary audio
codec handling, or resampling into pure text/DTO/decode units without a new
record.
