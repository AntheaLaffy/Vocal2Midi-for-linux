# ustx_project_export_core - data_algorithm_reviewer

Date: 2026-07-16
Decision: fail

## Findings

- Severity: medium
- Location: rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:197
- Issue: The updated YAML scalar renderer fixes the previously reported
  resolver and punctuation examples, but it still emits internal newline and
  tab characters as raw plain scalars. `save_ustx` accepts project names from
  `filepath.stem` and lyrics from `note.lyric or "a"` without validation, then
  delegates scalar styling to PyYAML. Those accepted string values are still in
  this unit's project-name and lyric surface.
- Evidence: Python builds note lyrics at `inference/API/ustx_api.py:384`,
  project and voice-part names at `inference/API/ustx_api.py:414` and
  `inference/API/ustx_api.py:447`, and serializes with
  `yaml.safe_dump(..., allow_unicode=True, sort_keys=False)` at
  `inference/API/ustx_api.py:460`. Rust renders those same fields through
  `yaml_scalar` at `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:70`,
  `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:90`, and
  `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:149`. The current
  predicate quotes empty strings, leading/trailing whitespace, resolver-like
  words, selected leading punctuation, `: `, ` #`, number-like strings, dates,
  and clock-like strings at
  `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:197` through
  `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:270`, but it has no
  internal `\n`, `\r`, or `\t` check before returning `value.to_string()` at
  `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:207`. The PyYAML
  probe emitted `line\nbreak` as a quoted multiline scalar and `tab\tbreak` as
  a double-quoted escaped scalar. A direct legacy `save_ustx` probe with a
  multiline stem and lyric wrote quoted multiline YAML. The fixture table now
  covers `a: b`, `yes`, `#tag`, and `{x: y}` at
  `rewrite-in-rust/fixtures/ustx_project_export_core.jsonl:2`, and the Rust
  scalar test covers those examples at
  `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:677`, but neither
  fixture nor test covers internal newlines or tabs.
- Required fix: Add a PyYAML-compatible policy for internal newline, carriage
  return, and tab scalars, or define and enforce a documented validation error
  that excludes them from `project_name` and `lyric`. Add golden Python fixtures
  for at least one internal newline and one tab-bearing scalar before rerunning
  this role.

- Severity: medium
- Location: rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:193
- Issue: Finite scientific-notation tempo values can be rendered as YAML
  strings rather than YAML floats. Legacy PyYAML emits exponent floats as
  `1.0e-07` / `1.0e+20`; Rust currently uses Rust debug float text, which emits
  `1e-7` / `1e20`. Under PyYAML's loader, the Rust form resolves as a string,
  while the legacy form resolves as a float.
- Evidence: Legacy stores tempo in root `bpm` and `tempos[0].bpm` as
  `float(tempo)` at `inference/API/ustx_api.py:420` and
  `inference/API/ustx_api.py:429`, then PyYAML serializes it at
  `inference/API/ustx_api.py:460`. Rust obtains `tempo_text` from
  `python_float_repr` at `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:67`
  and inserts it into both tempo fields at
  `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:80` and
  `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:84`.
  `python_float_repr` is `format!("{value:?}")` at
  `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:193`. The PyYAML
  probe printed `1.0e-07`, `1.0e-06`, and `1.0e+20`; the Rust float probe
  printed `1e-7`, `1e-6`, and `1e20`. A loader probe confirmed
  `yaml.safe_load("bpm: 1e-7\n")["bpm"]` is `str`, while
  `yaml.safe_load("bpm: 1.0e-07\n")["bpm"]` is `float`.
- Required fix: Render finite exponent tempos in a PyYAML-compatible float style
  or explicitly restrict tempo values to the fixed decimal range covered by
  fixtures. Add a golden fixture for a finite exponent tempo before rerunning
  this role.

## Checked Scope

