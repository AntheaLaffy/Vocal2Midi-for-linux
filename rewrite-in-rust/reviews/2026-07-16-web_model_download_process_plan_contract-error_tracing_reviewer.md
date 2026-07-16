# web_model_download_process_plan_contract - error_tracing_reviewer

Date: 2026-07-16
Decision: pass

## Findings

No findings.

## Evidence

The unit is still scoped to diagnosable process planning and parser state, not
live execution. The manifest keeps `current_owner: legacy` and limits the public
policy to command construction, proxy environment shaping, process-group kwargs,
stdout framing, model guessing, progress math, log classification, and log cap
behavior without starting a child process (`rewrite-in-rust/manifest.yaml:615`,
`rewrite-in-rust/manifest.yaml:624`, `rewrite-in-rust/manifest.yaml:631`).
Record 0030 confirms the same fixture-bound surface and excludes real
`subprocess.Popen`, `download_models.py` execution, SocketIO delivery
guarantees, lifecycle transitions, termination, network downloads, package
installation, archive extraction, and marker safety
(`rewrite-in-rust/records/0030-confirm-web-model-download-process-plan-boundary.md:16`,
`rewrite-in-rust/records/0030-confirm-web-model-download-process-plan-boundary.md:31`).

Command and process-plan diagnostics are preserved without leaking proxy values
through command logs. Legacy command construction includes only the Python
executable, `download_models.py`, `--only`, optional `--qwen-source`, and
optional `--force` (`web_model_download_manager.py:316`). Proxy policy is
isolated in `_build_process_env`, which sets `PYTHONUNBUFFERED`, inherits,
clears, or injects proxy env keys as requested (`web_model_download_manager.py:370`).
The Rust model mirrors those two surfaces separately through `build_command` and
`build_process_env` (`rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_process.rs:146`,
`rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_process.rs:172`).
Fixtures cover command ordering, duplicate selected models, system/none/manual
proxy env behavior, and POSIX/Windows process-group kwargs
(`rewrite-in-rust/fixtures/web_model_download_process_plan_contract.jsonl:1`,
`rewrite-in-rust/fixtures/web_model_download_process_plan_contract.jsonl:4`,
`rewrite-in-rust/fixtures/web_model_download_process_plan_contract.jsonl:7`).

Output parser diagnostics are explicit and fixture-backed. Legacy
`_read_process_output` handles `stdout is None`, character buffering, newline
and carriage-return framing, and final-buffer handling before delegating to
`_handle_output_line` (`web_model_download_manager.py:392`). Legacy line handling
stores each line as a task log, classifies `failed`/`error` as `error`,
`ready`/`already` as `success`, guesses active model, parses legacy percent
progress, and applies ready/already-present completion (`web_model_download_manager.py:419`).
The Rust parser mirrors those state and payload effects without live IO
(`rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_process.rs:215`,
`rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_process.rs:273`).
Fixtures cover error/success classification, success-over-error legacy behavior,
selected-order model guessing, Unicode percent-boundary behavior, stdout `None`,
and carriage/newline/final-buffer output handling
(`rewrite-in-rust/fixtures/web_model_download_process_plan_contract.jsonl:10`,
`rewrite-in-rust/fixtures/web_model_download_process_plan_contract.jsonl:18`,
`rewrite-in-rust/fixtures/web_model_download_process_plan_contract.jsonl:19`).

SocketIO emit failure handling is correctly treated as a boundary, not silently
approved by this unit. Legacy `_emit_log` and `_emit_progress` catch transport
exceptions and print WebSocket errors while preserving task state
(`web_model_download_manager.py:468`, `web_model_download_manager.py:484`), but
record 0030 and the bootstrap note exclude SocketIO delivery errors from this
process-plan contract (`rewrite-in-rust/records/0030-confirm-web-model-download-process-plan-boundary.md:37`,
`rewrite-in-rust/bootstrap/web_model_download_process_plan_contract.md:45`).
The Rust side therefore returns fake emit payloads rather than modeling
transport success or failure (`rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_process.rs:435`).

The log cap is preserved and diagnosable. Legacy keeps the most recent 500 log
entries (`web_model_download_manager.py:476`), Rust applies the same cap
(`rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_process.rs:256`),
and the fixture proves an existing 500-entry log drops `old-0` and keeps the new
entry (`rewrite-in-rust/fixtures/web_model_download_process_plan_contract.jsonl:20`).

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_web_model_download_process_plan_contract.py`: pass
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml web_model_download_process`: pass, 2 matching tests passed
- `uv run pytest tests/test_web_api.py::TestModelDownloadProxyEnv -q`: pass, 7 passed
- `uv run pytest tests/test_web_api.py -k ModelDownload -q`: pass, 19 passed and 34 deselected
- `git diff --check -- web_model_download_manager.py rewrite-in-rust/fixtures/web_model_download_process_plan_contract.jsonl rewrite-in-rust/bootstrap/check_web_model_download_process_plan_contract.py rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_process.rs rewrite-in-rust/bootstrap/web_model_download_process_plan_contract.md rewrite-in-rust/dependencies/web_model_download_process_plan_contract.yaml rewrite-in-rust/records/0030-confirm-web-model-download-process-plan-boundary.md rewrite-in-rust/manifest.yaml tests/test_web_api.py download_models.py`: pass
- `git diff --check`: pass
- `rg -n "proxy|redact|emit|log|error|failed|ready|already|stdout|stderr|trace|Popen|download_models\\.py|SocketIO|WebSocket|500|PYTHONUNBUFFERED" web_model_download_manager.py rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_process.rs rewrite-in-rust/bootstrap/check_web_model_download_process_plan_contract.py rewrite-in-rust/fixtures/web_model_download_process_plan_contract.jsonl rewrite-in-rust/bootstrap/web_model_download_process_plan_contract.md rewrite-in-rust/dependencies/web_model_download_process_plan_contract.yaml rewrite-in-rust/records/0030-confirm-web-model-download-process-plan-boundary.md -S`: reviewed; the grep matched the expected scoped command/env/parser/log paths and the documented live-execution/SocketIO exclusions.

## Residual Risk

This review does not prove live subprocess startup, real stdout blocking
behavior, SocketIO transport failures, status-change emission, process
termination, network download errors, package installation, archive extraction,
or model marker safety. Those remain legacy-owned or covered by separate Web
model-download units. Proxy URLs are intentionally raw inside the child-process
environment map; user-visible proxy redaction is handled by the request/catalog
and execution/status contracts, not this process-plan helper.

## Promotion Note

This error-tracing role does not block coordinator verification for
`web_model_download_process_plan_contract`. The coordinator may mark this role
satisfied; the remaining required review roles are separate gates.
