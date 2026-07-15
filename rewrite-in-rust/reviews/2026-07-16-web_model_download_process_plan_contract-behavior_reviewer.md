# web_model_download_process_plan_contract - behavior_reviewer

Date: 2026-07-16
Decision: fail

## Findings

- Severity: medium
- Location: rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_process.rs:359
- Issue: The Rust percent parser does not fully preserve Python's legacy `re.search(r"\b(\d{1,3})%\b", line)` semantics for Unicode output lines. The legacy parser uses Python's Unicode-aware regex boundary and digit handling at `web_model_download_manager.py:429`; the Rust replacement scans bytes, accepts only ASCII digits at `rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_process.rs:363`, and defines word bytes as ASCII alphanumeric or `_` at `rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_process.rs:391`. That makes lines such as `GAME 50%完成` diverge: legacy emits a 50% progress event, while the Rust boundary check treats the CJK character after `%` as non-word and would not match.
- Evidence: A manual legacy probe through `ModelDownloadManager._handle_output_line` returned `{'progress': 50, 'stage': 'GAME'}` and emitted a progress event for `GAME 50%完成`; it returned no progress for `GAME progress 50%`, matching the documented surprising plain-percent behavior. Current fixtures only cover ASCII percent-boundary cases such as `50%x` and `GAME progress 50%` in `rewrite-in-rust/fixtures/web_model_download_process_plan_contract.jsonl:10` and `rewrite-in-rust/fixtures/web_model_download_process_plan_contract.jsonl:14`, so the parity checker misses this Unicode boundary gap.
- Required fix: Add Python/Rust fixture cases for Unicode word-boundary behavior around the percent sign, at minimum `GAME 50%完成` and a non-boundary case such as `GAME进度50%x`; then update the Rust percent parser to match Python regex semantics for the scoped output parser, or explicitly re-scope the unit with a recorded ASCII-only compatibility decision.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_web_model_download_process_plan_contract.py`: pass
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml web_model_download_process`: pass
- `uv run pytest tests/test_web_api.py::TestModelDownloadProxyEnv -q`: pass, 7 tests
- Manual legacy probe for `GAME 50%完成`, `GAME 50%x`, and `GAME progress 50%`: confirmed Python emits progress for the Unicode-boundary case and not for the plain trailing-percent case
- `git diff --check -- rewrite-in-rust/fixtures/web_model_download_process_plan_contract.jsonl rewrite-in-rust/bootstrap/check_web_model_download_process_plan_contract.py rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_process.rs rewrite-in-rust/manifest.yaml rewrite-in-rust/dependencies/web_model_download_process_plan_contract.yaml rewrite-in-rust/bootstrap/web_model_download_process_plan_contract.md rewrite-in-rust/records/0030-confirm-web-model-download-process-plan-boundary.md`: pass
- `cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check`: pass
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml`: pass, 58 tests
- `uv run pytest tests/test_web_api.py -q`: pass, 53 tests

## Residual Risk

Other reviewed behavior surfaces matched the current fixtures and legacy source inspection: command argv ordering, qwen-source inclusion, force placement, duplicate selected model command arguments, system/none/manual proxy env policy, POSIX/Windows spawn kwargs including missing Windows flag, fake stdout splitting and `stdout is None`, model guessing with selected-order priority, log level classification, success-overrides-error behavior, ready/already-present completion, progress clamp/non-decrease, event payload shape, and the 500-log cap.

The unit boundary itself stayed contained. `create_task`, active-task locking, thread/subprocess lifecycle, stop/termination, SocketIO delivery guarantees, real downloads, package install, archive extraction, and asset marker safety remain outside this unit per `rewrite-in-rust/records/0030-confirm-web-model-download-process-plan-boundary.md:31` and `rewrite-in-rust/dependencies/web_model_download_process_plan_contract.yaml:39`.

## Promotion Note

This behavior role blocks coordinator state update for `web_model_download_process_plan_contract` until the Unicode percent-boundary parity gap is fixed or intentionally re-scoped.
