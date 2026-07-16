# web_settings_contract - behavior_reviewer

Date: 2026-07-16
Decision: fail

## Findings

- Severity: medium
- Location: `web_server.py:391`, `web_server.py:394`, `web_server.py:402`, `web_server.py:411`, `rewrite-in-rust/rust/crates/v2m-core/src/web_settings.rs:115`
- Issue: Rust rejects every top-level non-object update body as `400 Invalid JSON in request body`, but legacy Python does not. In `web_server.py`, `request.get_json(silent=True)` only maps parse failure or JSON `null` to `None`; valid JSON arrays and strings then pass the `data is None` guard, skip all known sections, save the current settings, and return `200`. A valid JSON number raises during `section in data` and is converted to the legacy `500 Failed to update settings...` response. Rust's `update_settings` returns the same 400 error for all non-object `Value`s before section iteration.
- Evidence: A Flask test-client probe against a temporary `SETTINGS_FILE` produced `[] -> 200 saved=True`, `{}` -> `200 saved=True`, `'abc' -> 200 saved=True`, `0 -> 500 saved=False`, and `None -> 400 saved=False`. The fixture table only covers a known-section non-object request and syntactically invalid raw JSON at `rewrite-in-rust/fixtures/web_settings_contract.jsonl:8` and `rewrite-in-rust/fixtures/web_settings_contract.jsonl:9`; it does not cover valid top-level non-object JSON. Rust hard-codes the stricter top-level object requirement at `rewrite-in-rust/rust/crates/v2m-core/src/web_settings.rs:117`.
- Required fix: Add parity fixtures for valid top-level non-object update bodies and either mirror the legacy behavior in Rust or record a deliberate contract change that tightens malformed request handling before rerunning the behavior gate.

## Checks

- `env PYTHONDONTWRITEBYTECODE=1 UV_CACHE_DIR=/tmp/v2m-uv-cache uv run python rewrite-in-rust/bootstrap/check_web_settings_contract.py`: pass. The malformed-load fixture emitted the expected warning and the checker completed.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml web_settings`: pass. One `web_settings` fixture-table test passed; unrelated tests were filtered.
- `env PYTHONDONTWRITEBYTECODE=1 UV_CACHE_DIR=/tmp/v2m-uv-cache uv run pytest -p no:cacheprovider tests/test_web_api.py`: pass. 53 tests passed.
- `git diff --check`: pass.
- `env PYTHONDONTWRITEBYTECODE=1 UV_CACHE_DIR=/tmp/v2m-uv-cache uv run python -c '<Flask test-client top-level non-object probe>'`: demonstrated the top-level non-object mismatch above.

## Residual Risk

Full `DEFAULT_SETTINGS` literal parity was source-inspected between `web_server.py:68` and `rewrite-in-rust/rust/crates/v2m-core/src/web_settings.rs:20`; the fixture table asserts representative fields rather than full-object equality. Exact serialized key order and atomic temp-file replacement are also not proven by the Rust payload-only model. These are secondary to the failing top-level non-object update parity.

## Promotion Note

This behavior review blocks promotion and should not be used as passing coordinator evidence. Runtime ownership remains legacy Python, and the manifest should not be marked verified for this unit until the mismatch is fixed or explicitly re-scoped and the behavior gate is rerun.
