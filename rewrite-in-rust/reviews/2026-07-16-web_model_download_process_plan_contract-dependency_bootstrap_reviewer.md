# web_model_download_process_plan_contract - dependency_bootstrap_reviewer

Date: 2026-07-16
Decision: pass

## Findings

No findings.

The manifest unit boundary is confirmed. The unit is marked reimplemented and
limits the public policy to command construction, proxy environment shaping,
process-group spawn kwargs, stdout framing, model guessing, per-model progress
math, log classification, and log cap behavior without starting a child process
(`rewrite-in-rust/manifest.yaml:585`). Record 0030 confirms the same
fixture-bound split and explicitly excludes real `subprocess.Popen`,
`download_models.py` execution, task registry creation, active-task locking,
SocketIO delivery guarantees, lifecycle transitions, process-tree termination,
network downloads, package installation, archive extraction, and asset marker
safety (`rewrite-in-rust/records/0030-confirm-web-model-download-process-plan-boundary.md:16`,
`rewrite-in-rust/records/0030-confirm-web-model-download-process-plan-boundary.md:31`).

The dependency record chooses an appropriate hand-written replacement seam. It
uses a library seam with no bridge dependencies, injects Python executable,
root directory, OS name, environment, fake process output, and socket emissions,
and keeps the heavy runtime owners in Python
(`rewrite-in-rust/dependencies/web_model_download_process_plan_contract.yaml:20`,
`rewrite-in-rust/dependencies/web_model_download_process_plan_contract.yaml:24`,
`rewrite-in-rust/dependencies/web_model_download_process_plan_contract.yaml:39`).
The bootstrap note matches that seam and states that fixture parity uses
explicit JSON inputs, no production bridge, and unchanged rollback ownership
(`rewrite-in-rust/bootstrap/web_model_download_process_plan_contract.md:65`,
`rewrite-in-rust/bootstrap/web_model_download_process_plan_contract.md:74`,
`rewrite-in-rust/bootstrap/web_model_download_process_plan_contract.md:91`).

The fixture and checker coverage is suitable for this dependency gate. Fixtures
cover command ordering, qwen source and force flags, duplicate selected IDs,
system/none/manual proxy environment behavior, POSIX/Windows spawn kwargs,
line classification, model guessing, legacy percent parsing, progress clamping,
read-output framing, stdout `None`, and log cap behavior
(`rewrite-in-rust/fixtures/web_model_download_process_plan_contract.jsonl:1`,
`rewrite-in-rust/fixtures/web_model_download_process_plan_contract.jsonl:4`,
`rewrite-in-rust/fixtures/web_model_download_process_plan_contract.jsonl:10`,
`rewrite-in-rust/fixtures/web_model_download_process_plan_contract.jsonl:16`,
`rewrite-in-rust/fixtures/web_model_download_process_plan_contract.jsonl:18`).
The Python checker uses fake stdout, fake process, and fake socket objects, then
calls legacy helper methods directly; it does not spawn downloads or contact
network services
(`rewrite-in-rust/bootstrap/check_web_model_download_process_plan_contract.py:25`,
`rewrite-in-rust/bootstrap/check_web_model_download_process_plan_contract.py:39`,
`rewrite-in-rust/bootstrap/check_web_model_download_process_plan_contract.py:52`,
`rewrite-in-rust/bootstrap/check_web_model_download_process_plan_contract.py:251`).

The Rust implementation stays fixture-bound. It documents that it does not
spawn `download_models.py`, own task lifecycle, or replace SocketIO transport,
and it implements pure command/env/kwargs/output-parser functions over explicit
task state and fixture inputs
(`rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_process.rs:1`,
`rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_process.rs:146`,
`rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_process.rs:172`,
`rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_process.rs:206`,
`rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_process.rs:215`).
The module is only exposed as an independent `v2m-core` module, and the crate
dependency surface remains narrow at `serde_json`
(`rewrite-in-rust/rust/crates/v2m-core/src/lib.rs:16`,
`rewrite-in-rust/rust/crates/v2m-core/Cargo.toml:12`).

Legacy references support the split. Process planning and output parsing are
localized in `ModelDownloadManager._build_command`,
`_popen_process_group_kwargs`, `_build_process_env`, `_read_process_output`,
`_handle_output_line`, `_emit_progress_for_model`, `_guess_model_from_line`,
`_emit_log`, and `_emit_progress`
(`web_model_download_manager.py:316`, `web_model_download_manager.py:327`,
`web_model_download_manager.py:370`, `web_model_download_manager.py:392`,
`web_model_download_manager.py:419`, `web_model_download_manager.py:440`,
`web_model_download_manager.py:452`, `web_model_download_manager.py:468`,
`web_model_download_manager.py:484`). The metadata reference is limited to
model IDs/assets/target names and qwen directory names
(`download_models.py:79`, `download_models.py:514`). Existing Web API tests
separately cover request/catalog and proxy/kwargs/termination concerns, with
termination tests remaining outside this unit
(`tests/test_web_api.py:602`, `tests/test_web_api.py:661`,
`tests/test_web_api.py:687`).

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_web_model_download_process_plan_contract.py`: pass
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml web_model_download_process`: pass, 2 selected tests passed
- `uv run pytest tests/test_web_api.py -k ModelDownload -q`: pass, 19 selected tests passed
- `cargo tree --manifest-path rewrite-in-rust/rust/Cargo.toml -p v2m-core`: pass, direct dependency tree is `v2m-core -> serde_json`
- `cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check`: pass

## Residual Risk

This review does not approve behavior parity, error tracing, product
ergonomics, or promotion. Lifecycle/termination behavior remains planned under
`web_model_download_lifecycle_termination_contract`, and network/download/archive
behavior remains planned under `download_models_asset_safety`.

## Promotion Note

Dependency/bootstrap review passes for this unit. This role does not block a
coordinator state update, but the manifest still lists `stage_behavior_reviewer`,
`error_tracing_reviewer`, and `product_ergonomics_reviewer` before the unit
should be treated as promotion-ready.
