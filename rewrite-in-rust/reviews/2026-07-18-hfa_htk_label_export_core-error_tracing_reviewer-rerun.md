# hfa_htk_label_export_core - error_tracing_reviewer rerun

Date: 2026-07-18
Decision: pass

Unit: `hfa_htk_label_export_core`
Role: `error_tracing_reviewer`
Rerun after: `rewrite-in-rust/records/0089-fix-hfa-htk-review-findings.md`

## Findings

No findings.

- Severity: none
- Location: `rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:46`
- Issue: The error/tracing surface remains acceptable after the record 0089 fixes for empty-name paths and arbitrary-size finite HTK time rendering.
- Evidence: `HfaHtkExportError` still carries the Python exception type and exact message, and `HfaHtkExportFailure` carries the partial side-effect plan for completed directory/file plans (`rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:46`, `rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:73`). Timestamp conversion failures clone the partial plan before returning at each fallible word/phoneme conversion site (`rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:99`, `rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:103`, `rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:111`, `rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:116`), and the new path-name failure does the same before any directory or file plan is appended (`rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:125`, `rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:132`). The empty-name error projects as `ValueError: PosixPath('.') has an empty name` for an empty `wav_path`, matching the added Python fixture (`rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:177`, `rewrite-in-rust/fixtures/hfa_htk_label_export_core.jsonl:13`). NaN and infinity still project to Python-compatible `ValueError`/`OverflowError` messages, while finite huge values render through a decimal string path instead of a fixed-width cast (`rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:200`, `rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:217`, `rewrite-in-rust/fixtures/hfa_htk_label_export_core.jsonl:9`). The Rust module still performs no logging, status printing, directory creation, file writes, bridge calls, or production routing; those remain legacy-owned by the unit boundary (`rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:1`, `rewrite-in-rust/dependencies/hfa_htk_label_export_core.yaml:34`).
- Required fix: None.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_hfa_htk_label_export_core.py`: passed, validated 14 fixtures generated from the real Python `Exporter.save_htk`.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_htk_label_export_core -- --nocapture`: passed, 1 focused Rust fixture-parity test; 115 `v2m-core` tests and 5 quant bridge tests were filtered out.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_htk_export::tests::hfa_htk_label_export_core_fixture_parity -- --exact --nocapture`: passed the exact focused Rust test.
- `uv run python - <<'PY' ...`: targeted Python probe confirmed empty/`.`/root path `ValueError` messages, NaN `ValueError`, positive and negative infinity `OverflowError`, huge finite arbitrary-precision integer text, and overflow after scaled infinity.
- `uv run python -m py_compile inference/HubertFA/tools/export_tool.py rewrite-in-rust/bootstrap/check_hfa_htk_label_export_core.py`: passed.
- `rg -n "hfa_htk_export|plan_htk_label_export|HfaHtk|save_htk|Exporter\\(" rewrite-in-rust/rust inference application web_server.py web_task_manager.py scripts tests -g '!target'`: inspected routing references; production Python still calls legacy `Exporter.save_htk`, and Rust HTK code is only exposed inside `v2m-core`.
- `git diff --check -- rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs rewrite-in-rust/bootstrap/check_hfa_htk_label_export_core.py rewrite-in-rust/fixtures/hfa_htk_label_export_core.jsonl rewrite-in-rust/dependencies/hfa_htk_label_export_core.yaml rewrite-in-rust/bootstrap/hfa_htk_label_export_core.md rewrite-in-rust/records/0089-fix-hfa-htk-review-findings.md`: passed.

## Residual Risk

This role did not rerun behavior, dependency/bootstrap, data/algorithm, Rust style, architecture, or product ergonomics review. Real filesystem IO failures, status text, effect execution, and rollback remain outside this Rust planner and intentionally legacy-owned until a promotion unit defines execution behavior. Non-UTF-8 OS path diagnostic projection is not covered by the current fixture set, but no production bridge or runtime owner switch is part of this unit.

## Promotion Note

This `error_tracing_reviewer` rerun does not block coordinator state update for this role. The unit is ready for coordinator state update with respect to `error_tracing_reviewer`, subject to the coordinator combining it with the separately required review-role results. This reviewer did not edit production code.
