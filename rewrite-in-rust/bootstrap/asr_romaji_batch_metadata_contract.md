# ASR Romaji Batch Metadata Contract Bootstrap

Date: 2026-07-18

## Unit

`asr_romaji_batch_metadata_contract`

## Public Boundary

This unit owns the deterministic metadata and batch-preparation helpers in
`inference/romaji_asr/common.py`:

- `get_fixed_batch_size`
- `get_fixed_num_samples`
- `ort_type_to_numpy_dtype`
- `prepare_batch`

It does not own `load_audio`, `create_session`, ONNX Runtime providers, audio
file IO, resampling, vocab/CTC decoding, or model execution.

## Fixture Strategy

Use fake session metadata objects with `.get_inputs()` returning `.name`,
`.type`, and `.shape`. Use monkeypatched `load_audio` to return synthetic
`np.float32` waveforms and record `sample_rate` passthrough.

`fixtures/asr_romaji_batch_metadata_contract.jsonl` contains 26 Python golden
cases covering:

- fixed, dynamic, bool, empty, and one-dimensional input shapes
- negative fixed sample dimension errors after audio loading
- case-sensitive dtype substring mapping
- empty `audio_paths`
- fixed batch-size mismatch before audio loading
- missing synthetic waveform keys after the load call is recorded
- missing `input_values` after audio loading
- dynamic target length and fixed target length
- the legacy fixed-zero-samples fallback to `max(lengths)` through `or`
- zero-length waveform handling
- truncation, padding, attention mask construction including float16, dtype
  casts, and used lengths
- optional absence of `attention_mask`

## Rust Boundary

Use an independent `v2m-core` module. `ndarray` owns 2D matrix storage for
`input_values` and `attention_mask`, while Python-specific metadata and dtype
mapping policy are hand-written against fixtures.

No PyO3 bridge, subprocess bridge, ONNX Runtime dependency, audio file IO, or
production Python routing is introduced.

## Writer Readiness

The unit is writer-ready after this bootstrap record. Runtime ownership remains
legacy until a separate promotion unit changes it.
