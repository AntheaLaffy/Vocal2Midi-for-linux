# web_filesystem_picker_contract - error_tracing_reviewer

Date: 2026-07-16
Decision: pass-with-followups

## Findings

- Severity: low
- Location: web_server.py:456
- Issue: Resolver failure behavior is source-inspected but not fixture-proven. The legacy route calls `_resolve_picker_path` before the missing-path check, and `_resolve_picker_path` returns `expanded.resolve()` without a local error wrapper. The Rust model normalizes POSIX strings and has no way to represent resolver `OSError`, symlink-loop, or platform resolution failures.
- Evidence: `web_server.py:456` performs the unresolved exception boundary; `web_server.py:558` calls it before `exists()` status mapping; Rust `resolve_picker_path` at `rewrite-in-rust/rust/crates/v2m-core/src/web_filesystem_picker.rs:62` through `rewrite-in-rust/rust/crates/v2m-core/src/web_filesystem_picker.rs:75` is pure string normalization; current resolver fixtures cover empty, whitespace, relative, home, and absolute-project paths at `rewrite-in-rust/fixtures/web_filesystem_picker_contract.jsonl:1` through `rewrite-in-rust/fixtures/web_filesystem_picker_contract.jsonl:5`, but no resolver error.
- Required fix: Before any Rust-owned live filesystem route promotion, add a fixture/record for resolver error behavior or explicitly document that legacy global 500 behavior is outside this picker contract.

- Severity: low
- Location: web_server.py:518
- Issue: Symlink and follow behavior remains implicit. Legacy resolves user-supplied picker paths through `Path.resolve()`, but entry classification uses `follow_symlinks=False`, so symlink inputs and symlink children can take different paths through the route. The Rust fixture model has only `directory` and `file` entry specs and cannot express a symlink child or resolved target distinction.
- Evidence: `_resolve_picker_path` follows resolved paths at `web_server.py:447` through `web_server.py:456`; `_filesystem_entry` probes entries with `follow_symlinks=False` at `web_server.py:518` and `web_server.py:519`; included entries expose `path.resolve()` at `web_server.py:533`. Rust entry modeling at `rewrite-in-rust/rust/crates/v2m-core/src/web_filesystem_picker.rs:227` through `rewrite-in-rust/rust/crates/v2m-core/src/web_filesystem_picker.rs:250` only handles fixture `directory`/`file` types.
- Required fix: Add symlink input/child fixtures or a compatibility record before using this unit as evidence for live filesystem ownership. This does not block the current legacy-owned fixture contract.

- Severity: low
- Location: web_server.py:578
- Issue: Unreadable-directory errors preserve raw `OSError` text. That matches the legacy route and current fixture shape, but it can include OS-specific path or permission details. The Web API docs already warn that filesystem picker endpoints expose local filesystem paths, so this is a promotion-hardening follow-up rather than a parity blocker.
- Evidence: `web_server.py:575` through `web_server.py:579` returns `Cannot read directory: {e}` with status 400; docs warn these endpoints expose local paths at `docs/web-api.md:199` and `docs/web-api.md:200`; fixture `list_scandir_error_returns_400` checks the raw diagnostic substring at `rewrite-in-rust/fixtures/web_filesystem_picker_contract.jsonl:15`; Rust mirrors the same message at `rewrite-in-rust/rust/crates/v2m-core/src/web_filesystem_picker.rs:170` through `rewrite-in-rust/rust/crates/v2m-core/src/web_filesystem_picker.rs:182`.
- Required fix: If the route is ever Rust-owned or exposed beyond local trusted use, make a recorded decision to keep raw legacy diagnostics or introduce a separate hardening unit with redacted errors.

- Severity: low
- Location: web_server.py:520
- Issue: Filtered-entry diagnosability is intentionally minimal. Entry probe errors, non-directory/non-file entries, symlink children, and extension-filtered files are all omitted without counts or per-entry diagnostics. Rust mirrors the silent filtering shape for fixture entries, but the fixture harness cannot distinguish an unreadable entry from an intentionally filtered one.
- Evidence: `_filesystem_entry` returns `None` on `OSError` at `web_server.py:516` through `web_server.py:521`, returns `None` for excluded entry types at `web_server.py:523` through `web_server.py:524`, and returns `None` for extension misses at `web_server.py:527` through `web_server.py:528`. The Python fake entry only models boolean directory/file type at `rewrite-in-rust/bootstrap/check_web_filesystem_picker_contract.py:95` through `rewrite-in-rust/bootstrap/check_web_filesystem_picker_contract.py:105`; Rust `picker_entry` silently filters at `rewrite-in-rust/rust/crates/v2m-core/src/web_filesystem_picker.rs:233` through `rewrite-in-rust/rust/crates/v2m-core/src/web_filesystem_picker.rs:243`.
- Required fix: Keep this as a documented legacy behavior for the current unit. Add richer skipped-entry diagnostics only in a separate product/error-design unit, because changing the response shape would not be parity-preserving.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_web_filesystem_picker_contract.py`: passed.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml web_filesystem_picker`: passed; 1 picker test passed in `v2m-core`, 59 filtered out, and 0 bridge tests ran.
- `UV_CACHE_DIR=/tmp/v2m-uv-cache uv run pytest tests/test_web_api.py::TestFilesystemBrowserAPI`: passed; 4 tests passed.
- `git diff --check`: passed.
- Targeted scans over `web_server.py`, `rewrite-in-rust/rust/crates/v2m-core/src/web_filesystem_picker.rs`, the checker, fixtures, `docs/web-api.md`, and `tests/test_web_api.py`: completed for filesystem status codes, raw error text, path exposure, `follow_symlinks`, and filtered-entry paths.

## Residual Risk

The current evidence is sufficient for the legacy-owned, fixture-backed picker contract. It does not prove Windows root behavior, live symlink edge cases, resolver exceptions, actual permission-denied OS messages, or a hardened/redacted HTTP surface. Output download authorization, settings, SocketIO, and model runtime behavior were intentionally out of scope.

## Promotion Note

This error-tracing role does not block continuing coordinator review of the fixture-backed unit, but it does not approve Rust runtime ownership. Keep `web_server.py` as the runtime owner until live filesystem error and symlink behavior are either fixture-proven or explicitly recorded as out of scope for promotion.
