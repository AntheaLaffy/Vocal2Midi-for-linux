# hfa_pyyaml_safe_load_contract - error_tracing_reviewer rerun

Date: 2026-07-18
Decision: pass

## Findings

No blocking findings.

- Severity: low
- Location: rewrite-in-rust/rust/crates/v2m-core/src/hfa_pyyaml.rs:364
- Issue: Duplicate-anchor diagnostics now depend on parsed `saphyr-parser`
  anchor events instead of the removed raw-input pre-scan. This closes the
  original false-positive blocker for repeated `&name` text in comments,
  quoted scalars, and block scalars.
- Evidence: Parsed scalar/sequence/mapping events call
  `register_anchor_name` only when `anchor > 0` at
  rewrite-in-rust/rust/crates/v2m-core/src/hfa_pyyaml.rs:283,
  rewrite-in-rust/rust/crates/v2m-core/src/hfa_pyyaml.rs:298, and
  rewrite-in-rust/rust/crates/v2m-core/src/hfa_pyyaml.rs:323. The duplicate
  error still uses PyYAML-style first/second marks at
  rewrite-in-rust/rust/crates/v2m-core/src/hfa_pyyaml.rs:370. The regression
  fixtures at rewrite-in-rust/fixtures/hfa_pyyaml_safe_load_contract.jsonl:47,
  rewrite-in-rust/fixtures/hfa_pyyaml_safe_load_contract.jsonl:48, and
  rewrite-in-rust/fixtures/hfa_pyyaml_safe_load_contract.jsonl:49 load
  successfully, while the real duplicate-anchor case at
  rewrite-in-rust/fixtures/hfa_pyyaml_safe_load_contract.jsonl:25 keeps the
  composer error and marks. This matches PyYAML's event-anchor check at
  third_party/sources/pyyaml-6.0.3/lib/yaml/composer.py:71.
- Required fix: None for this role.

- Severity: low
- Location: rewrite-in-rust/rust/crates/v2m-core/src/hfa_pyyaml.rs:496
- Issue: The review-regression constructor errors for non-ASCII `!!binary` and
  invalid timestamp ranges now preserve the expected PyYAML phase/class/message
  shape.
- Evidence: `!!binary` performs PyYAML's ASCII conversion step before base64
  decode at rewrite-in-rust/rust/crates/v2m-core/src/hfa_pyyaml.rs:496,
  matching third_party/sources/pyyaml-6.0.3/lib/yaml/constructor.py:294. The
  non-ASCII binary fixture at
  rewrite-in-rust/fixtures/hfa_pyyaml_safe_load_contract.jsonl:55 records a
  `yaml.constructor.ConstructorError` with line/column mark. Timestamp parsing
  accepts no-space timezone offsets and validates invalid time fields through
  `ValueError` at rewrite-in-rust/rust/crates/v2m-core/src/hfa_pyyaml.rs:1522,
  rewrite-in-rust/rust/crates/v2m-core/src/hfa_pyyaml.rs:1631, and
  rewrite-in-rust/rust/crates/v2m-core/src/hfa_pyyaml.rs:1682; fixtures at
  rewrite-in-rust/fixtures/hfa_pyyaml_safe_load_contract.jsonl:50 through
  rewrite-in-rust/fixtures/hfa_pyyaml_safe_load_contract.jsonl:54 lock those
  shapes.
- Required fix: None for this role.

- Severity: low
- Location: rewrite-in-rust/rust/crates/v2m-core/src/hfa_pyyaml.rs:73
- Issue: Structured error projection remains diagnosable for the reviewed seam:
  phase, Python class name, message, context/problem/note, context/problem
  marks, file-open fields, decode fields, and temporary-path normalization are
  retained.
- Evidence: The Rust projection normalizes messages and marks at
  rewrite-in-rust/rust/crates/v2m-core/src/hfa_pyyaml.rs:73 and
  rewrite-in-rust/rust/crates/v2m-core/src/hfa_pyyaml.rs:956. File-open and
  UTF-8 decode errors are projected separately at
  rewrite-in-rust/rust/crates/v2m-core/src/hfa_pyyaml.rs:1113 and
  rewrite-in-rust/rust/crates/v2m-core/src/hfa_pyyaml.rs:1146. The Python
  checker projects the same fields at
  rewrite-in-rust/bootstrap/check_hfa_pyyaml_safe_load_contract.py:156, with
  missing-file, directory, and invalid-UTF-8 fixtures at
  rewrite-in-rust/fixtures/hfa_pyyaml_safe_load_contract.jsonl:44 through
  rewrite-in-rust/fixtures/hfa_pyyaml_safe_load_contract.jsonl:46.
- Required fix: None for this role.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_hfa_pyyaml_safe_load_contract.py`:
  passed; validated 56 fixtures against Python 3.12/PyYAML 6.0.3.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_pyyaml`:
  passed; the fixture parity test consumed the 56-case JSONL projection.
- `uv run python - <<'PY' ...`: targeted PyYAML probe confirmed repeated
  duplicate-looking anchor text in comments, quoted scalars, and block scalars
  loads successfully; real duplicate anchors still raise
  `yaml.composer.ComposerError` with first/second occurrence marks; non-ASCII
  `!!binary` raises `yaml.constructor.ConstructorError`; invalid timestamp hour
  raises `builtins.ValueError`.
- `rg`/source inspection: checked duplicate-anchor registration, parser error
  mapping, constructor error branches, file/decode projection, path
  normalization, fixture cases, and PyYAML reference sources.

## Residual Risk

This pass is fixture-bound to the current 56-case matrix. It does not prove
every possible `saphyr-parser::ScanError` text maps to the exact PyYAML
exception class/message, nor does it cover permission-denied filesystem errors,
large alias graphs, deep nesting, or scanner/parser resource limits. Those are
acceptable residual risks while Python/PyYAML remains the production owner and
resource-limit policy is deferred to a future production-facing promotion plan.

## Promotion Note

Writer/reviewer separation is preserved: this rerun did not edit production
Rust/Python code, fixtures, manifest, dependencies, bootstrap docs, or records.
The original error/tracing blocker is closed, and this role does not block a
coordinator state update for `hfa_pyyaml_safe_load_contract`. The coordinator
still needs the separate behavior rerun outcome before marking the unit
`verified`.
