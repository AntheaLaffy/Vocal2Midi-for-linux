# batch_cli_reslice_json_core Bootstrap

## Boundary

`batch_cli_reslice_json_core` covers deterministic JSON timestamp and
JSON-driven re-slicing behavior in `scripts/slice_asr_cli.py`:

```text
extract_text
save_timestamps_json
slice_audio_from_json, with monkeypatched load_audio and sf.write
save_chunks, with monkeypatched sf.write
```

The compatibility surface is:

- `extract_text` returns `""` for `None`, prefers an object's `text` attribute
  when the attribute value is not `None` even if it is an empty string, lets
  property errors propagate, then uses dict `text or transcript or ""`, then
  `str(result)`;
- `save_timestamps_json` computes chunk duration from waveform length and
  sample rate, strips extracted text, rounds offsets/durations to six decimal
  places, sorts records by offset, writes UTF-8 JSON with `ensure_ascii=False`
  and indent `2`, stores source path/MD5 metadata, and still writes an empty
  `chunks` list when results are empty;
- `slice_audio_from_json` requires the JSON file and source audio path to
  exist, propagates JSON decode and record coercion errors, accepts either a
  dict with `chunks` or a list payload, treats missing `chunks`, `chunks: null`,
  and falsy non-list payloads such as top-level `0` as empty, returns `0` for
  empty chunk lists without loading audio, and returns `0` for empty synthetic
  audio without creating the output directory;
- JSON re-slice sample ranges use `int(round(...))` with Python's half-even
  rounding, clip start to `>= 0` and end to `<= waveform length`, skip
  `start_sample >= end_sample`, and keep processing later records;
- WAV names use
  `<safe_stem>_chunk{index:04d}_off{offset:08.2f}s_dur{duration:07.2f}s.wav`;
- lab sidecars use
  `<safe_stem>_chunk{index:04d}_off{offset:08.2f}s.lab` and are written only
  when `str(record.get("text", ""))` is truthy, so missing text and `""` skip
  the lab while `None`, `False`, `0`, and whitespace write sidecars;
- `save_chunks` creates the chunk directory, writes each chunk through
  `sf.write`, returns the written paths, and formats names using the supplied
  source stem without applying `safe_stem`.

The unit does not cover real audio decoding, resampling, FFmpeg lookup,
SoundFile/libsndfile encoding, actual WAV byte content, ASR/RMVPE/slicer/model
execution, full CLI parser UX, or production routing.

## Dependency Expansion

The selected behavior uses Python stdlib and synthetic arrays:

- `json` for payload loading and UTF-8 pretty output;
- `pathlib.Path` for existence checks, output paths, and source metadata;
- Python numeric coercion and `round` behavior for sample ranges;
- synthetic waveform length and shape behavior in the checker.

The source helpers import and call:

- `librosa` through `load_audio`;
- `soundfile` as `sf.write`;
- NumPy-like waveform objects with `.size`, `.shape[-1]`, `len(...)`, and
  slicing;
- broader ASR/RMVPE/slicer/model imports at module import time.

Dependency evidence is available in `pyproject.toml`, `requirements.txt`,
`uv.lock`, `third_party/sources/manifest.json`, and
`third_party/native_sources/manifest.json`: `librosa`, `numpy`, and
`soundfile` sources are vendored, and `libsndfile` native sources are mapped.
Those dependencies are real runtime dependencies of the broader helper, but
not replacement dependencies for this migration unit. The Rust implementation
should model the deterministic contract from fixture data rather than decode or
encode audio.

## Seam

Use the default rewrite seam:

- kind: independent Rust library
- crate: `rewrite-in-rust/rust/crates/v2m-core`
- runtime owner: legacy Python
- bridge dependencies: none

The Python checker should import the legacy module in the uv environment, patch
`load_audio` and `sf.write`, and use temp files/directories plus synthetic
waveforms. The Rust side should consume the same JSONL fixtures and produce the
same path, JSON, error, skip, write-plan, and return-count outputs.

No production bridge is introduced.

## Fixture Harness

Python and Rust tests should consume:

```text
rewrite-in-rust/fixtures/batch_cli_reslice_json_core.jsonl
```

The legacy Python side should be checked by:

```bash
uv run python rewrite-in-rust/bootstrap/check_batch_cli_reslice_json_core.py
```

Fixtures must not call real `librosa.load`, real `soundfile.write` with actual
codec output, `run_slicer`, `process_one_file`, `batch_transcribe_asr`,
`load_qwen_model`, `RmvpeTranscriber`, or `clear_qwen_model_cache`.

## Rollback

Rollback is keeping production ownership unchanged:

```text
scripts/slice_asr_cli.py
```

No caller should import Rust output for this unit until a later promotion record
chooses and verifies a bridge.
