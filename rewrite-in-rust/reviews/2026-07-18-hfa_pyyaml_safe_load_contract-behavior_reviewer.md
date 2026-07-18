# hfa_pyyaml_safe_load_contract - behavior_reviewer

Date: 2026-07-18
Decision: fail

## Findings

- Severity: high
- Location: rewrite-in-rust/rust/crates/v2m-core/src/hfa_pyyaml.rs:193
- Issue: The Rust loader rejects valid YAML when duplicate-looking anchors occur inside comments or quoted scalars. `load_yaml_str_inner` runs `duplicate_anchor_error` before parsing, and that helper scans raw bytes for every `&name` without tracking YAML lexical context (`rewrite-in-rust/rust/crates/v2m-core/src/hfa_pyyaml.rs:1239`). PyYAML only checks duplicate anchors while composing parser events (`third_party/sources/pyyaml-6.0.3/lib/yaml/composer.py:63`), so comments and quoted scalar contents are not anchors.
- Evidence: `uv run python - <<'PY' ...` with `# &dup &dup\n` returned `OK None`, and with `'&dup &dup'\n` returned `OK '&dup &dup' str`. The current Rust pre-scan would see two `&dup` byte sequences before `saphyr-parser` can classify them as a comment/scalar and would return the duplicate-anchor composer error.
- Required fix: Remove the raw duplicate-anchor pre-scan or replace it with parser/token-aware duplicate-anchor detection. Add golden fixtures for duplicate real anchors, comment-contained ampersands, quoted scalar ampersands, and preferably block-scalar ampersands.

- Severity: high
- Location: rewrite-in-rust/rust/crates/v2m-core/src/hfa_pyyaml.rs:1540
- Issue: Timestamp resolution/construction does not match PyYAML. Rust accepts one-digit minute/second fields that PyYAML leaves as strings, rejects no-space timezone offsets that PyYAML accepts, and constructs out-of-range times instead of raising Python `ValueError`. The Rust path uses `parse_datetime_parts` for resolver classification (`rewrite-in-rust/rust/crates/v2m-core/src/hfa_pyyaml.rs:1439`), parses minute/second without width checks (`rewrite-in-rust/rust/crates/v2m-core/src/hfa_pyyaml.rs:1564`), requires whitespace before `+/-` timezone offsets (`rewrite-in-rust/rust/crates/v2m-core/src/hfa_pyyaml.rs:1596`), and validates only the date portion (`rewrite-in-rust/rust/crates/v2m-core/src/hfa_pyyaml.rs:1512`). PyYAML's resolver requires two-digit minute and second fields and allows zero whitespace before timezone offsets (`third_party/sources/pyyaml-6.0.3/lib/yaml/resolver.py:207`), then constructs with `datetime.datetime(...)`, which raises on invalid hour/minute/second ranges (`third_party/sources/pyyaml-6.0.3/lib/yaml/constructor.py:310`).
- Evidence: The legacy probe showed `v: 2024-02-29 12:34:56-05:30` loads as a Python `datetime`, `v: 2024-02-29 1:2:03` loads as a `str`, and `v: 2024-02-29 25:00:00` raises `builtins.ValueError hour must be in 0..23`. Fixture coverage currently has a spaced offset and valid one-digit hour/month/day only (`rewrite-in-rust/fixtures/hfa_pyyaml_safe_load_contract.jsonl:10`).
- Required fix: Mirror PyYAML's timestamp resolver regex and constructor validation for timezone spacing, field widths, and time ranges. Add fixtures for no-space timezone offsets, short minute/second strings, and invalid hour/minute/second errors.

- Severity: medium
- Location: rewrite-in-rust/rust/crates/v2m-core/src/hfa_pyyaml.rs:1635
- Issue: `!!binary` construction accepts non-ASCII scalar content that PyYAML rejects. Rust filters bytes down to ASCII base64 characters and silently drops non-ASCII bytes (`rewrite-in-rust/rust/crates/v2m-core/src/hfa_pyyaml.rs:1635`), while PyYAML first encodes the scalar as ASCII and raises `ConstructorError` on `UnicodeEncodeError` (`third_party/sources/pyyaml-6.0.3/lib/yaml/constructor.py:294`).
- Evidence: The legacy probe showed `!!binary 你好\n` raises `yaml.constructor.ConstructorError` with `failed to convert base64 data into ascii...`. The Rust path would feed an empty cleaned byte vector to its base64 decoder and return empty bytes rather than the constructor error.
- Required fix: Add the ASCII conversion check before base64 decoding and project the PyYAML-compatible constructor error. Add a non-ASCII `!!binary` fixture.

## Checks

- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_pyyaml`: passed; the current 47-case Rust fixture parity test is green.
- `uv run python rewrite-in-rust/bootstrap/check_hfa_pyyaml_safe_load_contract.py`: passed; the Python checker validated 47 PyYAML fixtures.
- `uv run python - <<'PY' ...`: passed; confirmed the legacy behavior for comment/quoted ampersands, timestamp offset/width/range cases, and non-ASCII `!!binary`.
- `rg -n "load_yaml_path|load_yaml_str|hfa_pyyaml|config_utils\\.load_yaml" ...`: passed for behavior boundary review; no production Python caller route was switched to Rust, and `config_utils.load_yaml` remains the runtime owner.
- `rg -n "&dup &dup|12:34:56-05:30|1:2:03|25:00:00|!!binary 你好" rewrite-in-rust/fixtures/hfa_pyyaml_safe_load_contract.jsonl ...`: passed; confirmed the failing examples are not covered by the current fixture matrix, except for the already-covered spaced timestamp offset.

## Residual Risk

The 47-case fixture matrix is useful and covers the intended main behaviors, but this review found concrete mismatches in valid YAML/comment lexical handling, timestamp resolver/constructor compatibility, and binary constructor behavior. Error/tracing quality is intentionally left to the separate reviewer, but these issues are behavior-impacting because they change accepted inputs, returned value types, or exception-vs-success outcomes.

Writer/reviewer separation is preserved: this report reviews the current writer-owned Rust implementation and does not edit production code, fixtures, manifest state, dependencies, bootstrap docs, or records.

## Promotion Note

This behavior role blocks coordinator promotion of `hfa_pyyaml_safe_load_contract` to `verified`. The coordinator can keep the unit in `reimplemented`, but should not treat behavior review as passed until the findings above are fixed and rerun with added fixtures.
