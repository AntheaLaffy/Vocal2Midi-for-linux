# batch_cli_planning_and_index_core Bootstrap

## Boundary

`batch_cli_planning_and_index_core` covers deterministic planning and source
index behavior in `scripts/slice_asr_cli.py`:

```text
batch_iter
repair_text_candidates
normalize_slicing_method
resolve_slice_bounds
collect_audio_files
safe_stem
file_md5
source_key
has_existing_outputs
source_index_path
load_source_index
save_source_index
index_has_completed_output
update_source_index
the main-loop accounting shape with fake process_one_file outcomes
```

The compatibility surface is:

- batch sizes must be greater than zero;
- audio scanning supports `.wav`, `.flac`, `.m4a`, and `.mp3`, case
  insensitively, with recursive or top-level traversal and sorted output;
- slicing methods accept canonical values, `auto`, Chinese labels, repaired
  legacy mojibake candidates, and keyword fallback;
- unsupported slicing methods preserve the legacy supported-values error;
- slice bounds must be supplied as a pair, have `min >= 0`, `max > 0`, and
  `min <= max`;
- source keys use the space-normalized stem plus the first eight MD5
  characters;
- source-index files live at `output_dir/jsons/_source_index.json`, malformed or
  non-object JSON loads as `{}`, and saved JSON uses UTF-8 with
  `ensure_ascii=False` and indent `2`;
- source-index updates preserve the caller-supplied resolved source path and
  persist the whole index object, including pre-existing records;
- completed-output detection checks an indexed JSON/lab/slice output and the
  direct output tree shape, including legacy errors for truthy non-string
  `output_key` values in malformed index records;
- batch-loop accounting increments processed chunk/lab totals,
  skipped-existing, and skipped-failed counters in the same order as legacy
  Python when MD5, skip, and fake processing outcomes are supplied.

The unit does not cover full CLI help rendering, `ensure_ffmpeg_on_path`,
audio decoding, waveform chunk writing, JSON-driven re-slicing, slicer
algorithms, Qwen ASR, RMVPE inference, model loading, runtime reuse, cache
clearing, FFmpeg behavior, full stdout/progress text, flag-specific argparse
validation wording, or model execution.

## Dependency Expansion

The selected behavior uses Python stdlib:

- `pathlib.Path`
- `hashlib.md5`
- `json`
- `os` path separators indirectly through `Path`
- list/string/float parsing

The Rust fixture implementation uses `md-5` for MD5 digest parity,
`encoding_rs` for the legacy GB18030/GBK encode step used by mojibake slicing
label repair, and `serde_json` with `preserve_order` for Python dict
insertion-order parity when source-index JSON is loaded, updated, and
pretty-saved. These are local helper crates for the independent Rust test
surface; they do not introduce a production bridge or any model/runtime
dependency.

The source file imports `librosa`, `soundfile`, `inference.API.asr_api`,
`inference.API.rmvpe_api`, `inference.API.slicer_api`, and
`inference.device_utils` at module import time. Those dependencies are runtime
dependencies of the broader batch CLI, but their capabilities are not part of
this unit's replacement surface. Vendored-source evidence exists for
`librosa` and `soundfile`, while ONNX/Torch/Qwen/RMVPE execution remains
legacy-owned under the Stage 1 inference exclusions.

## Seam

Use the default rewrite seam:

- kind: independent Rust library
- crate: `rewrite-in-rust/rust/crates/v2m-core`
- runtime owner: legacy Python
- bridge dependencies: none

The Python checker should exercise legacy helpers with temp files, temp output
trees, index JSON, and fake processing outcomes. The Rust side should mirror
the same planning/index decisions from explicit JSON fixture inputs. No
production bridge is introduced.

## Fixture Harness

Python and Rust tests should consume:

```text
rewrite-in-rust/fixtures/batch_cli_planning_and_index_core.jsonl
```

The legacy Python side should be checked by:

```bash
uv run python rewrite-in-rust/bootstrap/check_batch_cli_planning_and_index_core.py
```

Fixtures must not call `load_audio`, `run_slicer`, `process_one_file` with real
model/audio behavior, `batch_transcribe_asr`, `load_qwen_model`,
`RmvpeTranscriber`, or `clear_qwen_model_cache`.

## Rollback

Rollback is keeping production ownership unchanged:

```text
scripts/slice_asr_cli.py
```

No caller should import Rust output for this unit until a later promotion record
chooses and verifies a bridge.
