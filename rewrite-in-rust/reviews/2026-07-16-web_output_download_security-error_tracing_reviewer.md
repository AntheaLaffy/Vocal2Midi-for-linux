# web_output_download_security - error_tracing_reviewer

Date: 2026-07-16
Decision: pass-with-followups

## Findings

- Severity: medium
- Location: web_server.py:779
- Issue: Requested-path resolver failures are not structured or fixture-modeled. `_safe_requested_download_path` calls `Path.resolve()` before the route maps denied downloads to the generic 404 response. A symlink loop under the project root therefore bypasses `{"error": "File not found"}` and falls through Flask's 500 path, with a stack trace in server logs. The Rust model is a pure POSIX string fixture model and cannot represent this resolver-error branch.
- Evidence: `web_server.py:768` through `web_server.py:779` validates URL strings and then resolves the requested project path without a local `try`; `web_server.py:814` through `web_server.py:816` only maps `None` authorization results to route-level 404; `web_server.py:955` through `web_server.py:963` handles the escaped exception as a generic 500. `rewrite-in-rust/rust/crates/v2m-core/src/web_output_download.rs:31` through `rewrite-in-rust/rust/crates/v2m-core/src/web_output_download.rs:46` normalizes strings and returns `Option<String>` with no resolver-error state. `rewrite-in-rust/fixtures/web_output_download_security.jsonl:14` covers a successful symlink canonicalization case, but no symlink loop or resolver exception. A temp-root probe using Flask's test client against `/api/download/loop.mid` with `loop.mid -> loop.mid` returned status `500` and emitted a Flask traceback; the focused fixture/bootstrap checks still pass because that branch is outside the current fixture table.
- Required fix: Before any Rust-owned live download route promotion, define the resolver-error policy and add a fixture or record for symlink-loop/path-resolution failures. Prefer keeping the public response redacted while adding an internal structured rejection reason; if preserving legacy 500 behavior, document that choice and its log redaction implications.

- Severity: low
- Location: web_server.py:768
- Issue: Denied download reasons are intentionally collapsed for the public API, but there is no private structured diagnostic surface for a future Rust owner. Empty paths, traversal, backslashes, NUL bytes, Windows drive prefixes, missing files, unregistered files, and canonical output mismatches all become `None` and then `{"error": "File not found"}`. That is good public redaction for this security boundary, but it leaves future bridge/runtime diagnostics unable to distinguish validation, existence, authorization, or canonicalization failures.
- Evidence: `web_server.py:768` through `web_server.py:801` uses `None` for every helper failure; `web_server.py:814` through `web_server.py:816` maps all route-level denials to the same 404 JSON. Rust mirrors this with `Option<String>` at `rewrite-in-rust/rust/crates/v2m-core/src/web_output_download.rs:31` through `rewrite-in-rust/rust/crates/v2m-core/src/web_output_download.rs:87` and a single 404 response at `rewrite-in-rust/rust/crates/v2m-core/src/web_output_download.rs:120` through `rewrite-in-rust/rust/crates/v2m-core/src/web_output_download.rs:126`. Fixtures at `rewrite-in-rust/fixtures/web_output_download_security.jsonl:1` through `rewrite-in-rust/fixtures/web_output_download_security.jsonl:24` prove the public redacted shape, not a private rejection taxonomy.
- Required fix: Keep the public `File not found` shape for compatibility, but add a non-public rejection enum or trace category before promotion to a Rust-owned live route. Categories should be redacted, for example `invalid_path`, `missing_file`, `unregistered_output`, and `canonicalization_error`, without logging raw user-supplied paths by default.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_web_output_download_security.py`: passed.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml web_output_download`: passed; 1 `web_output_download` test passed in `v2m-core`, 59 unrelated tests filtered, and 0 bridge tests ran.
- `uv run pytest tests/test_web_api.py -k download`: passed; 23 selected tests passed and 30 tests were deselected.
- `git diff --check`: passed.
- Direct report-file scan for trailing whitespace and conflict markers: no matches.
- Temp-root resolver probe with a self-referential symlink under a monkeypatched `PROJECT_ROOT`: reproduced the unstructured 500 resolver-error branch without writing to the repository tree.

## Residual Risk

This review covers the error and diagnostic shape of the fixture-backed output
download authorization unit. It does not prove real Flask streaming,
Werkzeug routing beyond the fixture cases, task-manager locking, pipeline output
collection, or live filesystem resolver failures other than the targeted
symlink-loop probe. Runtime ownership remains legacy Python and no Rust bridge
is introduced.

The current public denial behavior is appropriately redacted for a download
authorization route. The remaining risk is private diagnosability and resolver
exception handling if this boundary later becomes Rust-owned or is exposed
beyond the local trusted Web UI assumptions in `docs/web-api.md`.

## Promotion Note

This role does not block coordinator verification for the current legacy-owned,
fixture-backed unit. The coordinator may mark the `error_tracing_reviewer` role
satisfied as `pass-with-followups`, but these findings should block any future
Rust-owned live download route promotion until resolver-error policy and private
structured diagnostics are handled or explicitly recorded as out of scope.
