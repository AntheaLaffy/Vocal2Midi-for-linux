# web_settings_contract - dependency_bootstrap_reviewer

Date: 2026-07-16
Decision: pass

## Findings

No findings.

## Boundary Decision

The manifest unit boundary is confirmed.

`web_settings_contract` is appropriately scoped to `web_server.py` settings defaults, merge/load/save helpers, and the GET/PUT/reset settings route contracts. The dependency record and bootstrap note explicitly keep Flask route ownership, pipeline-start config assembly, model download validation, filesystem picker behavior, output download authorization, SocketIO, model runtime, and network/download execution out of this unit.

## Evidence

- `rewrite-in-rust/manifest.yaml:499` marks `web_settings_contract` as reimplemented, confirmed, legacy-owned, and limited to `web_server.py` settings behavior.
- `rewrite-in-rust/dependencies/web_settings_contract.yaml:1` records a library seam, no bridge dependencies, confirmed inventory impact, and kept-legacy decisions for Flask routing, pipeline config assembly, filesystem/download security, and model download behavior.
- `rewrite-in-rust/bootstrap/web_settings_contract.md:5` defines the compatibility boundary and explicitly excludes pipeline-start config assembly, model download request validation, filesystem picker behavior, output download authorization, and live Flask/SocketIO transport behavior.
- `rewrite-in-rust/records/0023-confirm-web-settings-boundary.md:11` confirms this as a JSON/file behavior unit and rejects introducing a Flask route bridge or replacing production `web_server.py`.
- `rewrite-in-rust/fixtures/web_settings_contract.jsonl:1` covers known-section merge, unknown top-level filtering, non-object fallback, missing/malformed load fallback, update success, unknown top-level update ignore, non-object rejection, invalid JSON rejection, reset, and UTF-8 pretty save payload behavior.
- `rewrite-in-rust/bootstrap/check_web_settings_contract.py:68` exercises legacy helpers and Flask test-client route behavior against temporary settings files. It imports `web_server.py`, so it depends on Flask importability and module initialization, but the checked paths do not start the server, open sockets, run model inference, download models, or perform network calls.
- `rewrite-in-rust/rust/crates/v2m-core/src/web_settings.rs:6` uses `serde_json` only; `rewrite-in-rust/rust/crates/v2m-core/Cargo.toml:12` declares `serde_json = "1"`.
- `rewrite-in-rust/rust/crates/v2m-core/src/web_settings.rs:188` consumes the same JSONL fixture table for Rust tests.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_web_settings_contract.py`: pass. The malformed-load fixture emitted the expected legacy warning and completed successfully.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml web_settings`: pass. One `web_settings` fixture-table test passed; unrelated tests were filtered.
- `cargo tree --manifest-path rewrite-in-rust/rust/Cargo.toml -p v2m-core --edges normal`: pass. Normal dependency tree is `v2m-core -> serde_json` plus serde_json transitive dependencies (`itoa`, `memchr`, `serde_core`, `zmij`).

## Residual Risk

The bootstrap checker imports the full `web_server.py` module to reach the legacy helpers and settings routes. That is acceptable for this review because it does not start a real Web server, model runtime, download subprocess, or network operation, but it means the checker still depends on Flask/SocketIO and top-level Web module import side effects remaining lightweight.

## Promotion Note

This dependency/bootstrap review does not block promotion. The unit is ready for the remaining required review roles, but the coordinator should not mark it verified until the required behavior and error-tracing reviews also pass.
