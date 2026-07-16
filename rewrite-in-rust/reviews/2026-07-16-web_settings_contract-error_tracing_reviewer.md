# web_settings_contract - error_tracing_reviewer

Date: 2026-07-16
Decision: pass-with-followups

## Findings

- Severity: medium
- Location: web_server.py:400; web_server.py:411; web_server.py:419; web_server.py:433; web_server.py:436; rewrite-in-rust/rust/crates/v2m-core/src/web_settings.rs:150; rewrite-in-rust/rust/crates/v2m-core/src/web_settings.rs:162; rewrite-in-rust/fixtures/web_settings_contract.jsonl:6
- Issue: Save/persistence failure behavior is only source-inspected, not fixture-modeled, and reset has a weaker structured-error surface than update. `PUT /api/settings` catches `_save_settings_to_disk` failures and returns JSON `500` as `Failed to update settings: ...`, but it mutates `current_settings` before the save call. `POST /api/settings/reset` deep-copies defaults and calls `_save_settings_to_disk` without a route-local `try`, so a save failure falls through to the global Flask 500 handler instead of a settings-specific error. The Rust model always returns a saved payload for update/reset and cannot represent write, replace, or directory creation failures.
- Evidence: `docs/web-api.md:186` documents `500` when settings cannot be saved; `web_server.py:411` calls `_save_settings_to_disk(current_settings)` after in-memory mutation; `web_server.py:419` wraps update exceptions; `web_server.py:436` has no matching reset exception wrapper; `web_server.py:955` global 500 handler returns `Internal server error` plus `details`; `rewrite-in-rust/rust/crates/v2m-core/src/web_settings.rs:150` and `:164` only produce a pretty JSON payload; current fixtures cover successful persistence but no save-error row.
- Required fix: Before any Rust-owned settings persistence bridge, add fixture/test coverage for directory creation, temp write, and replace failures for both update and reset, and decide whether failed saves roll back in-memory settings. If reset keeps the current legacy behavior, document that it uses the global 500 shape.

- Severity: low
- Location: web_server.py:137; web_server.py:143; web_server.py:145; rewrite-in-rust/rust/crates/v2m-core/src/web_settings.rs:98; rewrite-in-rust/fixtures/web_settings_contract.jsonl:2; rewrite-in-rust/fixtures/web_settings_contract.jsonl:3; rewrite-in-rust/fixtures/web_settings_contract.jsonl:4
- Issue: Load fallback diagnostics are partly implicit. Missing settings files fall back silently; malformed JSON logs a warning containing the raw settings file path and parser error; non-object JSON also falls back silently through `_merge_settings`. The fixture table proves missing and malformed load fallback, and proves non-object merge fallback, but it does not directly include a persisted non-object load case or assert the warning text/path exposure.
- Evidence: `web_server.py:139` returns defaults for a missing file without logging; `web_server.py:145` prints `[Warning] Failed to load web settings from {SETTINGS_FILE}: {e}` for `OSError`/`ValueError`; `web_server.py:147` passes JSON scalar/object payloads to `_merge_settings`; `rewrite-in-rust/rust/crates/v2m-core/src/web_settings.rs:103` returns defaults on parse errors and `:104` merges parsed values; fixture rows 3 and 4 cover missing/malformed load, while row 2 is a merge-only non-object case.
- Required fix: Add a persisted non-object load fixture and, if the warning is part of the durable diagnostic contract, capture/assert malformed-load stdout. Otherwise document that missing and non-object settings files intentionally have no diagnostic.

- Severity: low
- Location: web_server.py:68; web_server.py:373; web_server.py:413; rewrite-in-rust/fixtures/web_settings_contract.jsonl:6; rewrite-in-rust/fixtures/web_settings_contract.jsonl:11; tests/test_web_api.py:276; tests/test_web_api.py:296; docs/web-api.md:164
- Issue: Settings responses and saved payloads intentionally expose raw local model paths, `pipeline.save_dir`, and `downloads.proxy_url`; this matches the current Web UI contract but is not called out as a local-trust/privacy boundary for the settings endpoint. A manual proxy URL can include hostnames or credentials, and model/output paths can reveal local filesystem layout.
- Evidence: `GET /api/settings` expands `current_settings` directly into the response at `web_server.py:373`; successful PUT returns `settings: current_settings` at `web_server.py:413`; fixture row 6 preserves and returns `http://127.0.0.1:7890`; fixture row 11 asserts raw UTF-8 path/model payload contents; `tests/test_web_api.py:300` asserts saved `proxy_url` exactly. The docs describe the settings endpoint and save path at `docs/web-api.md:164`, but the explicit "do not expose to untrusted clients" warning appears for filesystem picker endpoints, not settings.
- Required fix: Before public/non-local Web deployment or Rust promotion that changes transport ownership, document settings as raw local-trust data or introduce a redaction policy for proxy URLs and filesystem paths. Do not redact in the current fixture contract without a new compatibility decision.

## Checks

- `env PYTHONDONTWRITEBYTECODE=1 UV_CACHE_DIR=/tmp/v2m-uv-cache uv run python rewrite-in-rust/bootstrap/check_web_settings_contract.py`: pass. The malformed-load fixture emitted the expected legacy warning and the script completed successfully.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml web_settings`: pass. One `web_settings` fixture-table test passed; unrelated tests were filtered.
- `env PYTHONDONTWRITEBYTECODE=1 UV_CACHE_DIR=/tmp/v2m-uv-cache uv run pytest -p no:cacheprovider tests/test_web_api.py -k "settings or invalid_json"`: pass, 6 selected tests.
- `rg` scans over `web_server.py`, `rewrite-in-rust/bootstrap/check_web_settings_contract.py`, `rewrite-in-rust/rust/crates/v2m-core/src/web_settings.rs`, `rewrite-in-rust/fixtures/web_settings_contract.jsonl`, `docs/web-api.md`, and `tests/test_web_api.py` for JSON/file error handling, settings routes, paths, proxy URLs, and logging: completed.
- `git diff --check`: pass.

## Residual Risk

The current contract is a legacy-owned settings JSON/file seam, not a Rust-owned filesystem writer or Flask route replacement. Save-failure, reset-failure, and diagnostic-redaction behavior remain source-inspected rather than fixture-proven. Pipeline-start merge, filesystem picker, output download, SocketIO, model download execution, and model runtime were intentionally out of scope.

## Promotion Note

This error-tracing review does not block retaining the current legacy-owned fixture contract, but it should block any Rust-owned settings persistence or public transport promotion until persistence failure and raw-settings exposure policy are explicitly tested or documented. Runtime ownership remains `legacy`; the manifest was not edited.
