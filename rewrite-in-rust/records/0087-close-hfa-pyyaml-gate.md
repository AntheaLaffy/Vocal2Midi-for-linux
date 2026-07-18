# 0087 - Close HFA PyYAML Gate

Date: 2026-07-18

## Context

Record 0085 implemented `hfa_pyyaml_safe_load_contract` behind the independent
Rust library seam. The first behavior and error/tracing reviews failed with
concrete parity findings. Record 0086 fixed those findings and expanded the
fixture matrix from 47 to 56 Python-generated cases.

## Review Evidence

Required review roles are now satisfied for the current library seam:

- dependency/bootstrap:
  `rewrite-in-rust/reviews/2026-07-18-hfa_pyyaml_safe_load_contract-dependency_bootstrap_reviewer.md`
- behavior initial failure:
  `rewrite-in-rust/reviews/2026-07-18-hfa_pyyaml_safe_load_contract-behavior_reviewer.md`
- behavior rerun pass:
  `rewrite-in-rust/reviews/2026-07-18-hfa_pyyaml_safe_load_contract-behavior_reviewer-rerun.md`
- error/tracing initial failure:
  `rewrite-in-rust/reviews/2026-07-18-hfa_pyyaml_safe_load_contract-error_tracing_reviewer.md`
- error/tracing rerun pass:
  `rewrite-in-rust/reviews/2026-07-18-hfa_pyyaml_safe_load_contract-error_tracing_reviewer-rerun.md`

The failed reports remain audit evidence. The reruns confirm the review-driven
fixes for duplicate-anchor false positives, timestamp edge cases, and
non-ASCII binary constructor errors.

## Decision

Accept `hfa_pyyaml_safe_load_contract` as verified for the current
legacy-owned, no-bridge Rust library seam.

The verified unit preserves:

- UTF-8 file loading plus file-open and decode error projection;
- PyYAML 6.0.3 SafeLoader scalar resolution for the fixture-bound YAML 1.1
  surface;
- SafeConstructor-compatible tagged values for null, booleans, integers,
  floats, strings, bytes, dates, datetimes, lists, tuples, dictionaries, sets,
  omap, pairs, aliases, merges, duplicate replacement, and non-string keys;
- alias identity and recursive aliases;
- duplicate-anchor, undefined-alias, single-document, custom-tag, scanner,
  parser, composer, constructor, file, and decode error shapes covered by the
  fixture matrix;
- review-regression coverage for repeated anchor-shaped text in comments,
  quoted scalars, and block scalars; no-space timestamp offsets; invalid time
  ranges; and non-ASCII `!!binary` input.

This unit does not promote Rust as the runtime owner and does not change
`config_utils.load_yaml`, `InferenceOnnx.load_config`, GUI, Web, CLI, export,
or model inference routes.

## Verification

```bash
uv run python rewrite-in-rust/bootstrap/check_hfa_pyyaml_safe_load_contract.py
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_pyyaml
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml
cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check
cargo clippy --manifest-path rewrite-in-rust/rust/Cargo.toml --all-targets --all-features -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --manifest-path rewrite-in-rust/rust/Cargo.toml --no-deps
uv run python -m py_compile inference/HubertFA/tools/config_utils.py rewrite-in-rust/bootstrap/check_hfa_pyyaml_safe_load_contract.py
uv run python scripts/audit_vendored_sources.py
git diff --check
```

The Python checker validates all 56 cases. The full Rust workspace passes 115
`v2m-core` tests plus five quant bridge tests.

## Residual Risk

Resource-limit fixtures for large aliases, deeply nested inputs, and
scanner/parser limits remain deferred until production-facing promotion
planning. Additional arbitrary `saphyr-parser::ScanError` text mappings are not
exhaustively proven outside the fixture matrix. These are acceptable residual
risks while Python/PyYAML remains the runtime owner.

## Reversal

Rollback remains keeping Python `config_utils.load_yaml` and PyYAML 6.0.3 as
runtime owners. Because no production route changed, reversal does not require
bridge, import, GUI, Web, CLI, or inference changes.
