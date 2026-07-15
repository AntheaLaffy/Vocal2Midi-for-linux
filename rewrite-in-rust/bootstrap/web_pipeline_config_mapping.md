# web_pipeline_config_mapping Bootstrap

## Boundary

`web_pipeline_config_mapping` covers only:

```text
web_task_manager.py::TaskManager._build_config
```

The public compatibility surface is the supported Web frontend configuration
mapping into `application.config.PipelineConfig`:

- Web defaults for slicing method, language, device, tempo, lyric options,
  export flags, debug flags, thresholds, timestamps, batch sizes, slice bounds,
  model paths, and pitch formatting.
- `normalize_runtime_device` is applied to the incoming Web device value.
- `save_dir` is converted to a `Path` and created with parents.
- `enable_lyrics_match` controls whether `lyrics` becomes `original_lyrics`.
- `output_formats` always starts with `mid`, then appends `txt`, `csv`,
  `chunks`, and `ustx` in that order when corresponding Web flags are true.
- `output_pitch_curve` is effective only when `export_ustx` is true.
- invalid slice bounds fall back to `DEFAULT_SLICE_MIN_SEC` and
  `DEFAULT_SLICE_MAX_SEC`.
- `ts` is generated as `t0 + i * (1.0 - t0) / nsteps` for
  `i in range(nsteps)`.
- Web `quantize_precision` and `quantize_algorithm` remain visible settings but
  ignored by this mapping; `PipelineConfig` keeps `quantization_step=16` and
  `quantization_mode="bayes"`.

The unit does not cover persisted settings merge, Flask request parsing, task
creation/lifecycle, SocketIO event behavior, GUI behavior, model downloads, or
model inference.

## Dependency Expansion

`web_task_manager.py` imports broad task-management dependencies, but the
selected mapping function only needs:

- stdlib: `pathlib`
- local: `application.config.PipelineConfig`
- local: `application.config.validate_slice_bounds`
- local: `application.config.DEFAULT_SLICE_MIN_SEC`
- local: `application.config.DEFAULT_SLICE_MAX_SEC`
- local: `inference.device_utils.normalize_runtime_device`

The mapping function does not call Flask, Flask-SocketIO, threading, WebSocket
redirectors, model runtimes, ONNX Runtime, Qwen ASR, PyQt, or network services.

Dependency evidence:

- `docs/web-api.md` states that `/api/pipeline/start` merges persisted settings
  first, then applies per-run config. That merge happens in `web_server.py`
  before this unit's selected boundary.
- `tests/test_quantization_caller_defaults_contract.py` already pins the
  visible-but-ignored Web quantization settings behavior.
- `pyproject.toml`, `requirements-web.txt`, and `uv.lock` include Flask and
  SocketIO for the Web backend, but this unit does not need a Rust Web server or
  WebSocket dependency.

## Seam

Use the default rewrite seam:

- kind: independent Rust library
- crate: `rewrite-in-rust/rust/crates/v2m-core`
- runtime owner: legacy Python
- bridge dependencies: none

Rust tests use `serde_json` as a dev-dependency to read the durable JSONL
fixtures. The production Rust mapping itself stays a plain library function.

## Fixture Harness

Python and Rust tests consume:

```text
rewrite-in-rust/fixtures/web_pipeline_config_mapping.jsonl
```

The fixtures cover:

- minimal Web config with Web-specific defaults
- explicit full mapping of model paths, language, device alias, lyrics, export
  flags, debug flags, thresholds, timestamps, batches, slice bounds, and model
  helper paths
- invalid slice-bound fallback
- USTX-gated pitch curve output
- zero `nsteps` producing an empty timestamp list
- `nsteps=1` timestamp generation
- quantization Web settings remaining ignored
- output directory creation errors when `save_dir` points at an existing file

The legacy Python side is checked by:

```bash
uv run python rewrite-in-rust/bootstrap/check_web_pipeline_config_mapping.py
```

## Repeated-Call Behavior

For the same frontend config and audio path, repeated calls produce the same
`PipelineConfig` fields and ensure the same output directory exists. The mapping
does not depend on task IDs, SocketIO rooms, model availability, ONNX providers,
or network access.

## Rollback

Rollback is keeping production ownership unchanged:

```text
web_task_manager.TaskManager._build_config
```

No Web caller should import Rust output for this unit until a later promotion
record chooses and verifies a bridge.
