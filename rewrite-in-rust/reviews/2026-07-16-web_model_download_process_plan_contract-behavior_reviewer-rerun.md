# web_model_download_process_plan_contract - behavior_reviewer rerun

Date: 2026-07-16
Decision: pass

## Findings

No findings.

The previous medium finding is resolved. The fixture table now covers both
Unicode percent-boundary edges: `GAME 50%完成` must emit 50 percent progress,
and `GAME进度50%x` must not emit progress
(`rewrite-in-rust/fixtures/web_model_download_process_plan_contract.jsonl:15`,
`rewrite-in-rust/fixtures/web_model_download_process_plan_contract.jsonl:16`).
The Python checker drives those cases through legacy
`ModelDownloadManager._handle_output_line`
(`rewrite-in-rust/bootstrap/check_web_model_download_process_plan_contract.py:242`,
`rewrite-in-rust/bootstrap/check_web_model_download_process_plan_contract.py:247`),
whose source uses Python's Unicode-aware
`re.search(r"\b(\d{1,3})%\b", line)`
(`web_model_download_manager.py:429`).

The Rust implementation now scans chars rather than ASCII bytes and applies
word-boundary checks around the digit run and `%`
(`rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_process.rs:359`,
`rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_process.rs:377`,
`rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_process.rs:389`).
It also covers common Unicode decimal digit ranges used by the Rust regression
test (`rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_process.rs:399`,
`rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_process.rs:409`,
`rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_process.rs:628`).

The behavior slice stayed inside the confirmed boundary. It covers process
command/env planning and output parsing without real subprocess, SocketIO
delivery, lifecycle, termination, network download, package install, archive
extraction, or model marker safety
(`rewrite-in-rust/manifest.yaml:585`,
`rewrite-in-rust/manifest.yaml:594`,
`rewrite-in-rust/manifest.yaml:604`,
`rewrite-in-rust/bootstrap/web_model_download_process_plan_contract.md:45`,
`rewrite-in-rust/dependencies/web_model_download_process_plan_contract.yaml:39`,
`rewrite-in-rust/records/0030-confirm-web-model-download-process-plan-boundary.md:31`).

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_web_model_download_process_plan_contract.py`: pass
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml web_model_download_process`: pass, 2 tests
- `uv run pytest tests/test_web_api.py::TestModelDownloadProxyEnv -q`: pass, 7 tests
- Manual legacy probe for `GAME 50%完成`, `GAME进度50%x`, `GAME ٩٠%完成`, `GAME ５０%完成`, and `GAME progress 50%`: pass; Python emitted progress for the Unicode boundary and Unicode decimal digit cases, and emitted no progress for the start-boundary and plain trailing-percent cases
- `cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check`: pass
- `git diff --check -- rewrite-in-rust/fixtures/web_model_download_process_plan_contract.jsonl rewrite-in-rust/bootstrap/check_web_model_download_process_plan_contract.py rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_process.rs rewrite-in-rust/manifest.yaml rewrite-in-rust/dependencies/web_model_download_process_plan_contract.yaml rewrite-in-rust/bootstrap/web_model_download_process_plan_contract.md rewrite-in-rust/records/0030-confirm-web-model-download-process-plan-boundary.md rewrite-in-rust/reviews/2026-07-16-web_model_download_process_plan_contract-behavior_reviewer.md`: pass
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml`: pass, 58 tests
- `uv run pytest tests/test_web_api.py -q`: pass, 53 tests

## Residual Risk

This review proves behavior parity for the scoped fixture table, focused legacy
probe, and inspected source paths. It does not exhaustively prove every Unicode
decimal digit or every Python regex category edge. Lifecycle behavior,
termination behavior, actual `download_models.py` execution, SocketIO transport
errors, network downloads, and archive/model asset safety remain outside this
unit by boundary record.

## Promotion Note

This behavior rerun no longer blocks coordinator state update for
`web_model_download_process_plan_contract`. The manifest should still be updated
only by the coordinator after the required non-behavior review roles for this
unit are handled.
