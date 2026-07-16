# ustx_project_export_core - error_tracing_reviewer final rerun

Date: 2026-07-16
Decision: pass-with-followups

## Findings

No blocking findings.

- Severity: low
- Location: rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:23
- Issue: `UstxProjectError` remains a bare enum and the `TickOutOfRange` path is implemented but not directly unit-tested. This is not a current verification blocker because `ustx_project_export_core` is still an independent library renderer and production filesystem/write/error mapping remains legacy-owned.
- Evidence: The only structured error variants are `InvalidTempo` and `TickOutOfRange` at `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:25`; non-finite tempo returns `InvalidTempo` at `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:41`; out-of-range or non-finite tick conversion returns `TickOutOfRange` at `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:174`. `rg -n "Display|impl std::error::Error|UstxProjectError|InvalidTempo|TickOutOfRange|skipped_invalid_notes|Skipped|rollback|runtime owner|save_ustx" ... -S` found no `Display` or `std::error::Error` implementation. The runtime rollback remains explicit at `rewrite-in-rust/manifest.yaml:917`, `rewrite-in-rust/bootstrap/ustx_project_export_core.md:120`, and `rewrite-in-rust/records/0042-confirm-ustx-project-export-boundary.md:51`.
- Required fix: Before runtime promotion or bridge wiring, add caller-facing `InvalidTempo`/`TickOutOfRange` mapping, implement standard error display if the bridge consumes Rust errors directly, and add an explicit overflow/tick-range diagnostic test.

The previous blocker is closed. Dynamic scalars now route through `yaml_scalar` at `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:219`, emit PyYAML-compatible double-quoted escapes for tab, carriage return, and mixed control characters at `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:250`, and preserve PyYAML's newline single-quoted form at `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:243`. The exponent tempo formatter now produces parseable PyYAML-style exponent floats at `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:193`. Fixture coverage pins `1.0e-07` at `rewrite-in-rust/fixtures/ustx_project_export_core/empty_project.ustx:7` and `rewrite-in-rust/fixtures/ustx_project_export_core/empty_project.ustx:227`, plus control-character lyrics at `rewrite-in-rust/fixtures/ustx_project_export_core.jsonl:2`, `rewrite-in-rust/fixtures/ustx_project_export_core/edge_project.ustx:298`, and `rewrite-in-rust/fixtures/ustx_project_export_core/edge_project.ustx:322`. Rust unit coverage pins these renderings at `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:763` and `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:771`.

Skipped invalid-note accounting is acceptable for this boundary. Rust counts skipped invalid notes at `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:45` and returns the count at `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:108`; the parity checker validates the expected skipped count and legacy warning text at `rewrite-in-rust/bootstrap/check_ustx_project_export_core.py:90`. The bootstrap document correctly keeps user-visible warning mapping and filesystem writes out of this unit at `rewrite-in-rust/bootstrap/ustx_project_export_core.md:77`.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_ustx_project_export_core.py`: pass
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml ustx_project`: pass, 5 tests
- `uv run python -c "import yaml, pathlib; ..."`: pass; `empty_project.ustx` parses root and tempo-list `bpm` as floats, and `edge_project.ustx` parses `mix\n\t\r` and `line\nbreak` lyrics back to the legacy values.
- `rg -n "Display|impl std::error::Error|UstxProjectError|InvalidTempo|TickOutOfRange|skipped_invalid_notes|Skipped|rollback|runtime owner|save_ustx" rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs rewrite-in-rust/bootstrap/check_ustx_project_export_core.py rewrite-in-rust/bootstrap/ustx_project_export_core.md rewrite-in-rust/dependencies/ustx_project_export_core.yaml rewrite-in-rust/manifest.yaml rewrite-in-rust/records/0042-confirm-ustx-project-export-boundary.md -S`: pass; confirms current error type, skipped-note accounting, and rollback references.

## Residual Risk

This review only covers the fixture-bound `save_ustx(..., rmvpe_result=None)` project renderer. It does not prove runtime bridge error mapping, filesystem write recovery, user-visible warning text after promotion, or RMVPE pitch-curve diagnostics.

## Promotion Note

This error/tracing role no longer blocks coordinator verification for `ustx_project_export_core`. The remaining low finding should be handled before runtime promotion, not before marking this standalone unit reviewed.
