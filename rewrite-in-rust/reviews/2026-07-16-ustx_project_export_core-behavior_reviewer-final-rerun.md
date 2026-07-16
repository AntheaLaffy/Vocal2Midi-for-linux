# ustx_project_export_core - behavior_reviewer final rerun

Date: 2026-07-16
Decision: fail

## Findings

- Severity: medium
- Location: rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:343
- Issue: Dynamic YAML scalar parity is still incomplete for valid project names and lyrics that look like YAML exponent floats or binary integers. Legacy `save_ustx(..., rmvpe_result=None)` accepts `filepath.stem` and `note.lyric or "a"` without validation, then lets PyYAML quote string scalars that would otherwise resolve as non-strings. Rust routes those same fields through `yaml_scalar`, but `looks_like_yaml_number` does not recognize exponent-style strings such as `1.0e-7` or binary integer strings such as `0b101`, so Rust can emit resolver-active plain scalars for accepted string inputs.
- Evidence: Python takes dynamic note lyrics at `inference/API/ustx_api.py:384`, project names at `inference/API/ustx_api.py:414` and `inference/API/ustx_api.py:447`, and serializes with `yaml.safe_dump(..., allow_unicode=True, sort_keys=False)` at `inference/API/ustx_api.py:460`. Rust emits project names and lyrics through `yaml_scalar` at `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:70`, `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:90`, and `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:149`. The current numeric-looking-string detector only handles signs, hex prefixes, digits, dots, and underscores at `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:343` through `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:367`. A direct PyYAML probe showed `"lyric: '1.0e-7'"`, `"lyric: '1.0e+20'"`, and `"lyric: '0b101'"`; the same probe confirmed that unquoted `1.0e-7`, `1.0e+20`, and `0b101` resolve as float/int values rather than strings.
- Required fix: Extend the scalar resolver guard to quote PyYAML/YAML-1.1 numeric-looking string forms such as exponent floats and binary integers, or explicitly narrow and validate the accepted `project_name` and lyric domain. Add golden Python fixtures for at least one exponent-like string lyric or stem before rerunning behavior review.

- Severity: low
- Location: rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:305
- Issue: Exact YAML text parity remains broader than the current fixture proof. The Rust predicate quotes every scalar starting with `-` or `?`, while PyYAML leaves examples such as `-x` and `?x` as plain scalars. This does not change parsed string semantics for those examples, but it means the renderer still should not be treated as byte-for-byte PyYAML-compatible for arbitrary dynamic strings.
- Evidence: `needs_single_quoted_scalar` treats leading `-` and `?` as quote triggers at `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:305` through `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:324`. The PyYAML probe printed `lyric: -x` and `lyric: ?x` for those string values.
- Required fix: Either align the indicator rules with PyYAML for exact text parity or document that semantically equivalent over-quoting is acceptable for non-resolver-active dynamic strings.

## Resolved Scope

The specific blockers requested for this final rerun are closed:

- Control-character scalars: `rewrite-in-rust/fixtures/ustx_project_export_core.jsonl:2` includes `mix\n\t\r` and `line\nbreak`; the golden YAML pins PyYAML output at `rewrite-in-rust/fixtures/ustx_project_export_core/edge_project.ustx:298` and `rewrite-in-rust/fixtures/ustx_project_export_core/edge_project.ustx:322`. Rust now handles double-quoted control characters at `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:250` through `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:278` and multiline single-quoted scalars at `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:243` through `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:247`.
- Tempo exponent formatting: `rewrite-in-rust/fixtures/ustx_project_export_core.jsonl:1` uses `tempo: 1e-7`, and the golden YAML records `bpm: 1.0e-07` at `rewrite-in-rust/fixtures/ustx_project_export_core/empty_project.ustx:7`. Rust normalizes exponent float text at `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:193` through `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:217`, with direct unit coverage at `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:771` through `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:779`.
- The narrowed project-export boundary remains `save_ustx(..., rmvpe_result=None)` per `rewrite-in-rust/records/0042-confirm-ustx-project-export-boundary.md:18`, and rollback remains legacy-owned at `rewrite-in-rust/manifest.yaml:917`.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_ustx_project_export_core.py`: pass
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml ustx_project`: pass, 5 tests
- `uv run python -c 'import yaml; values=["line\nbreak","mix\n\t\r"]; floats=[1e-7,1e20]; ...'`: pass; confirmed PyYAML emits `lyric: 'line\n\n  break'`, `lyric: "mix\\n\\t\\r"`, `bpm: 1.0e-07`, and `bpm: 1.0e+20`.
- `uv run python -c 'import yaml; values=["1.0e-7","1.0e+20","0b101","-x","?x"]; ...'`: pass; confirmed the remaining dynamic-scalar examples above.

## Residual Risk

The fixture path proves parity for empty projects, invalid-note skipping, stable onset ordering in the selected case, half-even tick conversion, minimum duration, tone clamping, fallback lyric, selected YAML-sensitive punctuation, control-character lyrics, exponent-form tempo metadata, voice-part duration, and `curves: []`.

Behavior remains unapproved because accepted dynamic string scalars can still diverge from PyYAML, including semantically type-changing cases. Non-finite tempo and tick overflow are error/tracing concerns outside this behavior pass. RMVPE-derived pitch curves remain out of scope for this unit.

## Promotion Note

This behavior role blocks coordinator verification for `ustx_project_export_core`. Do not mark the manifest `verified` until the remaining dynamic-scalar parity gap is fixed or explicitly narrowed and fixture-proven.
