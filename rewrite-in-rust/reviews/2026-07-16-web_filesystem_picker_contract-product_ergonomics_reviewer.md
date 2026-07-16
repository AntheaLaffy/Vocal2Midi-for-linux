# web_filesystem_picker_contract - product_ergonomics_reviewer

Date: 2026-07-16
Decision: pass-with-followups

## Findings

- Severity: low
- Location: tests/test_web_api.py:818
- Issue: The product-facing Web API tests cover roots, directory listing, extension filtering, and invalid mode, but they do not exercise typed missing paths, unreadable-directory messages, or file-path fallback through the actual Flask route and picker rendering path. The fixture contract covers these cases, so this does not block the current library fixture unit; it leaves a UI/operator regression gap before any live picker ownership transfer.
- Evidence: Fixture rows cover file-path fallback, missing path, invalid mode, and scandir failure in `rewrite-in-rust/fixtures/web_filesystem_picker_contract.jsonl:12`, `rewrite-in-rust/fixtures/web_filesystem_picker_contract.jsonl:13`, `rewrite-in-rust/fixtures/web_filesystem_picker_contract.jsonl:14`, and `rewrite-in-rust/fixtures/web_filesystem_picker_contract.jsonl:15`. The selected Web tests at `tests/test_web_api.py:818` through `tests/test_web_api.py:875` only assert root presence, directory-only listing, file extension filtering, and invalid mode. The browser displays failed picker reads through `apiCall` and the path-picker empty state at `Vocal2Midi Web.html:1754` and `Vocal2Midi Web.html:2584`.
- Required fix: Before promoting a Rust-owned picker route or browser workflow, add a non-mutating Web/API test for typed missing paths, unreadable/scandir failures, and file-path fallback so the operator-facing message and stale-selection behavior stay proven.

- Severity: low
- Location: docs/web-api.md:7
- Issue: Raw local path disclosure is documented as a local-trust behavior, but it remains an operator assumption rather than an enforced contract. This is acceptable for the current legacy-owned local Web workflow, yet it should stay visible if a future Rust bridge changes transport, bind defaults, or deployment packaging.
- Evidence: The Web API contract states there is no authentication layer and default bind host is `0.0.0.0` at `docs/web-api.md:7`, and the picker endpoint section explicitly says local filesystem paths should not be exposed to untrusted clients at `docs/web-api.md:197`. Runtime defaults still bind from `V2M_WEB_HOST` with `0.0.0.0` fallback at `web_server.py:977`. The picker intentionally returns absolute local paths for roots and entries at `web_server.py:476`, `web_server.py:530`, and `rewrite-in-rust/rust/crates/v2m-core/src/web_filesystem_picker.rs:10`.
- Required fix: Preserve the local-trust warning and bind/deployment guidance in any promotion record or Rust-owned Web transport. Do not redact paths inside this fixture contract without a separate compatibility decision, because the current UI relies on raw path values.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_web_filesystem_picker_contract.py`: passed.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml web_filesystem_picker`: passed; 1 picker test passed, 59 v2m-core tests filtered out, and 0 bridge tests ran.
- `uv run pytest tests/test_web_api.py -k filesystem`: passed; 4 selected tests passed, 49 deselected.
- `git diff --check`: passed.

## Residual Risk

This review is source, fixture, and test based. It did not launch a browser, test Windows drive roots, test real OS permission combinations beyond the synthetic scandir failure, or validate assistive-technology behavior of the modal picker. Browser static assets, Flask route ownership, and live filesystem enumeration remain legacy-owned by design.

## Promotion Note

This product ergonomics role does not block promotion of the fixture-backed `web_filesystem_picker_contract`. The picker contract is adequate for the local Web workflow: root labels, project-relative values, home/absolute/relative handling, directory-first sorting, extension filtering, non-directory fallback, and current error payloads are covered by fixtures or source/tests. Runtime ownership should remain legacy, and coordinator state updates remain separate.
