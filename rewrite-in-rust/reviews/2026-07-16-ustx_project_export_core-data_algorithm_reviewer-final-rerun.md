# ustx_project_export_core - data_algorithm_reviewer

Date: 2026-07-16
Decision: fail

## Findings

- Severity: medium
- Location: rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:343
- Issue: Exponent-form string scalars can still change YAML data type. The
  latest fix correctly renders numeric tempo floats such as `1e-7` as PyYAML
  float text, but project names and lyrics are arbitrary strings and are routed
  through the YAML scalar detector. PyYAML quotes string values such as
  `1.0e-07`, while the current Rust `looks_like_yaml_number` only recognizes
  digits, `.`, `_`, and `0x` hex forms. Rust would therefore emit an accepted
  lyric or project name `1.0e-07` as a plain scalar, which PyYAML resolves as a
  float instead of a string.
- Evidence: Legacy accepts note lyrics through `note.lyric or "a"` at
  `inference/API/ustx_api.py:384`, project and voice-part names from
  `filepath.stem` at `inference/API/ustx_api.py:414` and
  `inference/API/ustx_api.py:447`, and delegates serialization to
  `yaml.safe_dump(..., allow_unicode=True, sort_keys=False)` at
  `inference/API/ustx_api.py:460`. Rust renders the same dynamic fields through
  `yaml_scalar` at `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:70`,
  `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:90`, and
  `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:149`. The scalar
  detector calls `looks_like_yaml_number` at
  `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:336`, but that
  helper rejects any `e`/`E` exponent form before returning at
  `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:363`. A focused
  PyYAML probe emitted `- '1.0e-07'`, `- '1.0e+20'`, `- '-1.0e-07'`, and
  `- '+1.0e+20'` for string list items, and `yaml.safe_load("x: 1.0e-07\n")`
  returned a `float`.
- Required fix: Extend the scalar hazard detector to single-quote PyYAML
  resolver-sensitive exponent float strings, including signed forms with a
  decimal mantissa and signed exponent, or replace the ad hoc detector with a
  fixture-backed PyYAML resolver subset. Add at least one golden USTX fixture
  for an exponent-looking string lyric or stem, such as `1.0e-07`, before
  rerunning this role.

## Checked Scope

- The prior control-character scalar blocker is closed for the covered data
  shape. The fixture table includes `mix\n\t\r` and `line\nbreak` lyrics at
  `rewrite-in-rust/fixtures/ustx_project_export_core.jsonl:2`; the golden YAML
  matches PyYAML's escaped and multiline forms at
  `rewrite-in-rust/fixtures/ustx_project_export_core/edge_project.ustx:298` and
  `rewrite-in-rust/fixtures/ustx_project_export_core/edge_project.ustx:322`;
  Rust helper coverage asserts those forms at
  `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:764`.
- The prior finite exponent tempo blocker is closed for numeric tempo fields.
  Rust normalizes exponent float text in `python_float_repr` at
  `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:193`, uses that
  text for root `bpm` and `tempos[0].bpm` at
  `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:67`,
  `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:80`, and
  `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:84`, and tests
  `1e-7`, `1e-6`, `1.23e-7`, and `1e20` at
  `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:772`. The
  `empty_project` fixture pins `tempo: 1e-7` at
  `rewrite-in-rust/fixtures/ustx_project_export_core.jsonl:1`, with PyYAML
  output `1.0e-07` at
  `rewrite-in-rust/fixtures/ustx_project_export_core/empty_project.ustx:7` and
  `rewrite-in-rust/fixtures/ustx_project_export_core/empty_project.ustx:227`.
- Core note data algorithms remain acceptable for the narrowed
  `rmvpe_result=None` surface. Legacy filtering, stable sorting, tick
  conversion, minimum duration, tone clamp, lyric fallback, and voice-part
  duration are at `inference/API/ustx_api.py:30`,
  `inference/API/ustx_api.py:370`, `inference/API/ustx_api.py:374`,
  `inference/API/ustx_api.py:376`, `inference/API/ustx_api.py:377`,
  `inference/API/ustx_api.py:384`, and `inference/API/ustx_api.py:446`. Rust
  mirrors those with finite filtering, stable sorting, half-even tick rounding,
  min duration, tone clamp, and max-end voice duration at
  `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:54`,
  `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:126`,
  `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:128`,
  `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:132`,
  `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:160`,
  `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:164`,
  `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:174`, and
  `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:65`.
- The fixed project skeleton remains within the confirmed boundary:
  `save_ustx(..., rmvpe_result=None)` with one voice part, empty curves, and no
  pitch-curve generation is documented at
  `rewrite-in-rust/bootstrap/ustx_project_export_core.md:6` and
  `rewrite-in-rust/bootstrap/ustx_project_export_core.md:91`. The checker also
  asserts one voice part and `curves: []` at
  `rewrite-in-rust/bootstrap/check_ustx_project_export_core.py:102`.
- Complexity remains aligned: both paths perform one filter pass, one stable
  note sort, one render pass over valid notes, and fixed-size project skeleton
  emission. I found no algorithmic complexity regression in this unit.

## Checks

- `PYTHONDONTWRITEBYTECODE=1 uv run python rewrite-in-rust/bootstrap/check_ustx_project_export_core.py`: failed before running the checker because the sandbox could not create a temporary file under `/home/fuurin/.cache/uv`.
- `uv run python rewrite-in-rust/bootstrap/check_ustx_project_export_core.py`: pass.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml ustx_project`: pass; 5 tests passed.
- `uv run python -c "import yaml; vals=['line\\nbreak','tab\\tchar','carriage\\rreturn','mix\\n\\t\\r','yes','1','a: b','#tag','{x: y}','你']; print(yaml.safe_dump({'values': vals, 'bpm': 1e-7, 'tempo_big': 1e20}, allow_unicode=True, sort_keys=False), end='')"`: pass; confirmed PyYAML control-character scalar styles and numeric exponent float spelling.
- `uv run python -c "import yaml; vals=['1e3','1e-7','1.0e-07','1.0e+20','-1.0e-07','+1.0e+20']; print(yaml.safe_dump({'values': vals}, allow_unicode=True, sort_keys=False), end=''); print('loads:', [(v, type(yaml.safe_load('x: '+v+'\\n')['x']).__name__, yaml.safe_load('x: '+v+'\\n')['x']) for v in vals])"`: pass; confirmed PyYAML quotes exponent-float-looking strings with decimal mantissas and that unquoted forms load as floats.

## Residual Risk

The fixture table is still intentionally small. It does not separately pin all
non-finite positions of the `(onset, offset, pitch)` tuple, negative-time
rounding, same-onset stable ordering, very large tick overflow, non-finite tempo
errors, or every PyYAML scalar style. The exponent-string scalar issue above is
a concrete data-shape mismatch, not only residual risk.

## Promotion Note

This role blocks promotion. The latest control-character scalar and finite
tempo exponent fixes are effective for their covered cases, and the core note
numeric algorithms are acceptable, but accepted exponent-looking string scalars
can still be serialized as YAML floats by the Rust renderer.
