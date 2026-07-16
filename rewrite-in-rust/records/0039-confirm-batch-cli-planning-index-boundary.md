# 0039 - Confirm Batch CLI Planning And Index Boundary

## Context

`scripts/slice_asr_cli.py` mixes several different concerns:

- command-line validation and folder scanning;
- source identity and output-skip bookkeeping;
- real audio loading and writing through `librosa` and `soundfile`;
- model runtime calls through Qwen ASR and RMVPE APIs;
- JSON-driven re-slicing and chunk/lab sidecar writing.

The next manifest unit, `batch_cli_planning_and_index_core`, should not pull in
audio decoding, model runtime, or chunk writing. Those dependencies are native,
model-adjacent, or already assigned to later batch units.

## Decision

Confirm `batch_cli_planning_and_index_core` as one fixture-bound unit covering
only deterministic planning and index behavior:

- `batch_iter` positive-size behavior and batch grouping;
- `repair_text_candidates` and `normalize_slicing_method`, including Chinese
  labels and repaired legacy mojibake candidates;
- `resolve_slice_bounds` validation and normalized `(min, max)` output;
- `collect_audio_files` recursive and top-level scans, supported extension
  filtering, case-insensitive suffix matching, and sorted order;
- `safe_stem`, `file_md5`, and `source_key`;
- `has_existing_outputs`;
- `source_index_path`, `load_source_index`, `save_source_index`,
  `index_has_completed_output`, and `update_source_index`;
- fixture-backed batch loop accounting for skipped existing files, MD5
  failures, processing failures, processed chunks/labs, and saved index
  records, using fake `process_one_file` and injected or temp source files.

Do not include:

- `build_argparser` help text or full CLI parser parity;
- full `validate_args` device/language/model validation beyond the batch-size
  and slicing checks named above;
- `ensure_ffmpeg_on_path`;
- `load_audio`, `save_chunks`, `save_timestamps_json`, or chunk/lab writing;
- `slice_audio_from_json`;
- `run_slicer`, `process_one_file`, ASR calls, RMVPE calls, model loading,
  runtime reuse, or cache clearing;
- `librosa`, `soundfile`, ONNX Runtime, Qwen ASR, RMVPE, or model execution.

## Consequences

The Python checker can use temp directories/files, JSON index files, and fake
processing outcomes without loading ASR/RMVPE runtimes or decoding audio. The
Rust side can be a narrow library model over explicit fixture inputs and temp
file metadata.

`batch_cli_reslice_json_core` remains the owner for JSON timestamp parsing and
chunk/lab sidecar writing. Slicer policy and model runtime behavior stay in
their own later units or legacy Python.

## Reversal

Rollback is keeping `scripts/slice_asr_cli.py` as the runtime owner. No
production bridge is introduced by this record.
