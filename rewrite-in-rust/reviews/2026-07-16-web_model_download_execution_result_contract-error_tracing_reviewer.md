# web_model_download_execution_result_contract - error_tracing_reviewer

Date: 2026-07-16
Decision: pass

## Findings

No findings.

## Evidence

The legacy nonzero return-code branch sets `status = "failed"`, `stage = "failed"`,
`error = "download_models.py exited with code <returncode>"`, emits that error
log, then emits `status_change` (`web_model_download_manager.py:288`,
`web_model_download_manager.py:296`, `web_model_download_manager.py:302`). The
fixture preserves the user-visible code-7 string and status payload
(`rewrite-in-rust/fixtures/web_model_download_execution_result_contract.jsonl:2`),
and Rust maps the same string before status emission
(`rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_execution.rs:199`,
`rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_execution.rs:212`).

The legacy exception branch stores `str(exc)`, logs `str(exc)`, logs
`traceback.format_exc()`, and emits status (`web_model_download_manager.py:303`,
`web_model_download_manager.py:310`). The Python checker normalizes only the
volatile traceback body while still requiring a traceback log marker
(`rewrite-in-rust/bootstrap/check_web_model_download_execution_result_contract.py:177`,
`rewrite-in-rust/bootstrap/check_web_model_download_execution_result_contract.py:186`),
and the fixture/Rust path require the final error log to be `__traceback__`
(`rewrite-in-rust/fixtures/web_model_download_execution_result_contract.jsonl:4`,
`rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_execution.rs:239`,
`rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_execution.rs:254`).

Status payload coverage includes status, stage, progress, error, returncode,
completed_at, redacted proxy_url, and log count
(`rewrite-in-rust/bootstrap/check_web_model_download_execution_result_contract.py:256`,
`rewrite-in-rust/bootstrap/check_web_model_download_execution_result_contract.py:268`;
`rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_execution.rs:536`,
`rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_execution.rs:570`).
Proxy redaction is preserved through `serialize_task`
(`web_model_download_manager.py:232`, `web_model_download_manager.py:243`,
`web_model_download_manager.py:535`, `web_model_download_manager.py:542`) and
through the Rust status serializer
(`rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_execution.rs:286`,
`rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_execution.rs:304`).
The manual-proxy success fixture verifies the subprocess env receives the real
proxy while the user-visible `status_change.proxy_url` is redacted
(`rewrite-in-rust/fixtures/web_model_download_execution_result_contract.jsonl:1`).

The fake harness does not hide live process or network behavior: the boundary
documents fake Popen/process/stdout/socket/event inputs and explicitly excludes
real subprocess execution, SocketIO delivery failures, OS process-tree
termination, downloads, package installation, archive extraction, and model
marker safety (`rewrite-in-rust/bootstrap/web_model_download_execution_result_contract.md:31`,
`rewrite-in-rust/bootstrap/web_model_download_execution_result_contract.md:50`;
`rewrite-in-rust/dependencies/web_model_download_execution_result_contract.yaml:30`,
`rewrite-in-rust/dependencies/web_model_download_execution_result_contract.yaml:55`).

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_web_model_download_execution_result_contract.py`: pass
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml web_model_download_execution`: pass
- `uv run pytest tests/test_web_api.py -k ModelDownload -q`: pass, 19 passed and 34 deselected
- `git diff --check -- web_model_download_manager.py rewrite-in-rust/bootstrap/check_web_model_download_execution_result_contract.py rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_execution.rs rewrite-in-rust/fixtures/web_model_download_execution_result_contract.jsonl rewrite-in-rust/bootstrap/web_model_download_execution_result_contract.md rewrite-in-rust/dependencies/web_model_download_execution_result_contract.yaml rewrite-in-rust/records/0033-confirm-web-model-download-execution-result-boundary.md`: pass
- Manual legacy probe for success, nonzero failure, and Popen exception fixtures: pass; status payloads were `completed` with redacted proxy, `failed` with `download_models.py exited with code 7`, and `failed` with `popen boom` plus traceback log marker.

## Residual Risk

The traceback body is intentionally normalized rather than byte-for-byte
compared. This review proves that a traceback log is represented and ordered,
not that Python traceback frame text is preserved in Rust. Live SocketIO
delivery errors, OS process termination, network failures, and proxy exposure
outside emitted task/status payloads remain excluded from this unit.

## Promotion Note

This error-tracing role does not block coordinator state update for
`web_model_download_execution_result_contract`. Other required review roles are
separate gates.
