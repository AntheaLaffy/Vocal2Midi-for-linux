# application_job_contract - behavior_reviewer

Date: 2026-07-15
Decision: pass

## Findings

No findings after checking the full behavior role scope.

Behavior evidence checked:

- Public boundary: the unit is limited to the application-layer guard around `application/pipeline.py::_validate_model_paths` and `application/pipeline.py::run_auto_lyric_job`, not model execution or caller construction (`rewrite-in-rust/records/0017-confirm-application-job-contract-boundary.md:19`, `rewrite-in-rust/records/0017-confirm-application-job-contract-boundary.md:31`, `rewrite-in-rust/dependencies/application_job_contract.yaml:47`).
- Model path inputs and output/error shape: Python always checks GAME first, then checks HubertFA and ASR only when `output_lyrics` is true, and raises `ModelNotFoundError("模型路径验证失败", details="; ".join(errors))` (`application/pipeline.py:14`, `application/pipeline.py:15`, `application/pipeline.py:23`, `application/pipeline.py:26`). Rust collects the same labels in the same order and joins them with `; ` (`rewrite-in-rust/rust/crates/v2m-core/src/application.rs:77`, `rewrite-in-rust/rust/crates/v2m-core/src/application.rs:80`, `rewrite-in-rust/rust/crates/v2m-core/src/application.rs:82`, `rewrite-in-rust/rust/crates/v2m-core/src/application.rs:89`).
- Ordering: Python validates paths before checking cancellation and before dispatching the hybrid pipeline (`application/pipeline.py:46`, `application/pipeline.py:49`, `application/pipeline.py:53`). Rust preserves that order by calling `validate_model_paths` before `cancel_before_start` and before the legacy-pipeline closure (`rewrite-in-rust/rust/crates/v2m-core/src/application.rs:109`, `rewrite-in-rust/rust/crates/v2m-core/src/application.rs:111`, `rewrite-in-rust/rust/crates/v2m-core/src/application.rs:117`).
- Pipeline error mapping: Python maps `InterruptedError` to `CancellationError`, passes `Vocal2MidiError` through, and wraps other exceptions with `Pipeline execution failed: {error}` plus `details=str(error)` (`application/pipeline.py:52`, `application/pipeline.py:54`, `application/pipeline.py:56`, `application/pipeline.py:58`). Rust exposes the same fixture-visible error kinds, messages, and details for interrupted, Vocal2Midi, and other errors (`rewrite-in-rust/rust/crates/v2m-core/src/application.rs:117`, `rewrite-in-rust/rust/crates/v2m-core/src/application.rs:119`, `rewrite-in-rust/rust/crates/v2m-core/src/application.rs:122`, `rewrite-in-rust/rust/crates/v2m-core/src/application.rs:128`).
- Fixture coverage: the shared TSV covers valid lyric/no-lyric paths, file paths, cancel-false dispatch, single missing/empty paths, combined missing-path ordering, invalid-path-before-cancel, no-lyrics ignored HFA/ASR, pre-start cancellation, interrupted pipeline, `Vocal2MidiError` passthrough, and generic wrapping (`rewrite-in-rust/fixtures/application_job_contract.tsv:2`, `rewrite-in-rust/fixtures/application_job_contract.tsv:6`, `rewrite-in-rust/fixtures/application_job_contract.tsv:10`, `rewrite-in-rust/fixtures/application_job_contract.tsv:12`, `rewrite-in-rust/fixtures/application_job_contract.tsv:13`, `rewrite-in-rust/fixtures/application_job_contract.tsv:15`, `rewrite-in-rust/fixtures/application_job_contract.tsv:16`, `rewrite-in-rust/fixtures/application_job_contract.tsv:18`). This addresses the earlier dependency-bootstrap low follow-up for combined-missing-path and invalid-path-before-cancel rows (`rewrite-in-rust/reviews/2026-07-15-application_job_contract-dependency_bootstrap_reviewer.md:8`).
- Python checker coverage: the checker uses the shared fixture table, restores the legacy pipeline after each case, verifies kind/message/details, verifies whether the pipeline was called, and verifies successful dispatch passes exactly `cfg.to_kwargs()` to the legacy pipeline (`rewrite-in-rust/bootstrap/check_application_job_contract.py:166`, `rewrite-in-rust/bootstrap/check_application_job_contract.py:176`, `rewrite-in-rust/bootstrap/check_application_job_contract.py:187`, `rewrite-in-rust/bootstrap/check_application_job_contract.py:199`, `rewrite-in-rust/bootstrap/check_application_job_contract.py:204`).
- Rust fixture coverage: the Rust test consumes the same TSV table, checks error kind/message/details, and checks the legacy-pipeline closure call flag; targeted Rust tests separately cover no-lyrics path requirements and pre-start cancellation not calling the closure (`rewrite-in-rust/rust/crates/v2m-core/src/application.rs:141`, `rewrite-in-rust/rust/crates/v2m-core/src/application.rs:228`, `rewrite-in-rust/rust/crates/v2m-core/src/application.rs:279`, `rewrite-in-rust/rust/crates/v2m-core/src/application.rs:307`, `rewrite-in-rust/rust/crates/v2m-core/src/application.rs:317`, `rewrite-in-rust/rust/crates/v2m-core/src/application.rs:338`).
- Compatibility boundary: the Rust implementation is an independent `v2m-core` library seam with no production bridge and no model/runtime dependencies; `v2m-core` only adds `serde_json` as a dev-dependency for tests (`rewrite-in-rust/bootstrap/application_job_contract.md:64`, `rewrite-in-rust/bootstrap/application_job_contract.md:73`, `rewrite-in-rust/rust/crates/v2m-core/src/lib.rs:3`, `rewrite-in-rust/rust/crates/v2m-core/Cargo.toml:12`).
- Rollback: production ownership remains legacy Python, with rollback documented as keeping `application.pipeline.run_auto_lyric_job` and application exceptions as runtime owners (`rewrite-in-rust/manifest.yaml:409`, `rewrite-in-rust/bootstrap/application_job_contract.md:113`, `rewrite-in-rust/bootstrap/application_job_contract.md:121`, `rewrite-in-rust/records/0017-confirm-application-job-contract-boundary.md:49`).

