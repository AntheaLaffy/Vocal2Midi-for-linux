# Rust Rewrite Notes

## User Constraints

- The rewrite target is the Python library layer, not a one-shot rewrite of the
  whole application.
- If Python third-party dependencies or their FFI-linked native sources are
  available locally, they may be used as references.
- Dependency mismatches should be handled by capability-level Rust
  implementations when practical.
- This project has harder cross-language dependency alignment than mvsep-rs.
  Python dependency expansion may reveal native/FFI or algorithm boundaries that
  require changing the rewrite plan.
- The current module/unit list is temporary. Planned units are allowed to be
  split, merged, renamed, deferred, or replaced after discovery.
- Hand-written Rust replacements are acceptable when dependency parity is
  unrealistic, as long as they are narrow and fixture-bound.
- For audio and inference-adjacent units, Rust dependency discovery should
  follow existing high-level crates to their concrete lower-layer codec/model
  crates before deciding to hand-write behavior. For example, `rodio` can be a
  useful trailhead even when the selected seam ultimately needs a smaller
  Symphonia codec component or a compatibility adapter.
- Reference source for hand-written replacements is under `third_party/`:
  Python sdists in `third_party/sources/`, upstream fallbacks in
  `third_party/upstream_sources/`, native/FFI sources in
  `third_party/native_sources/`, and Rust-backed package crates in
  `third_party/cargo_vendor/`.
- The rewrite must be stepwise and stateful.
- Each small unit should focus on one independently verifiable thing.
- Review frequency should be much lower than one review per unit: one R0
  behavior review per stage is the default.
- Resources, references, records, and style constraints must be indexed like a
  teach workspace.
- The verified Stage 1 boundary does not include model inference chains or
  frontend surfaces. Any later expansion into those areas requires a new
  manifest stage and explicit dependency/bootstrap evidence.

## Current Repository Facts

- Intended dependency direction is `gui -> application -> inference` and
  `web -> application -> inference`.
- Existing fast test command is `uv run pytest tests/test_web_api.py`.
- Vendored dependency sources are already represented under `third_party/`.
- Most Rust modules are not connected to the production Python runtime. The
  quantization JSON bridge is the only production-adjacent seam and remains
  opt-in.
- `rewrite-in-rust/manifest.yaml` is a control plane, not a frozen project plan.
- The manifest contains 66 verified units across application/Web contracts,
  batch/model-asset planning, deterministic exports, slicer policies,
  lyric/G2P/HubertFA helpers, quantization, and ASR preprocessing.
- All units remain legacy-owned. Verification demonstrates fixture parity; it
  does not authorize production routing or promotion.
- Stage 1 must keep ONNX Runtime sessions, Qwen encoder/decoder, llama.cpp
  subprocess inference, romaji/GAME/HFA/RMVPE model execution, GUI code, and
  browser static assets legacy-owned.
- The Web model download manager unit was split into
  request/catalog, process-planning, task lifecycle, execution-result, and
  process-termination units. Each has a verified fixture boundary, while live
  subprocess execution, SocketIO delivery, and OS termination remain
  legacy-owned.

## Defaults Chosen

- Directory name: `rewrite-in-rust/`.
- Scope boundary: library-first.
- Bridge default: no PyO3, router, or subprocess bridge at initialization time.
- Owner default: legacy Python remains runtime owner for all planned units.
- Review default: stage-level R0 behavior review after 3-5 reimplemented units.
- Inventory default: dependency discovery may rewrite planned units before
  implementation starts.
- Stage 1 result: 66 verified units, 65 confirmed inventory boundaries, one
  split umbrella, and no promoted runtime owner.
- Stage 1 verification default: use synthetic fixtures, fake subprocesses,
  temp files, fake SocketIO objects, and mocked network/runtime boundaries so
  checks do not download models or run inference.
