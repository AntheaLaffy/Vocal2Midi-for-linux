# note_text_csv_export_core - error_tracing_reviewer

Date: 2026-07-15
Decision: pass

## Findings

No findings.

Evidence:

- `rewrite-in-rust/manifest.yaml:133` keeps this unit at `reimplemented`, with legacy Python still the current owner and rollback to `inference.io.note_io`.
- `rewrite-in-rust/bootstrap/note_text_csv_export_core.md:59` defines the seam as an independent Rust library with no bridge dependencies.
- `rewrite-in-rust/bootstrap/note_text_csv_export_core.md:102` states rollback is keeping production imports unchanged, and `rewrite-in-rust/bootstrap/note_text_csv_export_core.md:110` defers filesystem-write and warning-message mapping to a later promotion record.
- `inference/io/note_io.py:19` skips invalid notes and emits only a count warning at `inference/io/note_io.py:31`.
- `rewrite-in-rust/rust/crates/v2m-core/src/export.rs:29` exposes `TextExport` with rendered content and `skipped_invalid_notes`; `rewrite-in-rust/rust/crates/v2m-core/src/export.rs:43` counts invalid skipped notes before rendering.
- `rewrite-in-rust/fixtures/note_text_csv_export_core.tsv:4` covers skipped invalid note reporting, and `rewrite-in-rust/rust/crates/v2m-core/src/export.rs:337` asserts the skipped count against the fixture table.

## Checks

- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml export`: passed; 3 export tests passed.
- `uv run python rewrite-in-rust/bootstrap/check_note_text_csv_export_core.py`: passed; legacy Python emitted `[Warning] Skipped 3 invalid note(s) during export.` for the invalid-note fixture and all expected outputs matched.

## Residual Risk

This review covers only the pre-promotion library seam. The Rust API is intentionally infallible for in-memory rendering because invalid notes are warnings, not hard errors, in the legacy path. Future promotion work still needs an explicit bridge/error mapping record for:

- converting bridge string inputs into `TextFileFormat` and `PitchFormat`;
- mapping `skipped_invalid_notes` back to the legacy warning text;
- mapping filesystem and encoding/write failures if Rust starts owning file output;
- deciding whether per-note invalid index/reason diagnostics are worth adding beyond current Python parity.

No lyric or path data is logged by the Rust library seam, so no new redaction issue is introduced in this unit.

## Promotion Note

This error-tracing role does not block moving `note_text_csv_export_core` from `reimplemented` to `verified` once the required behavior review also passes. It does not approve runtime ownership promotion; bridge, file I/O, and warning-message mapping remain separate promotion work.
