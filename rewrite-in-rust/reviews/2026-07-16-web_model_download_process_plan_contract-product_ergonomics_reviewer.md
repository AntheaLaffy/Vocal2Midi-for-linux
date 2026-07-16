# web_model_download_process_plan_contract - product_ergonomics_reviewer

Date: 2026-07-16
Decision: pass-with-followups

## Findings

- Severity: low
- Location: docs/web-api.md:357
- Issue: The public SocketIO table under-describes model-download progress payloads. The process-plan contract emits `progress` events with `task_type: "model_download"` and model-label stages such as `GAME`, `romajiASR`, or `Qwen3-ASR-1.7B`; the browser depends on that `task_type` to route progress to the model-download panel. The docs only list `task id, progress, stage` for all progress events and separately call out `task_type` only for model-download log entries.
- Evidence: Legacy progress payload includes `task_type` at `web_model_download_manager.py:484`; Rust mirrors it at `rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_process.rs:303`; the browser routes model-download progress by `data.task_type` at `Vocal2Midi Web.html:1633` and renders unknown stages as `下载中: ${stage}` at `Vocal2Midi Web.html:2449`.
- Required fix: Non-blocking docs follow-up: document that model-download `progress` payloads include `task_type: "model_download"`, that process-plan parser stages may be model labels, and that in-process parser progress is capped below 100 until terminal completion.

- Severity: low
- Location: web_model_download_manager.py:476
- Issue: The 500-entry retained-log cap is preserved, but operators are not told when a rejoined/download-status log backlog is truncated. Connected clients still see live streamed entries, but a reconnecting or late-joining operator may copy or inspect the replayed model-download logs without realizing only the most recent 500 retained entries are available.
- Evidence: Legacy Python caps `task.logs` to the latest 500 entries at `web_model_download_manager.py:476`; Rust mirrors that at `rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_process.rs:256`; the fixture locks the oldest-retained behavior at `rewrite-in-rust/fixtures/web_model_download_process_plan_contract.jsonl:20`; the browser replays `backlogs` without a truncation indicator at `Vocal2Midi Web.html:1656` and appends model log entries without cap metadata at `Vocal2Midi Web.html:2472`.
- Required fix: Non-blocking UX/docs follow-up: either document the retained backlog cap or add a capped-backlog hint in the model-download log panel when the server replays exactly the retained window.

## Evidence

