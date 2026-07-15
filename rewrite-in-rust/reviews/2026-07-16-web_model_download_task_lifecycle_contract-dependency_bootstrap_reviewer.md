# web_model_download_task_lifecycle_contract - dependency_bootstrap_reviewer

Date: 2026-07-16
Decision: pass

## Findings

No findings.

The manifest boundary is confirmed. The unit is correctly limited to model
download manager task lifecycle state: task creation/defaults, registry lookup,
active-task filtering, active-start conflict, fake thread metadata, running
transition, and no-process stop behavior
(`rewrite-in-rust/manifest.yaml:608`,
`rewrite-in-rust/manifest.yaml:617`,
`rewrite-in-rust/manifest.yaml:624`). The split records explicitly keep route
mapping, process planning, `_execute_download` result handling, process-tree
termination, SocketIO delivery, and real downloads out of this unit
(`rewrite-in-rust/records/0031-split-web-model-download-lifecycle-termination.md:29`,
`rewrite-in-rust/records/0031-split-web-model-download-lifecycle-termination.md:41`,
`rewrite-in-rust/records/0031-split-web-model-download-lifecycle-termination.md:52`,
`rewrite-in-rust/records/0032-confirm-web-model-download-task-lifecycle-boundary.md:29`).

The dependency record matches that split. It defines a legacy-owned library seam
with no bridge dependencies and explicit fixture inputs for UUIDs, clocks,
thread/event objects, selected-model mutation, and manager state
(`rewrite-in-rust/dependencies/web_model_download_task_lifecycle_contract.yaml:20`,
`rewrite-in-rust/dependencies/web_model_download_task_lifecycle_contract.yaml:24`).
It also keeps route mapping, process planning, execution/result status delivery,
process termination, downloads, package installation, archive extraction, and
asset marker safety legacy-owned or assigned to later units
(`rewrite-in-rust/dependencies/web_model_download_task_lifecycle_contract.yaml:39`,
`rewrite-in-rust/dependencies/web_model_download_task_lifecycle_contract.yaml:44`,
`rewrite-in-rust/dependencies/web_model_download_task_lifecycle_contract.yaml:46`,
`rewrite-in-rust/dependencies/web_model_download_task_lifecycle_contract.yaml:48`).

The bootstrap and checker enforce the no-runtime boundary. The bootstrap states
that Web routes, command/env planning, stdout parsing, `_execute_download`, real
thread target execution, subprocesses, process waiting/termination, SocketIO,
network downloads, package installation, archive extraction, and model marker
safety are excluded (`rewrite-in-rust/bootstrap/web_model_download_task_lifecycle_contract.md:38`).
The checker replaces events and threads with fakes whose `start()` method only
marks metadata and does not call the target
(`rewrite-in-rust/bootstrap/check_web_model_download_task_lifecycle_contract.py:25`,
`rewrite-in-rust/bootstrap/check_web_model_download_task_lifecycle_contract.py:36`,
`rewrite-in-rust/bootstrap/check_web_model_download_task_lifecycle_contract.py:44`).
Its `start_task` probe passes a dummy SocketIO object but never executes the
thread target (`rewrite-in-rust/bootstrap/check_web_model_download_task_lifecycle_contract.py:265`,
`rewrite-in-rust/bootstrap/check_web_model_download_task_lifecycle_contract.py:280`),
and the stop fixtures cover no-process state transitions only
(`rewrite-in-rust/bootstrap/check_web_model_download_task_lifecycle_contract.py:300`).

The Rust module is a fixture-bound state model, not a production bridge. Its
module docs keep Flask, SocketIO, subprocess execution, and OS termination owned
by legacy Python (`rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_lifecycle.rs:1`).
The implementation models only an in-memory lifecycle manager over task state
(`rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_lifecycle.rs:125`,
`rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_lifecycle.rs:138`,
`rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_lifecycle.rs:179`,
`rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_lifecycle.rs:215`),
and its tests consume the lifecycle fixture table directly
(`rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_lifecycle.rs:268`).
`lib.rs` only exposes the independent module; it does not connect Rust to the
Python runtime (`rewrite-in-rust/rust/crates/v2m-core/src/lib.rs:15`).

Legacy inspection supports the split. The covered lifecycle behavior lives in
`create_task`, `get_task`, `active_task`, `start_task`, and the no-process branch
of `stop_task` (`web_model_download_manager.py:136`,
`web_model_download_manager.py:149`,
`web_model_download_manager.py:174`,
`web_model_download_manager.py:217`). The effectful execution and process-tree
behavior are separate legacy methods (`web_model_download_manager.py:252`,
`web_model_download_manager.py:333`), while HTTP and SocketIO mapping remain in
`web_server.py` (`web_server.py:648`, `web_server.py:846`, `web_server.py:904`).
The public docs likewise describe route-level stop as terminating a child
process group when possible, which is intentionally reserved for later
termination work (`docs/web-api.md:310`).

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_web_model_download_task_lifecycle_contract.py`: pass
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml web_model_download_lifecycle`: pass
- `uv run pytest tests/test_web_api.py -k ModelDownload -q`: pass, 19 passed and 34 deselected
- `cargo tree --manifest-path rewrite-in-rust/rust/Cargo.toml -p v2m-core`: pass, only existing `serde_json` tree shown for `v2m-core`
- `cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check`: pass
- `cargo clippy --manifest-path rewrite-in-rust/rust/Cargo.toml --all-targets --all-features -- -D warnings`: pass
- `git diff --check -- rewrite-in-rust/manifest.yaml rewrite-in-rust/records/0031-split-web-model-download-lifecycle-termination.md rewrite-in-rust/records/0032-confirm-web-model-download-task-lifecycle-boundary.md rewrite-in-rust/dependencies/web_model_download_task_lifecycle_contract.yaml rewrite-in-rust/bootstrap/web_model_download_task_lifecycle_contract.md rewrite-in-rust/fixtures/web_model_download_task_lifecycle_contract.jsonl rewrite-in-rust/bootstrap/check_web_model_download_task_lifecycle_contract.py rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_lifecycle.rs rewrite-in-rust/rust/crates/v2m-core/src/lib.rs`: pass

## Residual Risk

This dependency/bootstrap review does not prove full behavior parity, error
tracing, or product ergonomics. It also intentionally leaves route response
mapping, process planning, `_execute_download` result transitions, process-tree
termination, real SocketIO delivery, real threads/subprocesses, and asset
download/archive behavior to their separate units or legacy runtime ownership.

## Promotion Note

This role does not block coordinator state update for
`web_model_download_task_lifecycle_contract`. The manifest should not be marked
`verified` by this review alone; the remaining required review roles are still
separate gates.
