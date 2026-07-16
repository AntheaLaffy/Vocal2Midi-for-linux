# web_settings_contract - behavior_reviewer rerun

Date: 2026-07-16
Decision: pass

## Findings

No findings.

The prior top-level non-object update mismatch is resolved. Legacy Python parses
`PUT /api/settings` with `request.get_json(silent=True)`, treats only parse
failure or JSON `null` as `400 Invalid JSON in request body`, then iterates
known sections over the parsed value before saving or falling into the route's
`500 Failed to update settings: ...` handler. The updated fixture table now
captures the important legacy branches:

- `rewrite-in-rust/fixtures/web_settings_contract.jsonl:11` proves top-level
  `[]` is a no-op update that persists defaults with `200`.
- `rewrite-in-rust/fixtures/web_settings_contract.jsonl:12` proves a top-level
  string with no known section is also a no-op persisted `200`.
- `rewrite-in-rust/fixtures/web_settings_contract.jsonl:13` proves a top-level
  array containing a known section hits the legacy `500` path.
- `rewrite-in-rust/fixtures/web_settings_contract.jsonl:14` proves a top-level
  JSON number hits the legacy `500` path.

The Python checker now asserts successful raw-update messages, returned settings
subsets, saved-file presence, and trailing newline for `update_raw` success
cases at `rewrite-in-rust/bootstrap/check_web_settings_contract.py:116`. The
Rust implementation delegates non-object update bodies through
`update_non_object_settings` at
`rewrite-in-rust/rust/crates/v2m-core/src/web_settings.rs:117`, including
legacy `400`, no-op persisted `200`, and `500` branches at
`rewrite-in-rust/rust/crates/v2m-core/src/web_settings.rs:154`.

Existing settings merge/load/update/reset/save parity still holds for the
current fixture contract. Representative evidence remains in the same fixture
table for partial known-section merge, unknown top-level filtering, persisted
non-object load fallback, missing/malformed load fallback, known-section
non-object rejection, reset, and UTF-8 pretty save payload behavior.

## Checks

- `env PYTHONDONTWRITEBYTECODE=1 UV_CACHE_DIR=/tmp/v2m-uv-cache uv run python rewrite-in-rust/bootstrap/check_web_settings_contract.py`: pass. The malformed-load fixture emitted the expected legacy warning and the script completed.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml web_settings`: pass. One `web_settings` fixture-table test passed; unrelated tests were filtered.
- `cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check`: pass.
- `env PYTHONDONTWRITEBYTECODE=1 UV_CACHE_DIR=/tmp/v2m-uv-cache uv run pytest tests/test_web_api.py -k "settings or invalid_json"`: pass, 6 selected tests.
- `git diff --check`: pass.

## Residual Risk

This behavior review remains scoped to the legacy-owned settings JSON/file
contract. Additional top-level non-object variants such as JSON booleans,
`null`, floats, and strings containing known section names were source-inspected
through the same Rust/Python branches but are not each explicit fixture rows.
Save-failure diagnostics and reset persistence failures are covered by the
separate error-tracing report, not by this behavior rerun.

## Promotion Note

This behavior rerun no longer blocks coordinator use as passing behavior
evidence for `web_settings_contract`. Runtime ownership remains `legacy`; this
review did not edit production code, fixtures, bootstrap scripts, Rust source,
or the manifest.
