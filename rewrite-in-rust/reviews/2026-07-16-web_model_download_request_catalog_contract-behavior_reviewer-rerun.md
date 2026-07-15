# web_model_download_request_catalog_contract - behavior_reviewer rerun

Date: 2026-07-16
Decision: pass

## Findings

No findings.

The previous medium finding is resolved. The fixture table now covers the whole
request body being JSON `null` at
`rewrite-in-rust/fixtures/web_model_download_request_catalog_contract.jsonl:5`
and a top-level empty JSON array at
`rewrite-in-rust/fixtures/web_model_download_request_catalog_contract.jsonl:6`.
The Python checker sends the raw `null` body instead of wrapping it in an object
at `rewrite-in-rust/bootstrap/check_web_model_download_request_catalog_contract.py:195`
and still exercises ordinary `json=` route bodies at
`rewrite-in-rust/bootstrap/check_web_model_download_request_catalog_contract.py:203`.

The Rust route model now matches the legacy falsey-body behavior: non-object
falsey JSON values are normalized to an empty object at
`rewrite-in-rust/rust/crates/v2m-core/src/web_model_download.rs:267`, so they
fall through to missing-model default selection at
`rewrite-in-rust/rust/crates/v2m-core/src/web_model_download.rs:416`. This
preserves the legacy Flask route, where `request.get_json(silent=True) or {}`
normalizes falsey parsed bodies at `web_server.py:651` before defaulting missing
models at `web_server.py:657`.

The remainder of the request/catalog behavior stays inside the confirmed unit
boundary: catalog/status serialization, active-task serialization with proxy
redaction, start request validation/defaults/deduplication/conflict mapping,
status lookup mapping, and stop response mapping. Subprocess planning,
SocketIO/progress events, active-task locking, termination, real downloads, and
archive safety remain outside this unit per
`rewrite-in-rust/records/0029-split-web-model-download-contract.md:37`.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_web_model_download_request_catalog_contract.py`: pass
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml web_model_download`: pass, 2 tests
- `uv run pytest tests/test_web_api.py::TestModelDownloadAPI -q`: pass, 12 tests
- `git diff --check -- rewrite-in-rust/fixtures/web_model_download_request_catalog_contract.jsonl rewrite-in-rust/bootstrap/check_web_model_download_request_catalog_contract.py rewrite-in-rust/rust/crates/v2m-core/src/web_model_download.rs rewrite-in-rust/manifest.yaml rewrite-in-rust/records/0029-split-web-model-download-contract.md rewrite-in-rust/dependencies/web_model_download_request_catalog_contract.yaml rewrite-in-rust/bootstrap/web_model_download_request_catalog_contract.md`: pass
- Manual Flask test-client probe for `/api/models/download` with raw request bodies `null` and `[]`: pass, both returned `200` and selected `["game", "qwen"]` from fake missing-model status.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml`: pass, 56 tests
- `uv run pytest tests/test_web_api.py -q`: pass, 53 tests

## Residual Risk

The Web API docs document normal object bodies and the default "all missing
models" behavior, but do not specify every exotic non-object JSON body. The
reviewed implementation now preserves the legacy falsey-body route behavior for
the previously failing `null` case and the added empty-array case. Truthy
non-object bodies remain rejected by the Rust route model as invalid model-list
requests; this is acceptable for this fixture-bound unit because the documented
public request shape is an object body and the known falsey default path is now
covered.

This review did not evaluate process command construction, proxy environment
shaping, stdout/progress parsing, SocketIO emissions, lifecycle locking,
termination behavior, network downloads, or archive safety. Those behaviors are
assigned to later split units.

## Promotion Note

This behavior role no longer blocks coordinator state update for
`web_model_download_request_catalog_contract`. Required non-behavior roles for
the unit still need their own reports before broader promotion decisions.
