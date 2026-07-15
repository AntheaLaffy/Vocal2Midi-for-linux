# web_model_download_request_catalog_contract - dependency_bootstrap_reviewer

Date: 2026-07-16
Decision: pass

## Findings

No findings.

The manifest boundary is confirmed. The original broad model-download unit was
re-cut into request/catalog, process-planning, and lifecycle/termination slices;
only `web_model_download_request_catalog_contract` is marked reimplemented, and
the process/lifecycle units remain planned with separate verification scopes
(`rewrite-in-rust/manifest.yaml:562`, `rewrite-in-rust/manifest.yaml:585`,
`rewrite-in-rust/manifest.yaml:606`). Record 0029 explicitly excludes
subprocess command construction, proxy environment shaping, output parsing,
SocketIO behavior, active-task lock ownership, process termination, real
downloads, network access, and archive safety from this unit
(`rewrite-in-rust/records/0029-split-web-model-download-contract.md:37`).

The dependency/bootstrap record chooses a library seam with no bridge
dependencies and keeps the right legacy owners: Flask route binding and JSON
serialization, real filesystem marker checks, threading/subprocess/SocketIO/
termination behavior, and actual `download_models.py` execution/network/archive
work all stay legacy-owned (`rewrite-in-rust/dependencies/web_model_download_request_catalog_contract.yaml:20`,
`rewrite-in-rust/dependencies/web_model_download_request_catalog_contract.yaml:43`).
The bootstrap note matches that seam and states that fixture parity uses
injected install states, fake tasks, and fake task-manager outcomes without a
live Web server, SocketIO transport, model runtime, download process, network,
or archive extraction (`rewrite-in-rust/bootstrap/web_model_download_request_catalog_contract.md:58`,
`rewrite-in-rust/bootstrap/web_model_download_request_catalog_contract.md:64`).

The fixture/checker coverage is appropriate for this dependency gate. Fixtures
cover catalog status, active task serialization with proxy redaction, explicit
and default start requests, deduplication, Python force truthiness, validation
errors, conflict mapping, status lookup, and stop mapping
(`rewrite-in-rust/fixtures/web_model_download_request_catalog_contract.jsonl:1`).
The Python checker patches install-state helpers, manager task state, start
outcomes, status lookup, and stop outcomes instead of starting downloads or
touching real model files
(`rewrite-in-rust/bootstrap/check_web_model_download_request_catalog_contract.py:98`,
`rewrite-in-rust/bootstrap/check_web_model_download_request_catalog_contract.py:155`,
`rewrite-in-rust/bootstrap/check_web_model_download_request_catalog_contract.py:208`,
`rewrite-in-rust/bootstrap/check_web_model_download_request_catalog_contract.py:227`).

The Rust implementation stays inside the hand-written, fixture-bound contract.
It declares the legacy Python runtime owner for Flask, SocketIO, subprocesses,
real marker checks, and downloads, and exposes only the request/catalog module
from `v2m-core` (`rewrite-in-rust/rust/crates/v2m-core/src/web_model_download.rs:1`,
`rewrite-in-rust/rust/crates/v2m-core/src/lib.rs:15`). Its only direct crate
dependency for this modeling work is `serde_json`
(`rewrite-in-rust/rust/crates/v2m-core/Cargo.toml:12`).

Legacy/source references support the selected surface. The route contract lives
in `web_server.py` model status/start/status lookup/stop handlers
(`web_server.py:629`, `web_server.py:648`, `web_server.py:715`,
`web_server.py:731`); catalog serialization, task serialization, validation,
and proxy redaction live in `web_model_download_manager.py`
(`web_model_download_manager.py:103`, `web_model_download_manager.py:232`,
`web_model_download_manager.py:506`, `web_model_download_manager.py:535`);
asset metadata and install probes are narrow local references
(`download_models.py:79`, `download_models.py:319`, `download_models.py:514`).
The existing Web API docs and tests cover the same public endpoint fields and
error classes (`docs/web-api.md:240`, `tests/test_web_api.py:359`).

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_web_model_download_request_catalog_contract.py`: pass
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml web_model_download`: pass, 2 tests passed
- `uv run pytest tests/test_web_api.py -k 'ModelDownload'`: pass, 19 selected tests passed
- `cargo tree --manifest-path rewrite-in-rust/rust/Cargo.toml -p v2m-core`: pass, direct unit dependency shape is `v2m-core -> serde_json`
- `cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check`: pass
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml`: pass, 51 library tests, 5 bridge-bin tests, and 0 doctests passed
- `uv run pytest tests/test_web_api.py`: pass, 53 tests passed
- `git diff --check -- rewrite-in-rust/manifest.yaml rewrite-in-rust/resources.md rewrite-in-rust/notes.md rewrite-in-rust/rust/crates/v2m-core/src/lib.rs rewrite-in-rust/rust/crates/v2m-core/Cargo.toml rewrite-in-rust/rust/Cargo.lock`: pass

## Residual Risk

This review does not approve behavior parity, error tracing, product ergonomics,
or promotion. The process-planning and lifecycle/termination split units still
need their own dependency/bootstrap and behavior evidence before any broader
model-download runtime ownership changes.

## Promotion Note

Dependency/bootstrap review passes for this unit. This role does not block a
coordinator state update for the dependency gate, but the manifest still lists
`stage_behavior_reviewer`, `error_tracing_reviewer`, and
`product_ergonomics_reviewer` before the unit should be treated as fully ready
for promotion planning.
