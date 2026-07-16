# ustx_project_export_core - error_tracing_reviewer rerun

Date: 2026-07-16
Decision: fail

## Findings

- Severity: medium
- Location: rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:210
- Issue: The YAML scalar invalid/semantic mismatch blocker is only partially closed. The updated fixture and unit test now cover common hazards such as `a: b`, `yes`, `#tag`, and `{x: y}`, but `yaml_scalar` still has no branch for newline, carriage return, tab, or other control-character strings. Those values are still legal dynamic legacy inputs through `filepath.stem` and `note.lyric`; Rust would return `Ok` while emitting raw multi-line or tab-bearing YAML instead of PyYAML-compatible escaped/quoted YAML or a structured `UstxProjectError`.
- Evidence: Rust writes project names through `yaml_scalar` at `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:70` and `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:90`, and note lyrics at `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:149`. The current hazard detector at `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:210` handles trimming, boolean/null-like tokens, selected leading punctuation, `: `, ` #`, YAML-looking numbers, dates, and times, but does not inspect `\n`, `\r`, or `\t`. Legacy `save_ustx` accepts the same values from `filepath.stem` and `note.lyric or "a"` at `inference/API/ustx_api.py:369`, `inference/API/ustx_api.py:384`, `inference/API/ustx_api.py:413`, and serializes them with PyYAML at `inference/API/ustx_api.py:460`. A targeted PyYAML probe showed `line\nbreak` is emitted as a quoted multi-line scalar, `line\rbreak` as a double-quoted escaped scalar, and `tab\tvalue` as a double-quoted escaped scalar. `rg -n "\\\\n|\\\\r|\\\\t|line" rewrite-in-rust/fixtures/ustx_project_export_core.jsonl rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs rewrite-in-rust/bootstrap/ustx_project_export_core.md` found no scalar-control fixture coverage.
- Required fix: Extend the scalar policy to match PyYAML for supported dynamic strings containing line breaks, carriage returns, tabs, and similar control characters, or return a structured unsupported-scalar error before producing YAML. Add golden Python fixtures and Rust tests for those strings before this role can pass.

- Severity: low
- Location: rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:23
- Issue: `UstxProjectError` remains variant-only and has limited edge coverage. This is a promotion-time diagnostic risk rather than the current blocker because this unit is still fixture-bound and production filesystem/write mapping remains legacy-owned.
- Evidence: `UstxProjectError` has only `InvalidTempo` and `TickOutOfRange` at `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:25`; `rg -n "Display|impl std::error::Error|UstxProjectError|TickOutOfRange|InvalidTempo|non_finite|overflow" ...` found no `Display` or `std::error::Error` implementation, and only the non-finite tempo unit test at `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:669`. The rollback and runtime-owner docs keep `inference.API.ustx_api.save_ustx` as owner at `rewrite-in-rust/manifest.yaml:917`, `rewrite-in-rust/bootstrap/ustx_project_export_core.md:120`, and `rewrite-in-rust/records/0042-confirm-ustx-project-export-boundary.md:51`.
- Required fix: Before runtime promotion, define caller-facing mapping for `InvalidTempo` and `TickOutOfRange`, and add explicit tests or bridge diagnostics for non-finite tempo and tick overflow paths.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_ustx_project_export_core.py`: pass
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml ustx_project`: pass, 3 tests
- `uv run python -c 'import yaml; values=["line\nbreak","line\rbreak","tab\tvalue","quote '\\'' value"]; ...'`: pass; confirmed PyYAML quotes or escapes control-character scalar values.
- `uv run python -c '... save_ustx(... stem="stem\nbreak", lyric="line\nbreak") ...'`: pass; confirmed legacy accepts newline-bearing dynamic scalars and delegates their representation to PyYAML.
- `rg -n "\\\\n|\\\\r|\\\\t|line" rewrite-in-rust/fixtures/ustx_project_export_core.jsonl rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs rewrite-in-rust/bootstrap/ustx_project_export_core.md`: pass; no current fixture or scalar policy coverage for newline, carriage return, or tab scalar values.
- `rg -n "Display|impl std::error::Error|UstxProjectError|TickOutOfRange|InvalidTempo|non_finite|overflow" rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs rewrite-in-rust/bootstrap/check_ustx_project_export_core.py rewrite-in-rust/bootstrap/ustx_project_export_core.md rewrite-in-rust/dependencies/ustx_project_export_core.yaml -S`: pass; confirms current error type and coverage remain narrow.

## Residual Risk

The updated scalar-sensitive fixture closes the original concrete `a: b`, `yes`, `#tag`, and `{x: y}` examples. It does not prove the broader PyYAML-compatible dynamic scalar surface for strings containing control characters or line breaks, and the Rust renderer can still produce successful output that PyYAML would not have emitted.

## Promotion Note

This role still blocks coordinator verification for `ustx_project_export_core` because the structured-diagnostic YAML scalar issue is not fully closed. The bare `UstxProjectError` variants and missing overflow/non-finite edge tests remain promotion-only follow-ups while the Rust module stays outside the production Python runtime.
