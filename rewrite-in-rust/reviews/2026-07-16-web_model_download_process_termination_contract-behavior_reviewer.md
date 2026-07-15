# web_model_download_process_termination_contract - behavior_reviewer

Date: 2026-07-16
Decision: pass

## Findings

No findings.

## Behavior Evidence

- Legacy `stop_task` sets status to `stopping`, sets the stop event, invokes
  `_terminate_process_tree` only for a live process, returns `false` on
  termination `OSError`, and otherwise returns `true`:
  `web_model_download_manager.py:217`.
- Legacy `_terminate_process_tree` no-ops for already-exited processes, uses
  Windows `taskkill /PID <pid> /T` with optional `/F`, falls back to
  `terminate`/`kill` on Windows `OSError`, and uses POSIX `killpg` with
  `SIGTERM`/`SIGKILL`, `ProcessLookupError` no-op, and `OSError`
  `terminate`/`kill` fallback: `web_model_download_manager.py:333`.
- The fixture table covers already-exited no-op, POSIX normal/force signals,
  POSIX `ProcessLookupError`, POSIX fallback, Windows normal/force taskkill,
  Windows fallback, and `stop_task` success/OSError state-update outcomes:
  `rewrite-in-rust/fixtures/web_model_download_process_termination_contract.jsonl:1`.
- The Python checker runs the legacy code with fake process objects and patched
  `os.killpg`/`subprocess.run`, so no real signal, taskkill, or process
  termination is performed:
  `rewrite-in-rust/bootstrap/check_web_model_download_process_termination_contract.py:36`,
  `rewrite-in-rust/bootstrap/check_web_model_download_process_termination_contract.py:147`,
  `rewrite-in-rust/bootstrap/check_web_model_download_process_termination_contract.py:155`,
  `rewrite-in-rust/bootstrap/check_web_model_download_process_termination_contract.py:173`.
- The Rust unit is a fake decision model, not an OS integration: it records
  taskkill/killpg/process-call traces, preserves the same live-process and
  fallback branches, and consumes the same fixture table:
  `rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_termination.rs:1`,
  `rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_termination.rs:57`,
  `rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_termination.rs:115`,
  `rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_termination.rs:151`.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_web_model_download_process_termination_contract.py`: passed.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml web_model_download_termination`: passed, 1 test passed.
- `uv run pytest tests/test_web_api.py -k ModelDownload -q`: passed, 19 passed and 34 deselected.
- `rg -n "std::process|Command::new|libc::|nix::|killpg|SIGTERM|SIGKILL|taskkill|subprocess.run|os.killpg|process\\.terminate|process\\.kill" rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_termination.rs rewrite-in-rust/bootstrap/check_web_model_download_process_termination_contract.py`: scoped scan found only fake trace strings/helpers and patched legacy-call harness code.
- `git diff --check -- rewrite-in-rust/fixtures/web_model_download_process_termination_contract.jsonl rewrite-in-rust/bootstrap/check_web_model_download_process_termination_contract.py rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_termination.rs rewrite-in-rust/bootstrap/web_model_download_process_termination_contract.md rewrite-in-rust/dependencies/web_model_download_process_termination_contract.yaml rewrite-in-rust/records/0034-confirm-web-model-download-process-termination-boundary.md rewrite-in-rust/manifest.yaml`: passed.

## Residual Risk

This review proves fixture-backed branch parity only. It intentionally does not
prove production OS process-tree behavior against real child processes, because
the unit boundary excludes real termination and keeps
`ModelDownloadManager.stop_task` and `_terminate_process_tree` as runtime owners
until a later promotion record.

## Promotion Note

This behavior role does not block coordinator state update. Dependency,
error-tracing, and product ergonomics reviews remain separate gates if required
for this unit.