- The original punctuation/resolver scalar failure is partially fixed for the
  covered values: `a: b`, `yes`, `#tag`, and `{x: y}` appear in the golden
  fixture at `rewrite-in-rust/fixtures/ustx_project_export_core.jsonl:2`, the
  expected YAML quotes them at
  `rewrite-in-rust/fixtures/ustx_project_export_core/edge_project.ustx:1`,
  `rewrite-in-rust/fixtures/ustx_project_export_core/edge_project.ustx:274`,
  `rewrite-in-rust/fixtures/ustx_project_export_core/edge_project.ustx:298`,
  and `rewrite-in-rust/fixtures/ustx_project_export_core/edge_project.ustx:322`,
  and the Rust fixture test passed.
- Numeric note conversion remains acceptable for the covered finite cases:
  legacy tick conversion is `int(round(seconds * tempo * 8.0))` at
  `inference/API/ustx_api.py:26`, Rust uses explicit half-even rounding at
  `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:160` and
  `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:174`, and the
  fixture exercises 0.5-tick, 2.5-tick, minimum-duration, low-clamp, high-clamp,
  and 61.5-pitch tie behavior.
- Filtering and ordering are acceptable for the narrowed surface: legacy
  `_finite_notes` filters non-finite onset, offset, and pitch plus
  `offset <= onset` at `inference/API/ustx_api.py:30`, Rust mirrors that at
  `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:153`, and Rust uses
  stable `sort_by` by onset at
  `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:54`, matching
  Python's stable `sorted(..., key=lambda n: n.onset)` at
  `inference/API/ustx_api.py:370`.
- Project structures remain acceptable for `rmvpe_result=None`: fixed
  expressions, track metadata, pitch/vibrato defaults, empty curves, empty wave
  parts, and voice-part duration are mirrored by the static Rust project body and
  pinned by `empty_project.ustx` and `edge_project.ustx`.
- Complexity is acceptable: both implementations do one filter pass, one stable
  note sort, one render pass, and fixed-size project skeleton emission.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_ustx_project_export_core.py`:
  pass
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml ustx_project`:
  pass, 3 tests
- `uv run python -c "import yaml; vals=['line\\nbreak','tab\\tbreak',' leading','trailing ','-word','?word','1e3','12:34','a#b','a #b']; print(yaml.safe_dump({'values': vals}, allow_unicode=True, sort_keys=False), end='')"`:
  pass; confirms PyYAML quotes or escapes internal newline and tab scalars.
- `uv run python -c "... save_ustx(... lyric='line\\nbreak') ..."`:
  pass; confirms the legacy USTX path writes multiline project names and lyrics
  with quoted multiline scalar style.
- `uv run python -c "import yaml; vals=[1e-7,1e-6,1e20,120.0,90.5,-1e-7]; print(yaml.safe_dump({'bpm': vals, 'tempos':[{'position':0,'bpm':1e-7}]}, allow_unicode=True, sort_keys=False), end='')"`:
  pass; confirms PyYAML exponent float spelling.
- `printf 'fn main(){for value in [1e-7f64,1e-6f64,1e20f64,120.0f64,90.5f64,-1e-7f64]{println!("{value:?}");}}' | rustc -o /tmp/v2m_float_repr_probe - && /tmp/v2m_float_repr_probe`:
  pass; confirms Rust debug float spelling differs for exponent values.
- `uv run python -c "import yaml; samples=['bpm: 1e-7\\n','bpm: 1.0e-07\\n','bpm: 1e20\\n','bpm: 1.0e+20\\n']; ..."`:
  pass; confirms PyYAML loads Rust-style exponent text as `str` and legacy
  PyYAML exponent text as `float`.

## Residual Risk

The fixture table is still small. It does not separately pin all non-finite
positions of the `(onset, offset, pitch)` tuple, negative-time rounding, very
large tick overflow, non-finite tempo behavior, or every PyYAML scalar style.
The newline/tab scalar and exponent-tempo issues above are concrete mismatches,
not just residual risk.

## Promotion Note

This role still blocks promotion. The previously failed punctuation scalar class
is partially fixed and the core note algorithms, project structures,
sorting/filtering, and complexity are acceptable for the covered
`rmvpe_result=None` fixtures. However, accepted public strings with internal
newlines or tabs and finite exponent tempos can still produce non-PyYAML-compatible
output, including YAML type changes for tempo.
