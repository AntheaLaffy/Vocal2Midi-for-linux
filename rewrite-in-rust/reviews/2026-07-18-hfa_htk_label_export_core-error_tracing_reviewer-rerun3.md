# hfa_htk_label_export_core - error_tracing_reviewer rerun3

Date: 2026-07-18
Unit: `hfa_htk_label_export_core`
Role: `error_tracing_reviewer`
Rerun after: `rewrite-in-rust/records/0091-fix-hfa-htk-pathlib-suffix-names.md`

## Findings

No findings.

- Severity: none
- Location: `rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:46`
- Issue: The error/tracing surface remains acceptable after record `0091`; the new Python `Path.with_suffix(".lab").name` projection preserves structured Python-compatible failures, keeps partial plans diagnosable, and does not introduce logging, filesystem effects, or a public-input panic path.
- Evidence: `HfaHtkExportError` still carries the Python exception type and message projection (`rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:46`, `rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:53`), and `HfaHtkExportFailure` still carries the cloned partial side-effect plan (`rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:73`). Timestamp failures clone the already accumulated plan before returning from each fallible conversion site (`rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:99`, `rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:103`, `rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:111`, `rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:116`), and path-name failures do the same before appending the current prediction's directory/file plans (`rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:125`, `rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:132`). Empty-name path diagnostics still project as Python `ValueError` messages after the new name projection (`rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:188`, `rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:223`), while `..` and trailing-dot names are accepted by the suffix projection rather than routed through that error path (`rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:201`, `rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:213`). The 20-row fixture set covers NaN and infinity exception type/message projection, empty-name `wav_path` errors, partial plans after prior prediction writes, and record `0091` suffix-name cases (`rewrite-in-rust/fixtures/hfa_htk_label_export_core.jsonl:10`, `rewrite-in-rust/fixtures/hfa_htk_label_export_core.jsonl:11`, `rewrite-in-rust/fixtures/hfa_htk_label_export_core.jsonl:16`, `rewrite-in-rust/fixtures/hfa_htk_label_export_core.jsonl:17`, `rewrite-in-rust/fixtures/hfa_htk_label_export_core.jsonl:18`, `rewrite-in-rust/fixtures/hfa_htk_label_export_core.jsonl:19`). The Rust module remains an in-memory planner; directory creation, writes, status printing, dispatch, and production routing remain legacy-owned (`rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:1`, `rewrite-in-rust/dependencies/hfa_htk_label_export_core.yaml:34`).
- Required fix: None.

## Decision

pass

## Checks

- `PYTHONDONTWRITEBYTECODE=1 uv run python rewrite-in-rust/bootstrap/check_hfa_htk_label_export_core.py`: passed, validated 20 Python 3.12 fixtures generated from the real `Exporter.save_htk`.
- `CARGO_TARGET_DIR=/tmp/v2m-hfa-htk-review-rerun3-target cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml --locked hfa_htk_label_export_core -- --nocapture`: passed, 1 focused Rust fixture-parity test; 115 `v2m-core` tests and 5 quant bridge tests were filtered out.
- `PYTHONDONTWRITEBYTECODE=1 uv run python - <<'PY' ...`: passed; confirmed Python projects `""`, `"."`, and `"./"` to `ValueError: PosixPath('.') has an empty name`, `"/"` to `ValueError: PosixPath('/') has an empty name`, and accepts `song.`, `..`, and `...` as `song..lab`, `...lab`, and `....lab`.
- `rustc --edition=2024 ... -o /tmp/v2m-hfa-htk-rerun3-empty-path-probe -` and `/tmp/v2m-hfa-htk-rerun3-empty-path-probe`: passed; the compiled Rust API returned structured `ValueError` failures with matching empty-name messages and zero-directory/zero-file partial plans for `""`, `"."`, `"./"`, and `"/"` without panicking.
- `rg -n "log::|tracing::|println!|eprintln!|dbg!|std::fs|fs::|File::|OpenOptions|create_dir|write_all|std::io|Command::" rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs`: no matches; no logging, process, or filesystem side effects are present in the planner.
- `rg -n "panic!|todo!|unimplemented!|unwrap\(|expect\(" rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs`: inspected; public-input code has one internal `String::from_utf8(digits).expect("decimal digits")` guarded by a private ASCII-digit construction invariant (`rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:282`), while the remaining panic/unwrap sites are under `#[cfg(test)]`.
- `rg -n "hfa_htk_export|plan_htk_label_export|HfaHtk|save_htk|Exporter\(" rewrite-in-rust/rust inference application web_server.py web_task_manager.py scripts tests -g '!target'`: inspected; production Python still calls legacy `Exporter.save_htk`, and Rust HTK code is only exposed inside `v2m-core`.
- `git diff --check -- rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs rewrite-in-rust/bootstrap/check_hfa_htk_label_export_core.py rewrite-in-rust/fixtures/hfa_htk_label_export_core.jsonl rewrite-in-rust/dependencies/hfa_htk_label_export_core.yaml rewrite-in-rust/bootstrap/hfa_htk_label_export_core.md rewrite-in-rust/records/0091-fix-hfa-htk-pathlib-suffix-names.md`: passed.

## Residual Risk

This role did not rerun behavior, dependency/bootstrap, data/algorithm, Rust style, architecture, or product ergonomics review. Real filesystem IO errors, status text, effect execution, export dispatch, and rollback execution remain outside this Rust planner and intentionally legacy-owned. Non-UTF-8 OS path display remains fixture-uncovered, but no bridge or production owner switch is part of this unit.

## Promotion Note

This `error_tracing_reviewer` third rerun is ready for coordinator state update for this role and does not block promotion on error/tracing grounds. The unit as a whole still needs the coordinator to combine this with the separately required review-role results; this report alone does not mark `hfa_htk_label_export_core` verified. This reviewer did not edit production code.
