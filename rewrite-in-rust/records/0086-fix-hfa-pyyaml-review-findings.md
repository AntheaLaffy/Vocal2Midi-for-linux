# 0086 - Fix HFA PyYAML Review Findings

Date: 2026-07-18

## Context

The first independent behavior and error/tracing reviews for
`hfa_pyyaml_safe_load_contract` failed the writer implementation from record
0085. Both reviews identified the duplicate-anchor pre-scan as a blocker, and
the behavior review also found timestamp and binary constructor parity gaps.

The unit remains `reimplemented`; this record documents writer follow-up work
before review reruns.

## Fixes

The duplicate-anchor check no longer scans the raw input before parsing. It now
runs only when `saphyr-parser` emits a parsed event with an anchor id, then
recovers the adjacent source anchor name for PyYAML-compatible duplicate-name
errors. Anchor-shaped text inside comments, quoted scalars, and block scalars
therefore remains ordinary scalar/comment content.

Timestamp parsing now follows PyYAML's resolver/constructor shape more closely:

- no-space timezone offsets such as `12:34:56-05:30` resolve as datetimes;
- minute and second fields must be two digits for timestamp resolution;
- invalid hour, minute, and second ranges raise Python-style `ValueError`
  messages instead of constructing invalid datetimes.

`!!binary` construction now performs PyYAML's ASCII conversion step before
base64 decoding. Non-ASCII scalar content raises a constructor error instead of
silently decoding to empty bytes.

## Fixtures

`rewrite-in-rust/fixtures/hfa_pyyaml_safe_load_contract.jsonl` was expanded from
47 to 56 Python-generated rows. Added coverage locks:

- repeated `&dup` text in comments, quoted scalars, and block scalars;
- no-space timestamp timezone offsets;
- short minute/second timestamp strings;
- invalid hour, minute, and second `ValueError` paths;
- non-ASCII `!!binary` constructor rejection.

## Verification

Current focused evidence:

```bash
uv run python rewrite-in-rust/bootstrap/check_hfa_pyyaml_safe_load_contract.py
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_pyyaml
```

The Python checker validates all 56 cases, and the Rust fixture parity test
matches the expanded projection.

## Review State

The original failed reports remain durable audit evidence:

- `rewrite-in-rust/reviews/2026-07-18-hfa_pyyaml_safe_load_contract-behavior_reviewer.md`
- `rewrite-in-rust/reviews/2026-07-18-hfa_pyyaml_safe_load_contract-error_tracing_reviewer.md`

Behavior and error/tracing reruns are required before the coordinator may mark
the unit `verified`.

## Reversal

Rollback remains keeping Python `config_utils.load_yaml` and PyYAML 6.0.3 as
runtime owners. No production caller route changed.
