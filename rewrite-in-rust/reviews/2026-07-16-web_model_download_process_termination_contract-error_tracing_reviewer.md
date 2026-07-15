# web_model_download_process_termination_contract - error_tracing_reviewer

Date: 2026-07-16
Decision: pass

## Findings

No findings.

## Evidence

- Legacy `_terminate_process_tree` returns without logging when the process is already exited, when POSIX `ProcessLookupError` is raised, and after successful fallback from POSIX/Windows `OSError` to `process.terminate()` or `process.kill()` (`web_model_download_manager.py:333`, `web_model_download_manager.py:337`, `web_model_download_manager.py:344`, `web_model_download_manager.py:352`, `web_model_download_manager.py:360`, `web_model_download_manager.py:362`, `web_model_download_manager.py:364`).
- Legacy `stop_task` updates `status` to `stopping`, sets the stop event, invokes termination only for a live process, and returns `False` when `_terminate_process_tree` raises `OSError`; it does not log or attach the exception to task state in this path (`web_model_download_manager.py:217`, `web_model_download_manager.py:221`, `web_model_download_manager.py:222`, `web_model_download_manager.py:225`, `web_model_download_manager.py:226`, `web_model_download_manager.py:228`).
- Fixtures intentionally represent the silent/error boundary: POSIX `ProcessLookupError` is a no-op, POSIX and Windows `OSError` paths fall back to `terminate`/`kill`, and `stop_task_live_process_oserror_returns_false_after_state_update` asserts `success: false` while preserving `status: stopping` and the set stop event (`rewrite-in-rust/fixtures/web_model_download_process_termination_contract.jsonl:4`, `rewrite-in-rust/fixtures/web_model_download_process_termination_contract.jsonl:5`, `rewrite-in-rust/fixtures/web_model_download_process_termination_contract.jsonl:6`, `rewrite-in-rust/fixtures/web_model_download_process_termination_contract.jsonl:9`, `rewrite-in-rust/fixtures/web_model_download_process_termination_contract.jsonl:10`, `rewrite-in-rust/fixtures/web_model_download_process_termination_contract.jsonl:12`).
- The Python checker fakes `os.name`, `os.killpg`, `subprocess.run`, and the manager-level termination failure without starting or killing a process. It records only calls and task state, which matches the legacy lack of structured logs/traces for these branches (`rewrite-in-rust/bootstrap/check_web_model_download_process_termination_contract.py:147`, `rewrite-in-rust/bootstrap/check_web_model_download_process_termination_contract.py:155`, `rewrite-in-rust/bootstrap/check_web_model_download_process_termination_contract.py:167`, `rewrite-in-rust/bootstrap/check_web_model_download_process_termination_contract.py:186`).
- The Rust model mirrors the same diagnostic surface: `ProcessLookupError`/fallback outcomes alter call traces, and `stop_task_with_process` returns `success: false` on injected termination `OSError` after status/event mutation, without inventing logs, error strings, traceback, or redaction behavior not present in legacy (`rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_termination.rs:94`, `rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_termination.rs:99`, `rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_termination.rs:114`, `rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_termination.rs:127`, `rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_termination.rs:132`, `rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_termination.rs:138`).
- The fake harness does not hide the manager-level user-visible failure signal: `stop_task` false is preserved. HTTP/SocketIO response mapping remains outside this unit, and the legacy HTTP route turns that false into a 400 "cannot be stopped" response using the already-mutated task status (`web_server.py:748`, `web_server.py:749`, `web_server.py:750`, `web_server.py:751`, `web_server.py:753`; `rewrite-in-rust/bootstrap/web_model_download_process_termination_contract.md:30`).

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_web_model_download_process_termination_contract.py`: passed
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml web_model_download_termination`: passed
- `uv run pytest tests/test_web_api.py -k ModelDownload -q`: passed, 19 passed and 34 deselected
- `cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check`: passed
- `git diff --check -- web_model_download_manager.py rewrite-in-rust/fixtures/web_model_download_process_termination_contract.jsonl rewrite-in-rust/bootstrap/check_web_model_download_process_termination_contract.py rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_termination.rs rewrite-in-rust/bootstrap/web_model_download_process_termination_contract.md rewrite-in-rust/dependencies/web_model_download_process_termination_contract.yaml rewrite-in-rust/records/0034-confirm-web-model-download-process-termination-boundary.md rewrite-in-rust/manifest.yaml`: passed
- `rg -n "log|trace|emit|error|OSError|ProcessLookupError|false|failed|failure" rewrite-in-rust/fixtures/web_model_download_process_termination_contract.jsonl rewrite-in-rust/bootstrap/check_web_model_download_process_termination_contract.py rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_termination.rs rewrite-in-rust/bootstrap/web_model_download_process_termination_contract.md rewrite-in-rust/dependencies/web_model_download_process_termination_contract.yaml`: reviewed; no over-claimed logging/tracing path found.

## Residual Risk

The unit models process termination decisions with fakes and intentionally does not prove real OS behavior for `os.killpg`, `taskkill`, `process.terminate`, or `process.kill`. It also does not fixture the Web route response for termination failure; this role only confirms that the manager-level `False` signal is preserved for the route layer to expose.

## Promotion Note

This error tracing role does not block coordinator state update for `web_model_download_process_termination_contract`. Other required roles remain separate gates.