Reviewer separation was preserved. I reviewed only and wrote this report; I did not edit production code, manifest, records, fixtures, bootstrap scripts, or Rust/Python source.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_application_job_contract.py`: passed with no output.
- `env CARGO_TARGET_DIR=/tmp/v2m-application-job-contract-behavior-target cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml application`: passed; 3 application tests passed, 39 filtered out in `v2m-core`, and 0 matching tests in `v2m-quant-bridge`.
- `rg -n "pyo3|onnx|torch|qwen|flask|socketio|pyqt|reqwest|tokio|subprocess|http|auto_lyric_hybrid|web_server|gui" rewrite-in-rust/rust/crates/v2m-core/src/application.rs rewrite-in-rust/rust/crates/v2m-core/Cargo.toml rewrite-in-rust/rust/crates/v2m-core/src/lib.rs`: no matches; no bridge, frontend, Web, or model-runtime dependency/reference was found in the Rust application unit.
- `git diff --name-only && git ls-files --others --exclude-standard`: inspected current tracked and untracked changes; the behavior-reviewed production Python files `application/pipeline.py`, `application/config.py`, and `application/exceptions.py` were not modified.
- `git diff --check`: passed.

## Residual Risk

The Rust seam models the guard and error contract with minimal path/cancellation inputs plus a closure result. It intentionally does not model full `PipelineConfig.to_kwargs()` transfer in Rust; that remains legacy-owned here and is separately protected on the Python side by the checker. Any future runtime bridge must add its own promotion record and parity evidence for payload transfer and caller integration.

This review did not run live GUI, Web, or model-inference workflows. Those surfaces are outside this unit and remain legacy-owned.

## Promotion Note

This behavior role does not block promotion. The manifest was not marked verified. Coordinator state updates must still account for the separate required review roles, including the pending `error_tracing_reviewer` gate.
