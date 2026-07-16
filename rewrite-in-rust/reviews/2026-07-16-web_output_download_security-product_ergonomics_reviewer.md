# web_output_download_security - product_ergonomics_reviewer

Date: 2026-07-16
Decision: pass-with-followups

## Findings

- Severity: medium
- Location: Vocal2Midi Web.html:2104
- Issue: The browser workflow reports completed output filenames but does not expose `/api/download/<path>` links or buttons for them. The backend route and API tests prove the endpoint can serve registered outputs, but the user-facing Web UI currently leaves download as an API-only/manual path workflow.
- Evidence: `Vocal2Midi Web.html:2104` through `Vocal2Midi Web.html:2108` iterates `data.result.files` and logs only each basename. `rg -n "/api/download|api/download|download/" 'Vocal2Midi Web.html' tests/test_web_api.py docs/web-api.md web_server.py rewrite-in-rust/fixtures/web_output_download_security.jsonl rewrite-in-rust/bootstrap/web_output_download_security.md rewrite-in-rust/rust/crates/v2m-core/src/web_output_download.rs` found no `/api/download` use in `Vocal2Midi Web.html`; only backend/docs/fixture/test references exist.
- Required fix: Add a separate product/UI follow-up to render downloadable links for registered outputs, or explicitly document that output downloads are Web API only. Link construction must preserve the registered-output-only policy and URL-encode a project-relative path.

- Severity: medium
- Location: web_server.py:768
- Issue: Registered outputs outside `PROJECT_ROOT` are not downloadable through the current route, even though the Web settings allow users to choose an arbitrary save directory. This is consistent with the unit's project-relative URL security policy, but it is a user-visible recovery limitation that is not called out in the Web API docs.
- Evidence: `_safe_requested_download_path` rejects absolute URL paths and resolves accepted paths under `PROJECT_ROOT` at `web_server.py:768` through `web_server.py:779`; output collection registers whatever `config.output_dir` produced at `web_task_manager.py:239` through `web_task_manager.py:245`; the UI sends the free-form save directory at `Vocal2Midi Web.html:1969` through `Vocal2Midi Web.html:1971` and `Vocal2Midi Web.html:2063`. Manual Flask probe with a registered temp output outside the repo returned `404` for both an encoded absolute URL (`{"success": false, "error": "Resource not found"}`) and a guessed project-relative URL (`{"error": "File not found"}`).
- Required fix: Before any UI-level download promise or Rust ownership promotion, either document that `/api/download/<path>` only addresses registered outputs under the project root, constrain/download-link only project-root outputs, or design a task/file-id download route that can safely serve registered absolute outputs without accepting arbitrary absolute paths.

- Severity: low
- Location: rewrite-in-rust/fixtures/web_output_download_security.jsonl:11
- Issue: Filename/body coverage proves successful body bytes and simple ASCII basename behavior, but it does not cover browser-visible `Content-Disposition` behavior for spaces, quotes, or non-ASCII output names. This matters for a local UI where uploaded audio and generated outputs may have user-controlled names.
- Evidence: Successful fixture cases at `rewrite-in-rust/fixtures/web_output_download_security.jsonl:11` through `rewrite-in-rust/fixtures/web_output_download_security.jsonl:14` use simple ASCII filenames; the Python checker extracts `filename=` from `Content-Disposition` at `rewrite-in-rust/bootstrap/check_web_output_download_security.py:110` through `rewrite-in-rust/bootstrap/check_web_output_download_security.py:115`; the Rust model returns only `file_name(&path)` at `rewrite-in-rust/rust/crates/v2m-core/src/web_output_download.rs:134` through `rewrite-in-rust/rust/crates/v2m-core/src/web_output_download.rs:140`.
- Required fix: Add fixture/test cases for spaces and non-ASCII output filenames before claiming browser filename fidelity beyond the simple basename contract.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_web_output_download_security.py`: passed.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml web_output_download`: passed; 1 `web_output_download` test passed in `v2m-core`, with unrelated tests filtered.
- `uv run pytest tests/test_web_api.py::TestDownloadAPI`: passed; 4 tests passed.
- `git diff --check`: passed.
- Manual Flask test-client probe for a registered output outside `PROJECT_ROOT`: returned 404 for an encoded absolute URL and 404 for a guessed project-relative URL, confirming the project-relative limitation.
- `git diff --name-only -- rewrite-in-rust/rust/crates/v2m-core/src/web_output_download.rs rewrite-in-rust/bootstrap/check_web_output_download_security.py rewrite-in-rust/bootstrap/web_output_download_security.md rewrite-in-rust/fixtures/web_output_download_security.jsonl rewrite-in-rust/dependencies/web_output_download_security.yaml web_server.py web_task_manager.py docs/web-api.md tests/test_web_api.py 'Vocal2Midi Web.html'`: no relevant unit files are currently modified.

## Residual Risk

This role reviewed product ergonomics only. It did not re-review behavior parity, dependency/bootstrap adequacy, error tracing, or Rust style. The current Web server remains a trusted-local tool with no authentication, default `0.0.0.0` bind, and permissive CORS as documented in `docs/web-api.md:7` through `docs/web-api.md:14`; this review accepts those local-trust assumptions for the legacy-owned runtime and does not approve deployment to untrusted networks.

The Rust implementation is still a fixture-backed library model. It does not replace Flask routing, `send_file` streaming, task-manager locking, browser UI rendering, or output collection.

## Promotion Note

This product ergonomics role does not block coordinator verification while runtime ownership remains legacy Python and the unit remains scoped to backend registered-output authorization. The coordinator may mark this role satisfied with the follow-ups above tracked outside this review; do not mark the unit promoted or Rust-owned from this report alone.
