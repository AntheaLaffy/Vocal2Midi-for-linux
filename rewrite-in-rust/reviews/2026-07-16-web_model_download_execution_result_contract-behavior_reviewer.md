# web_model_download_execution_result_contract - behavior_reviewer

Date: 2026-07-16
Decision: pass

## Findings

No findings.

## Evidence

- Legacy behavior reviewed at `web_model_download_manager.py:252`: startup log/progress, fake Popen assignment, output-reader handoff, cancellation branch, wait/returncode mapping, exception handler, status emission, and active task cleanup.
- Boundary and fixture intent reviewed at `rewrite-in-rust/records/0033-confirm-web-model-download-execution-result-boundary.md:25`, `rewrite-in-rust/dependencies/web_model_download_execution_result_contract.yaml:1`, and `rewrite-in-rust/bootstrap/web_model_download_execution_result_contract.md:5`.
- Fixture coverage reviewed at `rewrite-in-rust/fixtures/web_model_download_execution_result_contract.jsonl:1`: success return code 0 with output, nonzero failure, cancellation with timeout escalation, Popen exception, and preserving a different active task id.
- Python checker reviewed at `rewrite-in-rust/bootstrap/check_web_model_download_execution_result_contract.py:309`: it replaces Popen, environment, root path, process waits, termination, output, and SocketIO with fakes.
- Rust implementation reviewed at `rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_execution.rs:104`: it records fake Popen calls, assigns the fake process, hands off to fake output parsing, maps completion/failure/cancellation/exception states, emits status summaries, and clears `active_task_id` only when it still points at the task.
- Dependency helpers reviewed at `rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_process.rs:146` and `rewrite-in-rust/rust/crates/v2m-core/src/web_model_download.rs:187`.
- No real download, subprocess spawn, process kill, or network behavior was found in the Rust unit; searched for execution/network symbols and found only fake Popen records plus command-vector construction.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_web_model_download_execution_result_contract.py`: pass
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml web_model_download_execution`: pass
- `uv run pytest tests/test_web_api.py -k ModelDownload -q`: pass, 19 passed and 34 deselected
- `cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check`: pass
- `git diff --check -- rewrite-in-rust/fixtures/web_model_download_execution_result_contract.jsonl rewrite-in-rust/bootstrap/check_web_model_download_execution_result_contract.py rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_execution.rs rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_process.rs rewrite-in-rust/rust/crates/v2m-core/src/web_model_download.rs web_model_download_manager.py`: pass

## Residual Risk

This behavior review proves fixture-backed result-state parity only. It does not prove real SocketIO delivery, OS process-tree termination, live subprocess I/O races, network downloads, package installation, archive extraction, or model marker safety; those remain legacy-owned or assigned to later units.

## Promotion Note

This behavior role does not block coordinator state update for `web_model_download_execution_result_contract`. Other required roles remain separate gates.
