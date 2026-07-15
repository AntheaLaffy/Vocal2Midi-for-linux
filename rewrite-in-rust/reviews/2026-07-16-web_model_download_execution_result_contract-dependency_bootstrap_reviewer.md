# web_model_download_execution_result_contract - dependency_bootstrap_reviewer

Date: 2026-07-16
Decision: pass

## Findings

No findings.

## Boundary Decision

The manifest unit boundary is confirmed. It should not be split, merged,
deferred, or replaced.

Evidence:

- `rewrite-in-rust/manifest.yaml:631` keeps the unit scoped to
  `_execute_download` task-visible execution result handling and explicitly
  excludes real subprocess execution.
- `rewrite-in-rust/records/0033-confirm-web-model-download-execution-result-boundary.md:25`
  confirms a fake process/socket orchestration boundary.
- `rewrite-in-rust/records/0033-confirm-web-model-download-execution-result-boundary.md:40`
  excludes real `download_models.py` execution, network access, package
  installation, archive extraction, marker safety, and OS process-tree
  termination semantics.
- `rewrite-in-rust/dependencies/web_model_download_execution_result_contract.yaml:24`
  records a legacy-owned library seam with dependencies on the request/catalog
  and process-plan units.
- `rewrite-in-rust/dependencies/web_model_download_execution_result_contract.yaml:48`
  keeps command/env planning and output-line parsing in
  `web_model_download_process_plan_contract`.
- `rewrite-in-rust/dependencies/web_model_download_execution_result_contract.yaml:52`
  reserves OS process-tree termination for
  `web_model_download_process_termination_contract`.
- `rewrite-in-rust/bootstrap/web_model_download_execution_result_contract.md:31`
  excludes route mapping, task lifecycle, process planning details, real
  SocketIO delivery failures, process-tree termination, and asset download
  behavior.
- `rewrite-in-rust/bootstrap/check_web_model_download_execution_result_contract.py:97`
  replaces `subprocess.Popen` with `FakePopenFactory`; `:135` records
  termination callbacks instead of killing a process.
- `rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_execution.rs:3`
  documents fake process/socket inputs; `:130` reuses process-plan helpers for
  command/env, and `:147` reuses process output parsing instead of reopening
  parser semantics.
- `web_model_download_manager.py:252` shows the legacy `_execute_download`
  orchestration being modeled, while `:333` keeps real process-tree termination
  as a separate legacy capability.
- `docs/web-api.md:310` describes stop-route process termination separately
  from execution-result status serialization.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_web_model_download_execution_result_contract.py`: passed
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml web_model_download_execution`: passed
- `cargo tree --manifest-path rewrite-in-rust/rust/Cargo.toml -p v2m-core`: passed; `v2m-core` depends only on `serde_json` and its transitive dependencies.
- `rg -n "std::process|Command::|Popen|subprocess|kill|terminate|taskkill|killpg|socketio|download_models\\.py" rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_execution.rs rewrite-in-rust/bootstrap/check_web_model_download_execution_result_contract.py rewrite-in-rust/fixtures/web_model_download_execution_result_contract.jsonl`: reviewed; Rust has no real process APIs, and Python `Popen` is monkey-patched to a fake factory.
- `git diff --check -- rewrite-in-rust/manifest.yaml rewrite-in-rust/records/0033-confirm-web-model-download-execution-result-boundary.md rewrite-in-rust/dependencies/web_model_download_execution_result_contract.yaml rewrite-in-rust/bootstrap/web_model_download_execution_result_contract.md rewrite-in-rust/fixtures/web_model_download_execution_result_contract.jsonl rewrite-in-rust/bootstrap/check_web_model_download_execution_result_contract.py rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_execution.rs rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_process.rs rewrite-in-rust/rust/crates/v2m-core/src/web_model_download.rs web_model_download_manager.py`: passed
- `uv run pytest tests/test_web_api.py -k ModelDownload -q`: passed, 19 passed and 34 deselected.
- `cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check`: passed

## Residual Risk

This review does not prove behavior parity beyond dependency/bootstrap scope.
Stage behavior, error/tracing, and product ergonomics remain separate required
reviews. Real OS process termination is intentionally unproved here and remains
owned by `web_model_download_process_termination_contract`.

## Promotion Note

This dependency/bootstrap role does not block coordinator state update for the
unit. The unit boundary is confirmed as a narrow fake-process execution-result
seam with legacy Python still owning production runtime behavior.
