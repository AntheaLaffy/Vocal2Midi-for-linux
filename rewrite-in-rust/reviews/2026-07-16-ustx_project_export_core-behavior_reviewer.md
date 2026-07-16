# ustx_project_export_core - behavior_reviewer

Date: 2026-07-16
Decision: fail

## Findings

- Severity: medium
- Location: rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:197
- Issue: Dynamic YAML scalars do not match the legacy PyYAML behavior for valid project names and lyrics outside the current fixtures. Python writes the project with `yaml.safe_dump(..., allow_unicode=True, sort_keys=False)` and feeds dynamic scalars from `filepath.stem` plus `note.lyric or "a"` in `inference/API/ustx_api.py:369`. Rust writes `project_name` and `lyric` through `yaml_scalar` at `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:70`, `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:90`, and `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:149`, but `yaml_scalar` only quotes empty strings and a small boolean/null-like word set before returning most values unchanged. That is not exact YAML parity for public string inputs.
- Evidence: `uv run python -c "import yaml; values=['a: b','#tag','line\\nbreak','yes','1','[x]','{x: y}','off','你']; [print(repr(v), '->', repr(yaml.safe_dump({'lyric': v}, allow_unicode=True, sort_keys=False).strip())) for v in values]"` shows PyYAML emits quoted or block-safe forms for values such as `a: b`, `#tag`, `yes`, `1`, `[x]`, and `{x: y}`. The Rust code would emit these as plain scalars because none of them match its quoted word list except `off`. The current fixture table at `rewrite-in-rust/fixtures/ustx_project_export_core.jsonl:1` covers empty, `half`, `hi`, and UTF-8 `你` lyrics plus simple stems only, so the checker and Rust tests do not exercise this mismatch.
- Required fix: before behavior approval, either make Rust scalar rendering match PyYAML for all dynamic `project_name` and lyric values accepted by `save_ustx`, or explicitly narrow the unit's accepted scalar domain in the bootstrap/manifest and add golden Python fixtures that prove the intended restriction. At minimum, add parity fixtures for punctuation, numeric-looking text, YAML collection-looking text, and a punctuation-bearing output stem.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_ustx_project_export_core.py`: pass
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml ustx_project`: pass, 3 tests
- `rg -n "render_ustx_project|UstxProject|ustx_project" . -S`: confirmed no production Python runtime bridge; references are limited to rewrite artifacts, Rust module exposure, tests, and docs.
- PyYAML scalar probe above: pass; confirms legacy quoting behavior for dynamic scalar examples not covered by fixtures.

## Residual Risk

The covered fixtures prove parity for empty projects, invalid-note skipping, stable onset ordering in the selected case, half-even tick conversion, minimum duration, lower and upper tone clamping, fallback lyric, UTF-8 lyric output, root and tempo metadata, voice-part duration, and `curves: []`. Behavior remains unapproved because exact YAML output for valid dynamic string values is broader than the current fixture set and currently diverges from PyYAML.

Error behavior for non-finite tempo and tick overflow belongs to a separate error/tracing review. RMVPE-derived pitch curves are intentionally out of scope for this unit and remain owned by `ustx_pitch_curve_core`.

## Promotion Note

This behavior role blocks promotion or coordinator verification until the dynamic YAML scalar parity gap is fixed or the public input domain is explicitly narrowed and fixture-proven. Rollback is intact: `rewrite-in-rust/manifest.yaml:917` keeps `inference.API.ustx_api.save_ustx` as the runtime owner, and `rewrite-in-rust/rust/crates/v2m-core/src/lib.rs:3` states the Rust crate is not wired into the Python runtime.
