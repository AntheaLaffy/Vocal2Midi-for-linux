# ustx_project_export_core - behavior_reviewer rerun

Date: 2026-07-16
Decision: fail

## Findings

- Severity: medium
- Location: rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:197
- Issue: The YAML scalar fix closes the reviewed punctuation/boolean-like cases, but it still does not match legacy PyYAML for all dynamic project-name and lyric strings accepted by `save_ustx(..., rmvpe_result=None)`. Legacy writes the project through `yaml.safe_dump(project, f, allow_unicode=True, sort_keys=False)` at `inference/API/ustx_api.py:460` after accepting `filepath.stem` and `note.lyric or "a"` at `inference/API/ustx_api.py:413` and `inference/API/ustx_api.py:379`. Rust sends those same dynamic fields through `yaml_scalar` at `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:69`, `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:89`, and `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:148`, but `needs_single_quoted_scalar` has no branch for embedded newlines or tabs. A lyric such as `line\nbreak` would therefore be emitted into the middle of a plain scalar, while PyYAML emits a quoted multiline scalar.
- Evidence: `uv run python -c "import yaml; values=['line\\nbreak','a\\tb',' spaced','trail ','1','[x]']; [print(repr(v), '=>', repr(yaml.safe_dump({'lyric': v}, allow_unicode=True, sort_keys=False))) for v in values]"` shows PyYAML quotes or escapes embedded newline/tab values. Code inspection of `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:210` through `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:270` shows the Rust predicate quotes leading/trailing whitespace, YAML booleans/nulls, YAML collection starters, colon/hash cases, numbers, dates, and times, but not embedded newline or tab. The bootstrap/manifest do not explicitly reject or validate such dynamic strings.
- Required fix: before behavior approval, either make `yaml_scalar` handle PyYAML-compatible control-character and multiline dynamic strings for project names and lyrics, or explicitly narrow the accepted scalar domain in the unit boundary and add fixture evidence that the narrowed domain matches legacy for accepted inputs.

## Resolved Scope Check

The previous blocking examples requested for this rerun are now covered and match:

- `rewrite-in-rust/fixtures/ustx_project_export_core.jsonl:2` includes stem `a: b` and lyrics `yes`, `#tag`, and `{x: y}`.
- `rewrite-in-rust/fixtures/ustx_project_export_core/edge_project.ustx:1`, `rewrite-in-rust/fixtures/ustx_project_export_core/edge_project.ustx:242`, `rewrite-in-rust/fixtures/ustx_project_export_core/edge_project.ustx:274`, `rewrite-in-rust/fixtures/ustx_project_export_core/edge_project.ustx:298`, and `rewrite-in-rust/fixtures/ustx_project_export_core/edge_project.ustx:322` show the PyYAML golden output quotes those values.
- `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:677` through `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:686` add Rust unit coverage for `yes`, `1`, `a: b`, `#tag`, `[x]`, and `{x: y}`.
- `uv run python rewrite-in-rust/bootstrap/check_ustx_project_export_core.py` proves the updated golden YAML still matches legacy Python `save_ustx(..., rmvpe_result=None)`.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml ustx_project` proves the Rust renderer matches that same fixture table.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_ustx_project_export_core.py`: pass
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml ustx_project`: pass, 3 tests
- `rg -n "a: b|#tag|yes|\\{x: y\\}|yaml_scalar_quotes" rewrite-in-rust/fixtures/ustx_project_export_core.jsonl rewrite-in-rust/fixtures/ustx_project_export_core/edge_project.ustx rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs rewrite-in-rust/reviews/2026-07-16-ustx_project_export_core-behavior_reviewer.md`: pass; confirms the rerun-target scalar examples are now fixture/test covered.
- `uv run python -c "import yaml; values=['line\\nbreak','a\\tb',' spaced','trail ','1','[x]']; [print(repr(v), '=>', repr(yaml.safe_dump({'lyric': v}, allow_unicode=True, sort_keys=False))) for v in values]"`: pass; confirms legacy PyYAML quotes/escapes newline and tab strings that remain uncovered by the Rust scalar predicate.

## Residual Risk

The current fixture path proves parity for empty projects, invalid-note skipping, stable onset ordering in the selected case, half-even tick conversion, minimum duration, lower and upper tone clamping, fallback lyric, UTF-8 lyric output, the rerun-target YAML-sensitive values, tempo metadata, voice-part duration, and `curves: []`.

Behavior remains unapproved because the Rust scalar renderer still has accepted dynamic string cases that can diverge from legacy PyYAML. Non-finite tempo and tick overflow remain error/tracing concerns outside this behavior rerun. RMVPE-derived pitch curves remain out of scope for this unit.

## Promotion Note

This behavior rerun still blocks coordinator verification for `ustx_project_export_core`. Rollback is intact: `rewrite-in-rust/manifest.yaml:917` keeps `inference.API.ustx_api.save_ustx` as the runtime owner, and `rewrite-in-rust/rust/crates/v2m-core/src/lib.rs:3` states the Rust crate is not wired into the Python runtime.
