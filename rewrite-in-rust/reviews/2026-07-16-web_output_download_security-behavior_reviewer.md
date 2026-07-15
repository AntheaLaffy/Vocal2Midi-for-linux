# web_output_download_security - behavior_reviewer

Date: 2026-07-16
Decision: pass

## Findings

No findings.

The reviewed unit preserves the scoped legacy behavior:

- `web_server.py:765` rejects Windows drive prefixes with either slash style.
- `web_server.py:768` rejects empty paths, NUL bytes, backslashes, absolute POSIX paths, and any `..` path part before resolving under `PROJECT_ROOT`.
- `web_server.py:782` authorizes only existing files whose canonical path exactly matches a registered `Task.output_files` entry, with relative registered outputs resolved under `PROJECT_ROOT`.
- `web_task_manager.py:239` through `web_task_manager.py:245` shows `Task.output_files` is a list of string file paths collected from output extensions.
- `web_server.py:804` returns route-level `{"error": "File not found"}` 404 responses for failed download authorization and successful `send_file(..., download_name=safe_path.name)` responses for authorized files.
- `rewrite-in-rust/fixtures/web_output_download_security.jsonl` covers helper sanitization, route percent-decoding, app-level 404 for unmatched download routes, registered-output-only authorization, relative and absolute registered outputs, canonical parent paths, symlink canonicalization, nonexistent registered outputs, unregistered neighbor files, and successful response body/download filename.
- `rewrite-in-rust/rust/crates/v2m-core/src/web_output_download.rs` models the same fixture-backed path validation, authorization, route 404 shape, app-level 404 shape, file body, and download filename behavior.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_web_output_download_security.py`: passed
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml web_output_download`: passed, 1 test passed

## Residual Risk

The Rust implementation remains a narrow POSIX fixture model. It does not replace Flask routing, Werkzeug decoding internals, real `send_file` streaming, task-manager locking, or host filesystem canonicalization beyond the synthetic symlink cases in the fixture table. Those are acceptable residual risks for this behavior unit because legacy Python remains the runtime owner and the boundary record explicitly excludes Flask/server replacement.

## Promotion Note

This behavior review does not block coordinator verification for `web_output_download_security`.
