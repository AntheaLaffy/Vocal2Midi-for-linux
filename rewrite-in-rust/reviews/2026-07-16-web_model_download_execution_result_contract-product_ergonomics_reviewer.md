# web_model_download_execution_result_contract - product_ergonomics_reviewer

Date: 2026-07-16
Decision: pass

## Findings

No findings.

The reviewed unit preserves the Web UI-facing execution-result workflow within its declared boundary:

- Startup logs remain readable and ordered: the legacy code emits "准备下载模型...", then the concrete `download_models.py` command, then progress `2` / `starting` before process handoff (`web_model_download_manager.py:252`, `rewrite-in-rust/fixtures/web_model_download_execution_result_contract.jsonl:1`, `rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_execution.rs:116`).
- Terminal states are covered for success, nonzero failure, cancellation, and spawn exception, including user-visible final log levels/messages and final status payloads (`web_model_download_manager.py:273`, `web_model_download_manager.py:288`, `web_model_download_manager.py:303`, `rewrite-in-rust/fixtures/web_model_download_execution_result_contract.jsonl:1`, `rewrite-in-rust/fixtures/web_model_download_execution_result_contract.jsonl:2`, `rewrite-in-rust/fixtures/web_model_download_execution_result_contract.jsonl:3`, `rewrite-in-rust/fixtures/web_model_download_execution_result_contract.jsonl:4`).
- Progress and status payloads keep the documented Web API shape: model-download SocketIO events are `log`, `progress`, and `status_change`, and status includes stable task fields such as `task_id`, `task_type`, `status`, `progress`, `stage`, model selection, proxy settings, timestamps, error, returncode, and logs (`docs/web-api.md:287`, `docs/web-api.md:345`, `web_model_download_manager.py:232`, `rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_execution.rs:263`, `rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_execution.rs:278`, `rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_execution.rs:286`).
- Proxy credentials remain redacted in `status_change` payloads via the same split rule as legacy Python. The success fixture exercises `http://user:pass@proxy.local:7890` becoming `http://***@proxy.local:7890` in status output (`web_model_download_manager.py:535`, `rewrite-in-rust/rust/crates/v2m-core/src/web_model_download.rs:187`, `rewrite-in-rust/fixtures/web_model_download_execution_result_contract.jsonl:1`).
- The unit does not overclaim migration of real downloads, real SocketIO transport, or OS process killing. The manifest, bootstrap record, and Rust rustdoc all keep those legacy-owned or reserved for later units (`rewrite-in-rust/manifest.yaml:631`, `rewrite-in-rust/bootstrap/web_model_download_execution_result_contract.md:31`, `rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_execution.rs:1`).

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_web_model_download_execution_result_contract.py`: passed
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml web_model_download_execution`: passed
- `uv run pytest tests/test_web_api.py -k ModelDownload -q`: passed, 19 passed and 34 deselected
- `git diff --check -- docs/web-api.md tests/test_web_api.py web_model_download_manager.py rewrite-in-rust/fixtures/web_model_download_execution_result_contract.jsonl rewrite-in-rust/bootstrap/check_web_model_download_execution_result_contract.py rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_execution.rs`: passed

## Residual Risk

This review did not prove live network downloads, package installation, archive extraction, real SocketIO delivery failures, or OS process-tree termination. Those are intentionally outside this unit and remain assigned to legacy Python or later model-download units.

## Promotion Note

This product ergonomics role does not block coordinator state update for `web_model_download_execution_result_contract`.
