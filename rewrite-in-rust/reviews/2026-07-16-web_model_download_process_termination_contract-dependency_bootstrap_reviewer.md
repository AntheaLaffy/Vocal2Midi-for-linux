# web_model_download_process_termination_contract - dependency_bootstrap_reviewer

Date: 2026-07-16
Decision: pass

## Findings

No findings.

The manifest boundary is confirmed. The unit should remain a narrow fake-process termination decision seam, not split, merged, deferred, or replaced.

Evidence:
- `rewrite-in-rust/manifest.yaml:653` defines the unit as `web_model_download_process_termination_contract`; `rewrite-in-rust/manifest.yaml:661` limits the public policy to process-present `stop_task`, POSIX process-group signal selection, Windows `taskkill` command construction, fallback methods, and already-exited no-op behavior.
- `rewrite-in-rust/manifest.yaml:668` through `rewrite-in-rust/manifest.yaml:671` require fixture parity and explicitly say not to terminate real processes.
- `rewrite-in-rust/records/0031-split-web-model-download-lifecycle-termination.md:52` through `rewrite-in-rust/records/0031-split-web-model-download-lifecycle-termination.md:65` assign OS-specific termination to this unit while excluding subprocess execution, network downloads, package installation, archive extraction, and model asset marker safety.
- `rewrite-in-rust/records/0034-confirm-web-model-download-process-termination-boundary.md:16` through `rewrite-in-rust/records/0034-confirm-web-model-download-process-termination-boundary.md:31` confirms the exact termination subset and excludes task lifecycle, process planning, execution-result handling, real process termination, network downloads, package installation, archive extraction, and marker safety.
- `rewrite-in-rust/dependencies/web_model_download_process_termination_contract.yaml:32` through `rewrite-in-rust/dependencies/web_model_download_process_termination_contract.yaml:42` assigns route mapping, process planning, task lifecycle, `_execute_download`, SocketIO, downloads, package installation, archive extraction, and marker safety to other units or later work.
- `rewrite-in-rust/bootstrap/web_model_download_process_termination_contract.md:45` through `rewrite-in-rust/bootstrap/web_model_download_process_termination_contract.md:47` states that fixtures inject fake process and fake OS APIs and do not call real OS termination APIs.
- `rewrite-in-rust/bootstrap/check_web_model_download_process_termination_contract.py:147` through `rewrite-in-rust/bootstrap/check_web_model_download_process_termination_contract.py:176` patches `os.killpg` and `subprocess.run` with local fakes before invoking legacy `_terminate_process_tree`.
- `rewrite-in-rust/bootstrap/check_web_model_download_process_termination_contract.py:52` through `rewrite-in-rust/bootstrap/check_web_model_download_process_termination_contract.py:61` overrides `_terminate_process_tree` for `stop_task`, so the stop-task branch records calls without killing a process.
- `rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_termination.rs:1` through `rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_termination.rs:5` declares the fake-process scope, and `rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_termination.rs:56` through `rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_termination.rs:145` records termination decisions without using `std::process` or OS signal APIs.
- Legacy source at `web_model_download_manager.py:217` through `web_model_download_manager.py:230` and `web_model_download_manager.py:333` through `web_model_download_manager.py:369` is the intended compatibility source for process-present `stop_task` and `_terminate_process_tree`.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_web_model_download_process_termination_contract.py`: passed.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml web_model_download_termination`: passed, 1 test passed.
- `uv run pytest tests/test_web_api.py -k 'stop_model_download or terminate_process_tree or popen_process_group_kwargs' -q`: passed, 6 tests passed and 47 deselected.
- `rg -n "Command::new|std::process|subprocess\\.Popen|subprocess\\.run\\(|os\\.kill|killpg\\(|taskkill|\\.kill\\(|\\.terminate\\(|download_models|requests|urllib|pip|extract_zip" rewrite-in-rust/bootstrap/check_web_model_download_process_termination_contract.py rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_termination.rs rewrite-in-rust/fixtures/web_model_download_process_termination_contract.jsonl`: inspected; matches are fixture strings, fake patch points, fake call recording, or documentation comments, not real process spawn/signal/kill calls.
- `cargo tree --manifest-path rewrite-in-rust/rust/Cargo.toml -p v2m-core`: inspected; this unit adds no process, networking, package-install, or archive dependencies beyond existing `serde_json`.
- `git diff --check -- rewrite-in-rust/manifest.yaml rewrite-in-rust/records/0031-split-web-model-download-lifecycle-termination.md rewrite-in-rust/records/0034-confirm-web-model-download-process-termination-boundary.md rewrite-in-rust/dependencies/web_model_download_process_termination_contract.yaml rewrite-in-rust/bootstrap/web_model_download_process_termination_contract.md rewrite-in-rust/fixtures/web_model_download_process_termination_contract.jsonl rewrite-in-rust/bootstrap/check_web_model_download_process_termination_contract.py rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_termination.rs`: passed.

## Residual Risk

This review proves the dependency/bootstrap boundary and fake-process harness shape. It does not prove real OS process-tree behavior on POSIX or Windows after a future bridge promotion, and it does not review behavior parity beyond the named fixtures, error tracing, Rust style, architecture, or product ergonomics.

## Promotion Note

This dependency/bootstrap role does not block coordinator state update. The unit remains legacy-owned at runtime, with rollback through `web_model_download_manager.ModelDownloadManager.stop_task` and `_terminate_process_tree`.
