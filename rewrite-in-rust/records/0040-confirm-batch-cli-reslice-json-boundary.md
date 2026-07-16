# 0040 - Confirm Batch CLI Reslice JSON Boundary

## Context

After `batch_cli_planning_and_index_core`, the next manifest unit is
`batch_cli_reslice_json_core`. The relevant helpers in
`scripts/slice_asr_cli.py` combine deterministic JSON/text/path/sample-index
logic with real audio and native codec effects:

- `extract_text` normalizes ASR-like result objects and dictionaries;
- `save_timestamps_json` formats timestamp JSON from chunk metadata and ASR
  results;
- `slice_audio_from_json` validates paths, parses JSON, loads audio, computes
  sample ranges, writes WAV chunks, and writes optional `.lab` files;
- `save_chunks` formats chunk filenames and writes each waveform via
  `soundfile`.

`load_audio` delegates to `librosa` and FFmpeg-related fallback behavior, while
`sf.write` reaches SoundFile/libsndfile native encoding. Those dependencies are
available as vendored source evidence, but replacing them would make this unit
larger than the deterministic compatibility surface.

## Decision

Confirm `batch_cli_reslice_json_core` as a fixture-bound unit covering
deterministic JSON re-slicing and synthetic write planning:

- `extract_text` fallback order, string conversion, `text=None` fallback, and
  property-error propagation;
- timestamp JSON source metadata, chunk index mapping, offset sorting,
  duration calculation from waveform length/sample rate, six-decimal rounding,
  stripped text, UTF-8 pretty JSON, and returned JSON path;
- JSON payload parsing for dict/list payloads, missing JSON/source path errors,
  malformed JSON propagation, empty or missing chunk behavior, empty waveform
  behavior, missing key and numeric coercion errors;
- sample range planning with Python `round`, clipping, invalid-range skipping,
  and written count;
- WAV filename formatting, lab sidecar filename/content decisions including
  truthy string conversion, and `save_chunks` path/write-plan output using
  monkeypatched `sf.write`.

Keep legacy-owned:

- real audio decoding, resampling, and FFmpeg path/runtime behavior;
- real WAV/PCM encoding and SoundFile/libsndfile/native codec behavior;
- ASR, RMVPE, slicer, ONNX Runtime, Qwen, and model runtime behavior;
- full CLI parser/help and complete progress text parity.

## Consequences

The Python checker can use temp files, monkeypatched `load_audio`, monkeypatched
`sf.write`, and synthetic waveform descriptors. The Rust implementation can be
a narrow library model over fixture data: JSON values, waveform lengths/sample
rates, source paths, output roots, and expected write plans.

This avoids pulling `librosa`, NumPy, SoundFile/libsndfile, or FFmpeg into the
Rust seam while still preserving the public behavior that callers observe from
the helper functions under synthetic fixtures.

## Reversal

Rollback is keeping `scripts/slice_asr_cli.py` as the runtime owner. No
production bridge is introduced by this record.
