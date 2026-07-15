# application_job_contract Bootstrap

## Boundary

`application_job_contract` covers the application-layer guard around the hybrid
pipeline:

- `application/pipeline.py::_validate_model_paths`
- `application/pipeline.py::run_auto_lyric_job`

The public compatibility surface is:

- GAME model path is always required.
- HubertFA and ASR model paths are required only when `output_lyrics` is true.
- Missing or empty paths raise `ModelNotFoundError` with message
  `模型路径验证失败` and details joined with `; ` in the Python check order.
- Path validation uses `os.path.exists`; it does not require a directory, model
  file contents, or model-format validation.
- Cancellation is checked after path validation and before the hybrid pipeline
  starts.
- A pre-start cancellation raises `CancellationError` with
  `Pipeline was cancelled before starting.`
- `InterruptedError` from the pipeline maps to `CancellationError` with
  `Pipeline was interrupted by user.`
- `Vocal2MidiError` from the pipeline passes through unchanged.
- Other exceptions are wrapped in `Vocal2MidiError` with message
  `Pipeline execution failed: {error}` and `details=str(error)`.

The unit does not replace `auto_lyric_hybrid_pipeline`, model loading, model
execution, GUI/Web config construction, Flask/SocketIO routing, or any frontend
surface.

## Dependency Expansion

`application/pipeline.py` imports:

- stdlib: `os`
- local: `application.config.PipelineConfig`
- local: `application.exceptions`
- local: `inference.pipeline.auto_lyric_hybrid.auto_lyric_hybrid_pipeline`

The selected compatibility behavior uses `os.path.exists`, `PipelineConfig`
fields, local exception classes, and the call/error boundary around
`auto_lyric_hybrid_pipeline`. It does not need to inspect, load, or execute any
model runtime.

Dependency evidence:

- `pyproject.toml`, `requirements*.txt`, and `uv.lock` include the broad runtime
  stack: Flask, PyQt, librosa, mido, NumPy, ONNX Runtime, qwen-asr, soundfile,
  torch, and other inference dependencies.
- `third_party/sources/manifest.json`,
  `third_party/sources/MISSING_SOURCES.md`, and
  `third_party/source_audit.json` show source coverage for the installed
  Python environment, including upstream fallbacks for no-sdist packages such
  as ONNX Runtime and torch.
- Those packages matter for the legacy hybrid pipeline, but this unit only
  preserves the guard and exception contract before and around that legacy call.

Therefore the Rust implementation should not add dependencies for Flask, PyQt,
ONNX Runtime, Qwen ASR, librosa, soundfile, mido, torch, PyO3, subprocess
bridges, HTTP, or a runtime router.

## Seam

Use the default rewrite seam:

- kind: independent Rust library
- crate: `rewrite-in-rust/rust/crates/v2m-core`
- runtime owner: legacy Python
- bridge dependencies: none

The Rust side models the application job boundary with a closure for the legacy
pipeline call. This proves validation order and error mapping without routing
production Python through Rust.

## Fixture Harness

Rust tests and the legacy Python checker consume:

```text
rewrite-in-rust/fixtures/application_job_contract.tsv
```

The fixtures cover:

- valid lyric and no-lyric model path cases
- file paths accepted by Python's `os.path.exists`
- missing and empty GAME, HubertFA, and ASR path errors
- order-sensitive combined missing-path errors
- path validation taking precedence over pre-start cancellation
- no-lyrics mode ignoring missing HubertFA and ASR paths
- pre-start cancellation not invoking the pipeline
- a cancel checker that exists but returns false still dispatching the pipeline
- successful dispatch passing exactly `cfg.to_kwargs()` to the legacy pipeline
- `InterruptedError` mapping
- `Vocal2MidiError` passthrough
- generic exception wrapping

The legacy Python side is checked by:

```bash
uv run python rewrite-in-rust/bootstrap/check_application_job_contract.py
```

## Repeated-Call Behavior

For the same filesystem state, cancellation result, and pipeline outcome,
repeated calls must produce the same application-layer result. The contract must
not depend on GUI state, WebSocket state, model runtime caches, ONNX provider
availability, or network access.

## Rollback

Rollback is keeping the production owner unchanged:

```text
application.pipeline.run_auto_lyric_job
```

No GUI, Web, CLI, or inference caller should import Rust output for this unit
until a later promotion record chooses and verifies a bridge.
