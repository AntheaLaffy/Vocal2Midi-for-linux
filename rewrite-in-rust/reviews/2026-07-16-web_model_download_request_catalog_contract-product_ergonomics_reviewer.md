# web_model_download_request_catalog_contract - product_ergonomics_reviewer

Date: 2026-07-16
Decision: pass

## Findings

No findings.

The reviewed request/catalog unit preserves the user-visible model download
workflow within its declared boundary:

- The catalog route exposes the fields the browser renders: model ids, names,
  roles/descriptions, target paths, markers, sources, installed flags, install
  counts, and an active task when present (`web_server.py:629`,
  `web_model_download_manager.py:103`, `Vocal2Midi Web.html:2196`,
  `Vocal2Midi Web.html:2237`, `Vocal2Midi Web.html:2265`). The fixture locks
  those catalog fields and installed/missing counts
  (`rewrite-in-rust/fixtures/web_model_download_request_catalog_contract.jsonl:1`).
- Default missing-model selection is preserved for omitted `models`, null
  request bodies, falsey top-level JSON arrays, and `models: null`
  (`web_server.py:651`, `web_server.py:657`,
  `rewrite-in-rust/fixtures/web_model_download_request_catalog_contract.jsonl:4`,
  `rewrite-in-rust/fixtures/web_model_download_request_catalog_contract.jsonl:5`,
  `rewrite-in-rust/fixtures/web_model_download_request_catalog_contract.jsonl:6`,
  `rewrite-in-rust/fixtures/web_model_download_request_catalog_contract.jsonl:7`,
  `rewrite-in-rust/rust/crates/v2m-core/src/web_model_download.rs:267`,
  `rewrite-in-rust/rust/crates/v2m-core/src/web_model_download.rs:416`).
- Explicit start requests keep UI order, deduplicate model ids, preserve the
  Python truthiness of `force`, and map active-task conflicts to `409`
  (`web_server.py:672`, `web_server.py:692`,
  `rewrite-in-rust/fixtures/web_model_download_request_catalog_contract.jsonl:3`,
  `rewrite-in-rust/fixtures/web_model_download_request_catalog_contract.jsonl:8`,
  `rewrite-in-rust/fixtures/web_model_download_request_catalog_contract.jsonl:17`).
- Proxy options and validation are aligned across docs, UI, legacy route, and
  Rust model: `system`, `manual`, and `none` are the accepted modes; manual
  mode requires a URL with a scheme; proxy credentials are redacted in
  serialized task state and in the browser's pre-start log line
  (`docs/web-api.md:266`, `docs/web-api.md:267`,
  `web_model_download_manager.py:517`, `web_model_download_manager.py:535`,
  `Vocal2Midi Web.html:1344`, `Vocal2Midi Web.html:1951`,
  `Vocal2Midi Web.html:2348`,
  `rewrite-in-rust/rust/crates/v2m-core/src/web_model_download.rs:187`).
- Status and stop response mapping match the documented recovery/cancellation
  workflow: missing status is `404`, stop requires a task id, missing tasks are
  `404`, non-stoppable tasks are `400`, and successful stop returns
  `status: stopping` (`docs/web-api.md:287`, `docs/web-api.md:310`,
  `web_server.py:715`, `web_server.py:731`,
  `rewrite-in-rust/fixtures/web_model_download_request_catalog_contract.jsonl:18`,
  `rewrite-in-rust/fixtures/web_model_download_request_catalog_contract.jsonl:23`).
- The browser UI consumes the same route contract: it loads `/models/status`,
  renders counts and rows, disables start controls while an active task is
  pending/running/stopping, starts downloads with selected ids and proxy/source
  options, posts `/models/download/stop`, and reconnects to active task rooms
  on WebSocket reconnect (`Vocal2Midi Web.html:1606`,
  `Vocal2Midi Web.html:2196`, `Vocal2Midi Web.html:2306`,
  `Vocal2Midi Web.html:2334`, `Vocal2Midi Web.html:2394`).
- The pre-report targeted working-tree diff did not touch this unit's source,
  fixtures, docs, Web UI, or tests; the dirty files were other rewrite units
  plus manifest state updates.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_web_model_download_request_catalog_contract.py`: passed
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml web_model_download`: passed, 7 selected tests
- `uv run pytest tests/test_web_api.py::TestModelDownloadAPI -q`: passed, 12 tests
- `uv run pytest tests/test_web_api.py -k ModelDownload -q`: passed, 19 selected tests
- `git diff --check`: passed
- `git diff --name-only -- web_server.py web_model_download_manager.py download_models.py docs/web-api.md "Vocal2Midi Web.html" tests/test_web_api.py rewrite-in-rust/fixtures/web_model_download_request_catalog_contract.jsonl rewrite-in-rust/bootstrap/check_web_model_download_request_catalog_contract.py rewrite-in-rust/rust/crates/v2m-core/src/web_model_download.rs rewrite-in-rust/dependencies/web_model_download_request_catalog_contract.yaml rewrite-in-rust/bootstrap/web_model_download_request_catalog_contract.md rewrite-in-rust/records/0029-split-web-model-download-contract.md`: no unit diff

## Residual Risk

This review did not prove subprocess command construction, proxy environment
application inside the child process, stdout/progress parsing, real SocketIO
delivery, task lifecycle locking, process-tree termination, network downloads,
package installation, archive extraction, or marker safety. Those behaviors are
explicitly split into adjacent model-download and `download_models.py` units
(`rewrite-in-rust/records/0029-split-web-model-download-contract.md:37`,
`rewrite-in-rust/dependencies/web_model_download_request_catalog_contract.yaml:43`).

The public docs describe the normal object-body API shape and the all-missing
default, but do not enumerate every exotic falsey non-object JSON body. That is
acceptable for product ergonomics because the browser always sends an object
body and the behavior gate now covers the known legacy falsey-body paths.

## Promotion Note

This product ergonomics role does not block coordinator verification for
`web_model_download_request_catalog_contract` while runtime ownership remains
legacy Python. The coordinator may mark this role satisfied. Full unit
verification still depends on the manifest-required roles being present or
explicitly waived by the coordinator; no production runtime ownership changes
are approved by this report.
