# 0017 - Confirm Application Job Contract Boundary

## Context

Stage 1 begins with `application_job_contract`, a provisional unit pointing at
`application/pipeline.py`, `application/config.py`, and
`application/exceptions.py`.

Source inspection showed that the useful compatibility surface is not the full
hybrid inference pipeline. It is the application-layer guard around that legacy
pipeline: model path existence checks, `output_lyrics` gating, cancellation
before start, and exception mapping.

The imported hybrid pipeline pulls in broad inference dependencies, but those
dependencies are call targets, not behavior this unit needs to own.

## Decision

Confirm `application_job_contract` as a narrow application-boundary unit.

Implement only:

- required GAME path validation;
- conditional HubertFA and ASR path validation when `output_lyrics` is true;
- Python-compatible `ModelNotFoundError` message/detail construction;
- cancellation-before-start ordering;
- `InterruptedError` to cancellation mapping;
- `Vocal2MidiError` passthrough;
- generic exception wrapping.

Keep `auto_lyric_hybrid_pipeline`, model loading, model execution, GUI/Web
config construction, Flask/SocketIO routing, and the full `PipelineConfig`
mapping legacy-owned or reserved for later Stage 1 units.

Use an independent Rust library seam with a closure that models the legacy
pipeline result. Do not add PyO3, subprocess, HTTP, Flask, PyQt, ONNX Runtime,
Qwen ASR, or model-runtime dependencies for this unit.

## Consequences

- Stage 1 can start with a small fixture-bound application contract instead of
  immediately entering model inference or Web runtime code.
- `application_job_contract` can move to `reimplemented` after Rust fixtures and
  the Python checker pass, but it still needs independent review before it can
  be marked `verified`.
- Later Web/GUI config mapping units must not silently absorb this job guard;
  they should call out their own public mapping behavior and rollback route.

## Reversal

If later promotion needs a runtime bridge for application validation, add a
separate promotion record and keep these fixtures as the guard/error baseline.
Until then, rollback is keeping `application.pipeline.run_auto_lyric_job` as the
production owner.
