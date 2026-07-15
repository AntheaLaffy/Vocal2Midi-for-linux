# application_job_contract - dependency_bootstrap_reviewer

Date: 2026-07-15
Decision: pass-with-followups

## Findings

- Severity: low
- Location: rewrite-in-rust/fixtures/application_job_contract.tsv:6
- Issue: The fixture table proves individual missing GAME, HubertFA, and ASR path errors plus the HubertFA+ASR combined case, but it does not include an order-sensitive case where GAME and one or both lyric paths are missing together, nor a case where paths are invalid while `cancel_checker` is true. The implementation currently orders validation correctly, but the fixture set does not lock those dependency/bootstrap claims down.
- Evidence: The bootstrap contract says model-path details are joined in Python check order and cancellation is checked after path validation (`rewrite-in-rust/bootstrap/application_job_contract.md:15`, `rewrite-in-rust/bootstrap/application_job_contract.md:19`). Python validates paths before cancellation (`application/pipeline.py:14`, `application/pipeline.py:46`), and Rust mirrors that order (`rewrite-in-rust/rust/crates/v2m-core/src/application.rs:80`, `rewrite-in-rust/rust/crates/v2m-core/src/application.rs:109`). Existing fixture rows cover single missing paths, HFA+ASR, and valid-path cancellation separately (`rewrite-in-rust/fixtures/application_job_contract.tsv:6`, `rewrite-in-rust/fixtures/application_job_contract.tsv:10`, `rewrite-in-rust/fixtures/application_job_contract.tsv:12`).
- Required fix: Add fixture rows before final verification for at least all-required-paths-missing detail order (`GAME 模型目录|HubertFA 模型目录|ASR 模型路径`) and invalid-path-with-cancel-true precedence, then rerun the Python checker and targeted Cargo test. This is a coverage follow-up, not a seam or dependency blocker.

## Boundary And Inventory

Manifest unit boundary: confirmed.

The confirmed scope is the application-layer guard around `application/pipeline.py::_validate_model_paths` and `application/pipeline.py::run_auto_lyric_job`: required GAME path validation, lyric-gated HFA/ASR validation, pre-start cancellation ordering, `InterruptedError` mapping, `Vocal2MidiError` passthrough, and generic exception wrapping (`rewrite-in-rust/manifest.yaml:389`, `rewrite-in-rust/manifest.yaml:399`; `rewrite-in-rust/records/0017-confirm-application-job-contract-boundary.md:19`).

The kept-legacy decisions are appropriate for this role. `auto_lyric_hybrid_pipeline`, model loading/execution, GUI/Web config construction, Flask/SocketIO routing, and full `PipelineConfig` mapping remain outside this unit (`rewrite-in-rust/records/0017-confirm-application-job-contract-boundary.md:31`; `rewrite-in-rust/dependencies/application_job_contract.yaml:44`). That keeps the unit out of model runtime and frontend ownership.

The seam choice is acceptable: an independent `v2m-core` library model with a closure for the legacy pipeline call and no bridge dependencies (`rewrite-in-rust/dependencies/application_job_contract.yaml:20`; `rewrite-in-rust/bootstrap/application_job_contract.md:64`). The Rust core crate does not add Flask, PyQt, ONNX Runtime, Qwen, PyO3, HTTP, subprocess, or model-runtime crates (`rewrite-in-rust/rust/crates/v2m-core/Cargo.toml:1`).

The hand-written replacement rationale is sound. The Python project carries broad runtime dependencies including Flask, PyQt, librosa, mido, ONNX Runtime, qwen-asr, soundfile, torch, and related packages (`pyproject.toml:8`), and vendored-source audit coverage exists for the installed environment including upstream fallbacks for ONNX Runtime and torch (`third_party/source_audit.json:2`; `third_party/sources/MISSING_SOURCES.md:12`). This unit only needs path-existence, local exception, and call-order behavior, so a narrow fixture-bound Rust implementation is preferable to dependency parity.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_application_job_contract.py`: passed after rerun with approved `uv` cache access; the first sandboxed attempt failed because `/home/fuurin/.cache/uv` is read-only in the sandbox.
- `env CARGO_TARGET_DIR=/tmp/v2m-application-job-contract-target cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml application`: passed; 3 application tests passed, 37 filtered out in `v2m-core`, and 0 matching tests in `v2m-quant-bridge`.
- `rg -n "pub mod application|serde|pyo3|onnx|torch|qwen|flask|pyqt|reqwest|tokio" rewrite-in-rust/rust/crates/v2m-core/Cargo.toml rewrite-in-rust/rust/crates/v2m-core/src/lib.rs rewrite-in-rust/rust/crates/v2m-core/src/application.rs`: only found the `pub mod application` export; no heavy runtime or bridge crate references in `v2m-core`.
- `git status --short`: used to identify the unit's touched files; production/runtime Python files were not modified by this review.

## Residual Risk

The closure seam proves validation order, dispatch occurrence, and error mapping, but it intentionally does not prove a future runtime bridge can transfer the full `PipelineConfig.to_kwargs()` payload. The current Python checker protects the legacy passthrough (`rewrite-in-rust/bootstrap/check_application_job_contract.py:204`); any later promotion bridge needs its own record and parity evidence.

The fixture follow-up above should be addressed or explicitly accepted by the behavior reviewer before this unit is used as final promotion evidence.

## Promotion Note

This dependency bootstrap role does not block the unit boundary. The unit may proceed to the required behavior and error/tracing reviews, but the coordinator should not mark it fully verified until the low fixture follow-up is addressed or consciously carried as accepted residual risk.
