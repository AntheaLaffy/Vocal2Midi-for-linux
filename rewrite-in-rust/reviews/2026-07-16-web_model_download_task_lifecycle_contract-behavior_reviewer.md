# web_model_download_task_lifecycle_contract - behavior_reviewer

Date: 2026-07-16
Decision: pass

## Findings

No findings.

Evidence:

- The unit boundary is limited to manager lifecycle state in `rewrite-in-rust/records/0032-confirm-web-model-download-task-lifecycle-boundary.md:15` and explicitly excludes route mapping, process planning, `_execute_download`, process termination, SocketIO delivery, and real downloads at `rewrite-in-rust/records/0032-confirm-web-model-download-task-lifecycle-boundary.md:27`.
- Legacy `create_task`, `get_task`, `active_task`, `start_task`, and no-process `stop_task` behavior is defined in `web_model_download_manager.py:136`, `web_model_download_manager.py:149`, `web_model_download_manager.py:174`, and `web_model_download_manager.py:217`.
- Rust lifecycle behavior mirrors the same state transitions in `rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_lifecycle.rs:138`, `rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_lifecycle.rs:167`, `rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_lifecycle.rs:172`, `rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_lifecycle.rs:179`, and `rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_lifecycle.rs:215`.
- The fixture table covers create defaults, selected-model copy, proxy URL trimming, get lookup, active status filtering, start conflict/success/replacement, and no-process stop behavior in `rewrite-in-rust/fixtures/web_model_download_task_lifecycle_contract.jsonl:1`.
- The checker patches UUID, clock, event, and thread creation without calling the thread target or spawning downloads in `rewrite-in-rust/bootstrap/check_web_model_download_task_lifecycle_contract.py:25`, `rewrite-in-rust/bootstrap/check_web_model_download_task_lifecycle_contract.py:53`, and `rewrite-in-rust/bootstrap/check_web_model_download_task_lifecycle_contract.py:265`.
- Web route response mapping remains outside this unit and is still owned by `web_server.py:648`, `web_server.py:715`, and `web_server.py:731`.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_web_model_download_task_lifecycle_contract.py`: pass
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml web_model_download_lifecycle`: pass
- `uv run pytest tests/test_web_api.py -k 'ModelDownload' -q`: pass, 19 passed
- `cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check`: pass
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml`: pass, 54 library tests and 5 bridge tests
- `uv run pytest tests/test_web_api.py -q`: pass, 53 passed
- Manual legacy probe for `stop_task` statuses `pending`, `running`, `stopping`, `completed`, `failed`, and `cancelled`: pending/running transition to `stopping` and return true; all other statuses return false
- `git diff --check -- rewrite-in-rust/fixtures/web_model_download_task_lifecycle_contract.jsonl rewrite-in-rust/bootstrap/check_web_model_download_task_lifecycle_contract.py rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_lifecycle.rs rewrite-in-rust/manifest.yaml rewrite-in-rust/dependencies/web_model_download_task_lifecycle_contract.yaml rewrite-in-rust/bootstrap/web_model_download_task_lifecycle_contract.md rewrite-in-rust/records/0031-split-web-model-download-lifecycle-termination.md rewrite-in-rust/records/0032-confirm-web-model-download-task-lifecycle-boundary.md`: pass

## Residual Risk

This behavior review does not prove route status-code mapping, command/env planning, stdout parsing, `_execute_download` result transitions, process-tree termination, SocketIO delivery, network downloads, package installation, archive extraction, or model marker checks. Those are explicitly assigned to adjacent units or left legacy-owned.

The fixture table samples `completed` as the non-running stop case; code inspection and the manual legacy probe confirm the branch also rejects `stopping`, `failed`, and `cancelled`, but those exact statuses are not separate fixture rows.

## Promotion Note

This behavior role does not block coordinator state update for `web_model_download_task_lifecycle_contract`. Coordinator state should still wait for the other required review roles before any promotion decision.
