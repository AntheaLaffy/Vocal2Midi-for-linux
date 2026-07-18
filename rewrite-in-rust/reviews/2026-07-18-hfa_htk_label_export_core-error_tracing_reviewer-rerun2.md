# hfa_htk_label_export_core - error_tracing_reviewer rerun2

Date: 2026-07-18
Unit: `hfa_htk_label_export_core`
Role: `error_tracing_reviewer`
Rerun after: `rewrite-in-rust/records/0090-fix-hfa-htk-current-directory-paths.md`

## Findings

No findings.

- Severity: none
- Location: `rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:46`
- Issue: The error/tracing surface remains acceptable after record `0090`; the new `pathlib`-style normalization path preserves Python-compatible error projection and does not add panics, logging, filesystem effects, or weaker partial-plan diagnostics.
- Evidence: `HfaHtkExportError` still exposes the Python exception type and exact message (`rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:46`, `rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:53`), and `HfaHtkExportFailure` still carries the partial side-effect plan (`rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:73`). Timestamp and path failures clone the accumulated plan before returning (`rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:99`, `rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:125`), so conversion failures after prior predictions remain diagnosable. The record `0090` path projection is implemented by removing `.` components while preserving absolute roots (`rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:182`), and empty-name diagnostics map an empty normalized path back to `"."` (`rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:203`). The 17-row fixture set covers current-directory path text, empty-name `wav_path` errors, NaN/Infinity error types/messages, and partial plans after prior writes (`rewrite-in-rust/fixtures/hfa_htk_label_export_core.jsonl:10`, `rewrite-in-rust/fixtures/hfa_htk_label_export_core.jsonl:11`, `rewrite-in-rust/fixtures/hfa_htk_label_export_core.jsonl:13`, `rewrite-in-rust/fixtures/hfa_htk_label_export_core.jsonl:16`). The source boundary keeps real directory creation and file writes in Python `Exporter.save_htk` (`inference/HubertFA/tools/export_tool.py:35`, `inference/HubertFA/tools/export_tool.py:57`), while the Rust module documents that filesystem effects, status printing, dispatch, and routing remain legacy-owned (`rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:1`, `rewrite-in-rust/dependencies/hfa_htk_label_export_core.yaml:34`).
- Required fix: None.

## Decision

pass

## Checks

- `PYTHONDONTWRITEBYTECODE=1 uv run python rewrite-in-rust/bootstrap/check_hfa_htk_label_export_core.py`: passed, validated 17 Python 3.12 fixtures generated from the real `Exporter.save_htk`.
- `CARGO_TARGET_DIR=/tmp/v2m-hfa-htk-review-rerun2-target cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml --locked hfa_htk_label_export_core -- --nocapture`: passed, 1 focused Rust fixture-parity test; 115 `v2m-core` tests and 5 quant bridge tests were filtered out.
- `PYTHONDONTWRITEBYTECODE=1 uv run python - <<'PY' ...`: targeted legacy probe confirmed `""`, `"."`, and `"./"` raise `ValueError: PosixPath('.') has an empty name`, `"/"` raises `ValueError: PosixPath('/') has an empty name`, and current-directory HTK paths render without a leading `./`.
- `rustc --edition=2024 ... /tmp/v2m-hfa-htk-empty-path-probe`: targeted Rust API probe confirmed `""`, `"."`, `"./"`, and `"/"` return structured `ValueError` failures with matching messages and empty partial plans; no panic occurred.
- `rg -n "log::|tracing::|println!|eprintln!|std::fs|fs::|File::|OpenOptions|create_dir|write_all|std::io|Command::" rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs`: passed with no matches.
- `rg -n "hfa_htk_export|plan_htk_label_export|HfaHtk|save_htk|Exporter\\(" rewrite-in-rust/rust inference application web_server.py web_task_manager.py scripts tests -g '!target'`: inspected routing references; production Python still calls legacy `Exporter.save_htk`, and Rust HTK code is only exposed inside `v2m-core`.
- `git diff --check -- rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs rewrite-in-rust/bootstrap/check_hfa_htk_label_export_core.py rewrite-in-rust/fixtures/hfa_htk_label_export_core.jsonl rewrite-in-rust/dependencies/hfa_htk_label_export_core.yaml rewrite-in-rust/bootstrap/hfa_htk_label_export_core.md rewrite-in-rust/records/0090-fix-hfa-htk-current-directory-paths.md`: passed.

## Residual Risk

This role did not rerun behavior, dependency/bootstrap, data/algorithm, Rust style, architecture, or product ergonomics review. Real filesystem IO errors, status text, effect execution, export dispatch, and rollback execution remain outside this Rust planner and intentionally legacy-owned. Non-UTF-8 OS path display remains fixture-uncovered, but no bridge or production owner switch is part of this unit.

## Promotion Note

This `error_tracing_reviewer` second rerun is ready for coordinator state update for this role and does not block promotion on error/tracing grounds. The unit as a whole still needs the coordinator to combine this with the separately required review-role results; this report alone does not mark `hfa_htk_label_export_core` verified.
