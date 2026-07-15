# 0016 - Plan Non-Inference Backend Stage

## Context

The verified rewrite inventory now covers the first foundation batch:
application slice bounds, runtime device normalization, GAME helper logic,
TXT/CSV export rendering, quantization algorithms, the quantization JSON bridge,
caller/default contracts, and quantization routing.

The next requested scope is the rest of the backend, but without touching model
inference chains or frontend surfaces. In this repository that means the
remaining work is not one module. It spans application validation, Web backend
contracts, task/download management, batch and asset tooling, deterministic
exports, slicer helpers, lyric/text alignment helpers, HubertFA helper logic,
and ASR text/schema post-processing.

## Decision

Create `stage1_non_inference_backend` in `rewrite-in-rust/manifest.yaml` and
add the remaining non-inference backend candidates as planned, provisional
migration units.

This stage may cover fixture-bound behavior from:

- `application/`;
- `web_server.py`, `web_task_manager.py`, `web_model_download_manager.py`, and
  `web_stream_redirector.py`;
- `download_models.py` and `scripts/slice_asr_cli.py`;
- deterministic export and slicer helpers under `inference/API/`,
  `inference/io/`, and `inference/slicer/`;
- lyric, G2P, alignment, export, metric, and text post-processing helpers under
  `inference/LyricFA/`, `inference/HubertFA/`, `inference/qwen3asr_dml/`, and
  `inference/romaji_asr/` when those helpers can be verified without creating
  model sessions.

This stage must not rewrite:

- desktop GUI code or browser frontend assets;
- Flask/SocketIO routing into a Rust Web server;
- ONNX Runtime session creation or provider execution;
- Qwen encoder/decoder, llama.cpp subprocess inference, or model-worker
  orchestration;
- romaji, GAME, HubertFA, or RMVPE model execution;
- GGUF weight/model-format execution paths.

## Consequences

- Every new Stage 1 unit starts as `planned` and `inventory_status:
  provisional`.
- The next action for any unit is dependency/bootstrap discovery. Writer work
  should not start until the unit has a dependency record, fixture strategy,
  public behavior boundary, and rollback route.
- Verification must use synthetic fixtures, temp files, fake SocketIO objects,
  fake subprocesses, and mocked network/runtime boundaries where needed. It
  must not download model assets or run model inference.
- The existing quantization work remains closed except for regression checks.

## Reversal

If dependency expansion shows that a planned unit is too broad, model-adjacent,
or not independently verifiable, split, defer, or remove that unit before
writer work. Legacy Python remains the runtime owner for all Stage 1 units
until a later promotion record changes ownership with rollback evidence.
