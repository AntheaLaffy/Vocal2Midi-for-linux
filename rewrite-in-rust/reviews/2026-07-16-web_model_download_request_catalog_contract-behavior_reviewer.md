# web_model_download_request_catalog_contract - behavior_reviewer

Date: 2026-07-16
Decision: fail

## Findings

- Severity: medium
- Location: rewrite-in-rust/rust/crates/v2m-core/src/web_model_download.rs:267
- Issue: `start_route_response` rejects any non-object JSON request body with `400`, but the legacy Flask route treats falsey parsed JSON values as `{}` before applying the missing-model default. In particular, a JSON `null` body currently starts a download for missing models, while the Rust model returns `{"success": false, "error": "models must be a list of model ids"}`.
- Evidence: legacy `web_server.py:651` uses `request.get_json(silent=True) or {}`, then falls through to the missing-model default at `web_server.py:657`. A manual Flask test-client probe with a fake manager returned `200` for request body `null` and captured `selected_models == ["game"]`; the Rust branch at `rewrite-in-rust/rust/crates/v2m-core/src/web_model_download.rs:267` returns the `400` error before defaulting. The existing fixture set covers `{"models": null}` at `rewrite-in-rust/fixtures/web_model_download_request_catalog_contract.jsonl:5`, but does not cover a JSON `null` request body.
- Required fix: Add a fixture for a JSON `null` request body and update the Rust model to preserve the legacy default-to-missing behavior, or explicitly narrow/change the route contract with a separate decision record before promotion.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_web_model_download_request_catalog_contract.py`: pass
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml web_model_download`: pass, 2 tests
- `uv run pytest tests/test_web_api.py::TestModelDownloadAPI -q`: pass, 12 tests
- `uv run pytest tests/test_web_api.py -q`: pass, 53 tests
- Manual Flask test-client probe for `/api/models/download` with request bodies `null`, `[]`, and `["game"]`: `null` and `[]` defaulted to missing models with `200`; truthy non-object `["game"]` produced the legacy `500` path.

## Residual Risk

Catalog/status fields, installed/missing counts, active-task serialization, proxy URL redaction, object-payload start-route defaults, deduplication, force truthiness, validation strings, conflict mapping, status lookup, and stop response mapping are covered by the current fixtures and passed both Python and Rust checks. Non-string/non-object exotic JSON payloads remain weakly specified by the public docs; this report only blocks the documented review scope because `null` default behavior is part of route-level request parity.

The unit does not implement subprocess command construction, proxy environment shaping, output parsing, progress math, active-task locking, SocketIO emission, process termination, real downloads, or archive safety. Those behaviors are still excluded here and assigned to later units in `rewrite-in-rust/manifest.yaml:585` and `rewrite-in-rust/manifest.yaml:606`.

## Promotion Note

This behavior role blocks promotion/state update for `web_model_download_request_catalog_contract` until the JSON `null` request-body parity gap is fixed or intentionally re-scoped.
