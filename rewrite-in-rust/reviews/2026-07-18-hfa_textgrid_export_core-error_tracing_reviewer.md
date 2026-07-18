# hfa_textgrid_export_core - error_tracing_reviewer

Date: 2026-07-18
Decision: pass
Unit: `hfa_textgrid_export_core`
Role: `error_tracing_reviewer`

## Findings

No findings.

- Severity: none
- Location: `rewrite-in-rust/rust/crates/v2m-core/src/hfa_textgrid_export.rs:45`
- Issue: The error/tracing surface is acceptable for this unit. The Rust planner exposes a structured error with Python exception type/message projection, carries a partial side-effect plan on failures, preserves legacy interval/path/serialization failure ordering, and does not add logging or filesystem effects.
- Evidence: `HfaTextGridExportError` stores the legacy exception type and exact message projection behind accessors (`rewrite-in-rust/rust/crates/v2m-core/src/hfa_textgrid_export.rs:45`, `rewrite-in-rust/rust/crates/v2m-core/src/hfa_textgrid_export.rs:52`), and `HfaTextGridExportFailure` carries the cloned partial plan (`rewrite-in-rust/rust/crates/v2m-core/src/hfa_textgrid_export.rs:72`). The planner returns failures before mutating the current prediction's plan for interval and path errors, but after adding the directory plan for serialization errors, matching the Python order in `Exporter.save_textgrids` (`rewrite-in-rust/rust/crates/v2m-core/src/hfa_textgrid_export.rs:106`, `rewrite-in-rust/rust/crates/v2m-core/src/hfa_textgrid_export.rs:113`, `rewrite-in-rust/rust/crates/v2m-core/src/hfa_textgrid_export.rs:121`, `rewrite-in-rust/rust/crates/v2m-core/src/hfa_textgrid_export.rs:130`; `inference/HubertFA/tools/export_tool.py:11`). Interval errors project legacy `ValueError` messages for zero-duration, bounds, and overlap cases (`rewrite-in-rust/rust/crates/v2m-core/src/hfa_textgrid_export.rs:179`, `rewrite-in-rust/rust/crates/v2m-core/src/hfa_textgrid_export.rs:199`, `third_party/sources/textgrid-1.6.1/textgrid/textgrid.py:186`, `third_party/sources/textgrid-1.6.1/textgrid/textgrid.py:453`), while path projection preserves the empty-name `ValueError` surface (`rewrite-in-rust/rust/crates/v2m-core/src/hfa_textgrid_export.rs:370`). Serialization fallback keeps the legacy `IndexError: list index out of range` path when zero max-time tiers have no intervals (`rewrite-in-rust/rust/crates/v2m-core/src/hfa_textgrid_export.rs:284`, `third_party/sources/textgrid-1.6.1/textgrid/textgrid.py:751`). Fixture evidence covers invalid zero-duration and overlap errors before side effects, an out-of-range interval after a prior prediction write, empty wav-path errors before side effects, exact quote/newline/Unicode rendering, and repeated stateless calls (`rewrite-in-rust/fixtures/hfa_textgrid_export_core.jsonl:3`, `rewrite-in-rust/fixtures/hfa_textgrid_export_core.jsonl:11`, `rewrite-in-rust/fixtures/hfa_textgrid_export_core.jsonl:12`, `rewrite-in-rust/fixtures/hfa_textgrid_export_core.jsonl:13`, `rewrite-in-rust/fixtures/hfa_textgrid_export_core.jsonl:14`, `rewrite-in-rust/fixtures/hfa_textgrid_export_core.jsonl:15`). The module-level contract keeps status printing, directory creation, file writes, artifact copying, dispatch, and production routing out of Rust (`rewrite-in-rust/rust/crates/v2m-core/src/hfa_textgrid_export.rs:1`; `rewrite-in-rust/dependencies/hfa_textgrid_export_core.yaml:36`), and rollback remains the legacy Python/textgrid owner (`rewrite-in-rust/manifest.yaml:1625`, `rewrite-in-rust/records/0093-implement-hfa-textgrid-export-core.md:59`).
- Required fix: None.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_hfa_textgrid_export_core.py`: passed; validated 15 Python 3.12 fixtures generated through the real `Exporter.save_textgrids` with monkeypatched `Path.mkdir` and `codecs.open`.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_textgrid_export_core -- --nocapture`: passed; 1 focused Rust fixture-parity test passed, 116 `v2m-core` tests and 5 quant bridge tests were filtered out.
- `rg -n "std::fs|File::|create_dir|write_all|println!|eprintln!|dbg!|log::|tracing::" rewrite-in-rust/rust/crates/v2m-core/src/hfa_textgrid_export.rs`: no matches; the Rust planner has no logging, tracing, stdout/stderr macro, filesystem, or write side effects.
- `rg -n "panic!|todo!|unimplemented!|unwrap\\(|expect\\(" rewrite-in-rust/rust/crates/v2m-core/src/hfa_textgrid_export.rs`: inspected; the only non-test match is an internal `expect("Rust float debug exponent is an integer")` guarded by Rust `f64` debug-format structure, and the remaining panic/unwrap sites are under `#[cfg(test)]`.
- `rg -n "plan_textgrid_export|HfaTextGrid|hfa_textgrid_export|save_textgrids|TextGrid" rewrite-in-rust/rust inference application web_server.py web_task_manager.py scripts tests -g '!target'`: inspected; production Python still uses legacy `Exporter.save_textgrids`, and the Rust TextGrid code is exposed only inside the independent `v2m-core` workspace.
- `git diff --check -- rewrite-in-rust/rust/crates/v2m-core/src/hfa_textgrid_export.rs rewrite-in-rust/bootstrap/check_hfa_textgrid_export_core.py rewrite-in-rust/fixtures/hfa_textgrid_export_core.jsonl rewrite-in-rust/dependencies/hfa_textgrid_export_core.yaml rewrite-in-rust/bootstrap/hfa_textgrid_export_core.md rewrite-in-rust/records/0093-implement-hfa-textgrid-export-core.md`: passed.

## Residual Risk

This role did not review behavior parity beyond the error/tracing evidence, dependency sufficiency, data/algorithm choices, Rust style, architecture, or product ergonomics. Real filesystem IO errors, status text, export dispatch, API artifact copying, and execution of planned side effects remain intentionally legacy-owned. Redaction risk is unchanged from Python: projected errors may include user-supplied interval marks or path display text when legacy Python would include them, but the Rust planner adds no logs or additional disclosure channel.

## Promotion Note

This `error_tracing_reviewer` role is ready for coordinator state update and does not block promotion on error/tracing grounds. The coordinator still needs to combine this with the other required review roles before marking `hfa_textgrid_export_core` verified. This reviewer did not edit production code.
