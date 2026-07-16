# ustx_project_export_core - error_tracing_reviewer scalar final rerun

Date: 2026-07-16
Decision: pass-with-followups

## Findings

No blocking findings.

- Severity: low
- Location: rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:23
- Issue: `UstxProjectError` is still only a two-variant enum and does not implement `Display` or `std::error::Error`; the `TickOutOfRange` branch is present but still lacks direct diagnostic/overflow coverage. This is a promotion-time diagnosability gap, not a blocker for the current standalone renderer, because the unit is still outside production runtime wiring.
- Evidence: The enum exposes only `InvalidTempo` and `TickOutOfRange` at `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:25`; non-finite tempo returns `InvalidTempo` at `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:41`; non-finite or out-of-range tick conversion returns `TickOutOfRange` at `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:174`. The diagnostic sweep found no `Display` or `std::error::Error` implementation. Rollback and runtime-owner clarity remain explicit in `rewrite-in-rust/manifest.yaml:917`, `rewrite-in-rust/bootstrap/ustx_project_export_core.md:120`, and `rewrite-in-rust/records/0042-confirm-ustx-project-export-boundary.md:51`.
- Required fix: Before runtime promotion or bridge wiring, add caller-facing mapping for `InvalidTempo` and `TickOutOfRange`, implement standard error display if Rust errors cross a user-visible boundary, and add explicit overflow/tick-range diagnostic coverage.

## Verified Scope

The scalar resolver blocker is closed for the reviewed surface. Dynamic project names and lyrics route through `yaml_scalar` at `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:70`, `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:90`, and `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:149`. The final policy double-quotes tab, carriage return, and mixed control-character scalars at `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:250`, preserves PyYAML's multiline newline style at `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:243`, and quotes resolver-active numeric strings through `looks_like_yaml_number` at `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:343`.

The fixture now pins the previously failing semantic cases: the project/voice-part name `1.0e-07` is quoted at `rewrite-in-rust/fixtures/ustx_project_export_core/edge_project.ustx:1` and `rewrite-in-rust/fixtures/ustx_project_export_core/edge_project.ustx:242`; the binary-looking lyric `0b101` is quoted at `rewrite-in-rust/fixtures/ustx_project_export_core/edge_project.ustx:274`; mixed control characters and newline lyrics are represented at `rewrite-in-rust/fixtures/ustx_project_export_core/edge_project.ustx:298` and `rewrite-in-rust/fixtures/ustx_project_export_core/edge_project.ustx:322`. Rust unit tests cover the same scalar classes at `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:899` and `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:915`.

Finite exponent tempos are also closed. Rust normalizes exponent float text in `python_float_repr` at `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:193` and writes that value into root `bpm` and `tempos[0].bpm` at `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:80` and `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:84`. The golden fixture records PyYAML-compatible `1.0e-07` float text at `rewrite-in-rust/fixtures/ustx_project_export_core/empty_project.ustx:7` and `rewrite-in-rust/fixtures/ustx_project_export_core/empty_project.ustx:227`, with direct unit coverage at `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:923`.

Skipped invalid-note accounting is acceptable for this boundary. Rust increments `skipped_invalid_notes` at `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:45` and returns it at `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:108`; the parity checker validates expected skipped counts and legacy warning text at `rewrite-in-rust/bootstrap/check_ustx_project_export_core.py:90` and `rewrite-in-rust/bootstrap/check_ustx_project_export_core.py:96`. The bootstrap keeps warning printing, filesystem writes, and runtime routing out of this unit at `rewrite-in-rust/bootstrap/ustx_project_export_core.md:77`.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_ustx_project_export_core.py`: pass.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml ustx_project`: pass; 6 tests passed.
- `uv run python -c $'import pathlib\nimport yaml\n...'`: pass after rerun with escalation for the `uv` cache temp file; confirmed `empty_project.ustx` parses root and tempo-list `bpm` as floats, while `edge_project.ustx` parses `name`, voice-part `name`, `0b101`, `mix\n\t\r`, and `line\nbreak` as strings.
- `uv run python -c $'import yaml\nvalues=[...]'`: pass; confirmed PyYAML loads unquoted `1e-7`, `0o77`, and `0:01` as strings, but loads unquoted `1.0e-7`, `1.0e+20`, `0b101`, `077`, `0x10`, and `1:20` as numeric values and dumps those string values quoted. The Rust tests and fixtures now quote the resolver-active cases.
- `rg -n "Display|impl std::error::Error|UstxProjectError|InvalidTempo|TickOutOfRange|skipped_invalid_notes|Skipped|rollback|runtime owner|save_ustx|yaml_scalar|looks_like_yaml_(int|float|number)|needs_double_quoted_scalar|python_float_repr" ... -S`: pass; confirms current error type shape, scalar policy locations, skipped-note accounting, and rollback references.

## Residual Risk

This review covers the fixture-bound `save_ustx(..., rmvpe_result=None)` project renderer. It does not prove runtime bridge error mapping, filesystem write recovery, user-visible warning behavior after promotion, or RMVPE pitch-curve diagnostics. The scalar policy is still a selected PyYAML resolver subset for this project tree, not a general PyYAML replacement.

## Promotion Note

This error/tracing role does not block coordinator verification for `ustx_project_export_core`. The remaining low finding should be handled before runtime promotion or bridge exposure.
