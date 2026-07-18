# hfa_pyyaml_safe_load_contract - error_tracing_reviewer

Date: 2026-07-18
Decision: fail

## Findings

- Severity: medium
- Location: rewrite-in-rust/rust/crates/v2m-core/src/hfa_pyyaml.rs:193
- Issue: `load_yaml_str_inner` runs `duplicate_anchor_error` before the YAML parser, and that helper scans raw bytes for every `&name` occurrence without tracking YAML tokens, quotes, block scalars, or comments. This can synthesize a `yaml.composer.ComposerError` for valid YAML that only contains repeated anchor-shaped text in scalars or comments. PyYAML raises duplicate-anchor errors from parsed event anchors only.
- Evidence: `duplicate_anchor_error` treats any raw `&` followed by ASCII anchor characters as an anchor and returns a composer error on the second occurrence at rewrite-in-rust/rust/crates/v2m-core/src/hfa_pyyaml.rs:1239. PyYAML's duplicate-anchor check is token/event based at third_party/sources/pyyaml-6.0.3/lib/yaml/composer.py:71. `uv run python - <<'PY' ...` confirmed PyYAML accepts `a: '&dup'\nb: '&dup'\n` as `{'a': '&dup', 'b': '&dup'}` and accepts duplicate `&dup` text in comments. The current fixture table only proves the real duplicate-anchor error case at rewrite-in-rust/fixtures/hfa_pyyaml_safe_load_contract.jsonl:25 and does not cover quoted/comment false positives.
- Required fix: Replace the raw duplicate-anchor pre-scan with token-aware anchor detection, or make the compatibility scan fully YAML-lexical for comments, quoted scalars, block scalars, flow scalars, and plain scalar boundaries. Add golden cases proving repeated `&name` in quoted scalars and comments loads successfully, while a real duplicate anchor still raises the PyYAML-compatible composer error with first/second marks. Rerun the Python checker and the Rust `hfa_pyyaml` fixture test.

## Checks

- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_pyyaml`: passed; 1 `hfa_pyyaml` fixture parity test passed and 114 `v2m-core` tests were filtered out.
- `uv run python rewrite-in-rust/bootstrap/check_hfa_pyyaml_safe_load_contract.py`: passed; validated 47 fixtures against Python 3.12/PyYAML 6.0.3.
- `rg -n 'case_id.*(error|rejected|invalid|malformed|missing|directory|utf8|multi_document|undefined|duplicate_anchor)|"ok":false|map_parse_error|constructor_error\(|composer_error\(|scanner_error\(|parser_error\(|io_error\(|decode_error\(|unknown_tag_error\(' rewrite-in-rust/fixtures/hfa_pyyaml_safe_load_contract.jsonl rewrite-in-rust/rust/crates/v2m-core/src/hfa_pyyaml.rs`: inspected error fixture coverage and Rust error mapping branches.
- `uv run python - <<'PY' ...`: confirmed PyYAML loads repeated anchor-shaped text inside quoted scalars and comments without error.

## Residual Risk

The fixture matrix is strong for the currently enumerated file-open, UTF-8, scanner, parser, composer, constructor, unsupported-tag, path-normalization, and projection fields. It is still fixture-bound rather than exhaustive. Additional parser fallback errors, permission-denied file opens, more UTF-8 decoder edge cases, and resource-limit diagnostics for large alias/depth inputs remain unproven, which is acceptable only while this module stays outside the production runtime path.

## Promotion Note

Writer/reviewer separation is preserved: this review did not edit production Rust/Python code, fixtures, manifest, dependencies, bootstrap docs, or records. This role blocks coordinator state update for `hfa_pyyaml_safe_load_contract` until the duplicate-anchor false-positive is fixed and rerun evidence is attached. The production owner remains Python `config_utils.load_yaml` plus PyYAML 6.0.3.