- Unit and role are explicit: `web_model_download_process_plan_contract` with `product_ergonomics_reviewer`. The manifest keeps the unit `reimplemented`, confirms source refs, and requires this product ergonomics role while preserving command construction, proxy env shaping, process-group kwargs, stdout framing, model guessing, progress math, log classification, and log cap behavior without starting a child process (`rewrite-in-rust/manifest.yaml:615`, `rewrite-in-rust/manifest.yaml:624`, `rewrite-in-rust/manifest.yaml:625`).
- The boundary stayed inside process planning and output parsing. Record 0030 includes command order, proxy env behavior, stdout line framing, progress parsing/math, log classification, and log cap, while excluding real `subprocess.Popen`, `download_models.py` execution, active-task locking, cancellation transitions, process termination, SocketIO delivery guarantees, downloads, package installation, archive extraction, and marker safety (`rewrite-in-rust/records/0030-confirm-web-model-download-process-plan-boundary.md:16`, `rewrite-in-rust/records/0030-confirm-web-model-download-process-plan-boundary.md:31`).
- Writer/reviewer separation is preserved for this pass. I only inspected files and wrote this report; no production code, fixtures, bootstrap scripts, Rust source, or manifest entries were edited.
- User-visible command/model order behavior is fixture-backed: selected model order is preserved in command construction, qwen source is appended only when qwen is selected, and `--force` is appended last (`rewrite-in-rust/fixtures/web_model_download_process_plan_contract.jsonl:1`, `rewrite-in-rust/fixtures/web_model_download_process_plan_contract.jsonl:2`, `web_model_download_manager.py:316`, `rewrite-in-rust/rust/crates/v2m-core/src/web_model_download_process.rs:146`). The Web route deduplicates API input while preserving UI order before process planning (`web_server.py:672`).
- Proxy mode effects are aligned across process planning and the browser: `system` inherits proxy env, `none` clears upper/lower proxy keys, and `manual` trims the URL, clears inherited proxy keys, then sets HTTP/HTTPS/ALL proxy variables in upper/lower case (`web_model_download_manager.py:370`, `rewrite-in-rust/fixtures/web_model_download_process_plan_contract.jsonl:4`, `rewrite-in-rust/fixtures/web_model_download_process_plan_contract.jsonl:5`, `rewrite-in-rust/fixtures/web_model_download_process_plan_contract.jsonl:6`). The Web UI exposes the same modes, validates manual URLs before start, and logs the selected proxy mode with manual credentials redacted (`Vocal2Midi Web.html:1344`, `Vocal2Midi Web.html:2348`, `Vocal2Midi Web.html:2360`).
- Progress/log behavior matches the process-plan output parsing contract: line logs are classified as error/success/info, model guessing follows selected-model order, per-model progress is aggregated across selected models and never decreases, ready/already-present lines mark model completion, percent parsing includes the Unicode-boundary behavior resolved by the behavior rerun, and process-running progress remains capped at 99 (`web_model_download_manager.py:419`, `web_model_download_manager.py:440`, `rewrite-in-rust/fixtures/web_model_download_process_plan_contract.jsonl:10`, `rewrite-in-rust/fixtures/web_model_download_process_plan_contract.jsonl:13`, `rewrite-in-rust/fixtures/web_model_download_process_plan_contract.jsonl:15`, `rewrite-in-rust/reviews/2026-07-16-web_model_download_process_plan_contract-behavior_reviewer-rerun.md:10`).
- Cancellation and recovery expectations are correctly excluded from this unit. The docs and UI expose stop/rejoin workflows, but task lifecycle, terminal cancellation, real process termination, and SocketIO delivery guarantees are covered by adjacent units or remain legacy-owned (`docs/web-api.md:310`, `Vocal2Midi Web.html:1606`, `Vocal2Midi Web.html:2394`, `rewrite-in-rust/reviews/2026-07-16-web_model_download_execution_result_contract-product_ergonomics_reviewer.md:10`, `rewrite-in-rust/reviews/2026-07-16-web_model_download_process_termination_contract-product_ergonomics_reviewer.md:12`).
- The current working-tree diff does not modify this process-plan unit's source, fixtures, bootstrap record, dependency record, docs, browser UI, or tests. The dirty tracked files are other rewrite units plus manifest state updates.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_web_model_download_process_plan_contract.py`: passed
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml web_model_download_process`: passed, 2 selected tests
- `uv run pytest tests/test_web_api.py -k ModelDownload -q`: passed, 19 passed and 34 deselected
- `git diff --check`: passed
- `git diff --name-only`: current tracked diff is unrelated to this unit's source/fixture/bootstrap/Rust/docs/UI/test files; it contains other rewrite-unit files and manifest status updates.

## Residual Risk

This product ergonomics review did not run a live browser, spawn `download_models.py`, contact network services, install packages, extract archives, or prove real WebSocket transport delivery. Those behaviors are outside this process-plan unit by record 0030 and remain legacy-owned or assigned to adjacent model-download/download-models units.

The two findings are non-blocking documentation/operator-visibility follow-ups. They do not show a behavior regression in command planning, proxy effects, progress parsing, model order, or current Web UI event consumption.

## Promotion Note

This product ergonomics role may be marked satisfied with follow-ups for `web_model_download_process_plan_contract`. The coordinator should still require the remaining manifest-required roles, including error tracing if not already completed or explicitly waived, before marking the unit verified.
