# application_job_contract - error_tracing_reviewer

Date: 2026-07-16
Decision: pass-with-followups

## Findings

- Severity: low
- Location: rewrite-in-rust/rust/crates/v2m-core/src/application.rs:23
- Issue: The Rust seam and fixture table preserve the current contract's stable kind, message, and details strings, but they do not represent Python traceback causes for generic exceptions or specific `Vocal2MidiError` subclass identity. That is acceptable for the current legacy-owned, closure-modeled guard seam, but a future runtime bridge would need an explicit typed/source strategy to keep diagnosability equivalent to Python.
- Evidence: Python wraps unexpected pipeline exceptions with `raise Vocal2MidiError(...) from e`, preserving the original exception as `__cause__` (`application/pipeline.py:58`, `application/pipeline.py:62`). Python also defines specific subclasses under `Vocal2MidiError` (`application/exceptions.py:16`, `application/exceptions.py:20`, `application/exceptions.py:24`, `application/exceptions.py:28`, `application/exceptions.py:32`). The Rust model collapses pipeline results to `Vocal2MidiError { message, details }` or `OtherError { display }` (`rewrite-in-rust/rust/crates/v2m-core/src/application.rs:23`, `rewrite-in-rust/rust/crates/v2m-core/src/application.rs:28`, `rewrite-in-rust/rust/crates/v2m-core/src/application.rs:29`), and `ApplicationJobError` implements `std::error::Error` without a source chain (`rewrite-in-rust/rust/crates/v2m-core/src/application.rs:34`, `rewrite-in-rust/rust/crates/v2m-core/src/application.rs:75`). The shared fixture only asserts the base passthrough and generic string wrapping (`rewrite-in-rust/fixtures/application_job_contract.tsv:17`, `rewrite-in-rust/fixtures/application_job_contract.tsv:18`), and the checker records kind/message/details only (`rewrite-in-rust/bootstrap/check_application_job_contract.py:96`, `rewrite-in-rust/bootstrap/check_application_job_contract.py:182`, `rewrite-in-rust/bootstrap/check_application_job_contract.py:195`).
- Required fix: Before any caller-facing Rust runtime bridge owns pipeline exception mapping, either add typed/source/cause fields and fixtures for bridge-visible failures, or record that Python cause/subclass identity is intentionally outside that future contract.

## Checks

- `env UV_CACHE_DIR=/tmp/v2m-uv-cache uv run python rewrite-in-rust/bootstrap/check_application_job_contract.py`: passed with no output.
- `env CARGO_TARGET_DIR=/tmp/v2m-application-job-contract-error-target cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml application`: passed; 3 application tests passed in `v2m-core`, 57 filtered out, and 0 matching tests in `v2m-quant-bridge`.
- `rg -n "ApplicationJobError|source\\(|details\\(|eprintln|println|dbg!|log::|tracing::|secret|password|token|api[_-]?key|path\\.display|Pipeline execution failed|from e|from None" application/pipeline.py application/exceptions.py rewrite-in-rust/rust/crates/v2m-core/src/application.rs rewrite-in-rust/bootstrap/check_application_job_contract.py rewrite-in-rust/fixtures/application_job_contract.tsv`: confirmed no runtime logging calls in the Rust application seam or Python guard, confirmed Python generic cause wrapping, and confirmed model path details use `path.display()`.
- `git diff --name-only -- application/pipeline.py application/config.py application/exceptions.py rewrite-in-rust/rust/crates/v2m-core/src/application.rs rewrite-in-rust/bootstrap/check_application_job_contract.py rewrite-in-rust/fixtures/application_job_contract.tsv rewrite-in-rust/manifest.yaml`: only `rewrite-in-rust/manifest.yaml` has unrelated pre-existing working-tree changes; this review did not edit production code, fixtures, bootstrap scripts, Rust source, or manifest.

## Residual Risk

The contract intentionally exposes full model paths in `ModelNotFoundError.details` to match Python (`application/pipeline.py:23`, `application/pipeline.py:28`; `rewrite-in-rust/rust/crates/v2m-core/src/application.rs:95`, `rewrite-in-rust/rust/crates/v2m-core/src/application.rs:97`). This is useful local diagnostic context but can disclose filesystem paths if a caller forwards details to logs or remote clients. No new logging sink is introduced by this unit; Web/GUI redaction policy remains a caller-facing concern for later promotion work.

The cancellation surface is diagnosable at the current contract level: pre-start cancellation and interrupted pipeline use distinct messages and empty details, matching Python's `CancellationError` behavior (`application/pipeline.py:49`, `application/pipeline.py:55`; `rewrite-in-rust/rust/crates/v2m-core/src/application.rs:111`, `rewrite-in-rust/rust/crates/v2m-core/src/application.rs:119`; `rewrite-in-rust/fixtures/application_job_contract.tsv:15`, `rewrite-in-rust/fixtures/application_job_contract.tsv:16`).

## Promotion Note

This error/tracing role does not block the current legacy-owned unit from coordinator review, provided the low future-bridge diagnostic follow-up is either accepted as residual risk or handled before caller-facing Rust promotion. Runtime ownership remains legacy; I did not mark the manifest verified.
