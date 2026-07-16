# web_stream_redirector_contract - dependency_bootstrap_reviewer

Date: 2026-07-16
Decision: pass

## Findings

No findings.

## Boundary Decision

Boundary remains confirmed.

The split is justified by record 0019, which separates the original broad Web task lifecycle surface into task registry, stream redirector, and pipeline execution event units (`rewrite-in-rust/records/0019-split-web-task-lifecycle-unit.md:15`, `rewrite-in-rust/records/0019-split-web-task-lifecycle-unit.md:24`). Record 0021 confirms this unit as only callback write filtering, callback failure swallowing, original stream writes, flush delegation, and delegated attribute access (`rewrite-in-rust/records/0021-confirm-web-stream-redirector-boundary.md:14`).

Capability coverage is sufficient for dependency bootstrap. The legacy source is a standalone callback wrapper: `write` strips only for callback payloads, swallows callback exceptions, and always writes the original text to the stream; `flush` and `__getattr__` delegate to the underlying stream (`web_stream_redirector.py:32`, `web_stream_redirector.py:41`, `web_stream_redirector.py:45`). The dependency record maps those capabilities to a narrow Rust behavior model and fixture table (`rewrite-in-rust/dependencies/web_stream_redirector_contract.yaml:4`, `rewrite-in-rust/dependencies/web_stream_redirector_contract.yaml:8`, `rewrite-in-rust/dependencies/web_stream_redirector_contract.yaml:16`). The Rust unit models only those operations and consumes the shared JSONL fixture in tests (`rewrite-in-rust/rust/crates/v2m-core/src/web_stream.rs:24`, `rewrite-in-rust/rust/crates/v2m-core/src/web_stream.rs:46`, `rewrite-in-rust/rust/crates/v2m-core/src/web_stream.rs:57`, `rewrite-in-rust/rust/crates/v2m-core/src/web_stream.rs:97`).

Kept-legacy decisions are appropriate. SocketIO payload delivery, task log construction, stdout/stderr installation/restoration, task lifecycle, and model inference are explicitly excluded by the bootstrap record (`rewrite-in-rust/bootstrap/web_stream_redirector_contract.md:20`). Real SocketIO emission, `_execute_pipeline` stream installation/restoration, and real Python stream objects are also listed as legacy-kept or separately owned capabilities in the dependency record (`rewrite-in-rust/dependencies/web_stream_redirector_contract.yaml:33`). The caller context confirms those broader concerns live in `TaskManager._execute_pipeline`, not in `WebStreamRedirector` itself (`web_task_manager.py:209`, `web_task_manager.py:252`, `web_task_manager.py:298`).

The seam choice is sound. The bootstrap record keeps runtime ownership in legacy Python, declares no bridge dependencies, and forbids importing Rust output into Web callers until a later promotion record chooses a bridge (`rewrite-in-rust/bootstrap/web_stream_redirector_contract.md:32`, `rewrite-in-rust/bootstrap/web_stream_redirector_contract.md:72`). The Rust module did not introduce Flask, SocketIO, Python threading, PyO3, async runtime, network, subprocess, ONNX, Torch, Qwen, or PyQt dependencies; dependency scanning found only the documented exclusions in control-plane text and `serde_json` for fixture tests.

## Checks

- `env PYTHONDONTWRITEBYTECODE=1 UV_CACHE_DIR=/tmp/v2m-uv-cache uv run python rewrite-in-rust/bootstrap/check_web_stream_redirector_contract.py`: passed.
- `env CARGO_TARGET_DIR=/tmp/v2m-web-stream-review-target cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml web_stream`: passed, 1 `web_stream` test.
- `rg -n "flask|Flask|socketio|SocketIO|thread|Thread|subprocess|onnx|torch|qwen|model|PyQt|pyo3|tokio|reqwest|axum" rewrite-in-rust/rust/crates/v2m-core/src/web_stream.rs rewrite-in-rust/rust/crates/v2m-core/Cargo.toml rewrite-in-rust/rust/Cargo.toml rewrite-in-rust/bootstrap/web_stream_redirector_contract.md rewrite-in-rust/dependencies/web_stream_redirector_contract.yaml web_stream_redirector.py`: passed as dependency scan; hits were source docstrings/control-plane exclusions, the dependency record's explicit SocketIO legacy-kept note, and Rust comments naming a behavior model.
- `rg -n "name = \"(flask|flask-socketio|python-socketio|onnxruntime|torch|PyQt5|pyo3|tokio|reqwest|axum|socketio|thread)\"|serde_json" rewrite-in-rust/rust/Cargo.lock rewrite-in-rust/rust/Cargo.toml rewrite-in-rust/rust/crates/v2m-core/Cargo.toml`: passed; no heavy Web/model/runtime crates found, with `serde_json` present for fixture tests.
- `git diff --check`: passed.

## Residual Risk

This review does not prove full WebSocket delivery, stdout/stderr restoration around pipeline execution, task lifecycle atomicity, or model pipeline behavior. Those concerns are intentionally assigned to adjacent units or remain legacy-owned. I did not run the full `tests/test_web_api.py` suite in this dependency/bootstrap pass to avoid creating application settings/cache files under the user's write constraint; the unit-level Python and Rust parity checks passed.

## Promotion Note

This role does not block coordinator state update. Runtime ownership must remain `legacy`; do not promote this unit or mark the manifest verified from this report alone.
