# 0082 - Close HFA Config Validation Gate

Date: 2026-07-18

## Context

Record 0079 split the mixed HFA config file contract into a deterministic
`hfa_config_validation_core` validator and a separate planned
`hfa_pyyaml_safe_load_contract` loader contract. Record 0081 implemented the
validation child behind the independent Rust library seam without adding a YAML
parser, bridge, or production caller route.

The first independent reviews found no blocker, but dependency/bootstrap and
behavior both left a low fixture-coverage follow-up: the seam claimed
JSON-compatible injected loader values while the table did not explicitly cover
top-level numeric values or dictionary-entry float/mapping values. The
dependency/bootstrap review also found that the sibling loader dependency record
used `confirmed` wording while the manifest and bootstrap correctly kept that
loader unit planned and provisional.

## Follow-Up Closure

The fixture table now has 38 Python 3.12 golden cases. The added rows lock:

- top-level integer and float loader values raising exact Python
  `AttributeError` messages through `.get`;
- dictionary-entry float and nested mapping values raising exact Python
  `TypeError` messages through `Path / value`.

`rewrite-in-rust/dependencies/hfa_pyyaml_safe_load_contract.yaml` now marks the
loader child inventory impact as `provisional`, matching manifest status and
record 0079. It remains a real future unit, but is not writer-ready until it has
a tagged value/error fixture matrix and a selected parser/event plus
compatibility-adapter strategy.

## Review Evidence

Current passing evidence:

- dependency/bootstrap rerun2:
  `rewrite-in-rust/reviews/2026-07-18-hfa_config_validation_core-dependency_bootstrap_reviewer-rerun2.md`
- behavior rerun:
  `rewrite-in-rust/reviews/2026-07-18-hfa_config_validation_core-behavior_reviewer-rerun.md`
- error/tracing report:
  `rewrite-in-rust/reviews/2026-07-18-hfa_config_validation_core-error_tracing_reviewer.md`

The initial behavior and dependency/bootstrap `pass-with-followups` reports and
the first dependency/bootstrap rerun failure remain as audit evidence. That
first rerun failed only because the manifest cited this close-gate record before
it existed; rerun2 passed after the record was created and did not reopen a
dependency, seam, fixture, or crate-reuse finding.

## Decision

Accept `hfa_config_validation_core` as verified for the current legacy-owned,
no-bridge Rust library seam.

The verified unit preserves:

- vocab then config existence order and exact assertion path text;
- one loader call after both existence checks and direct loader error
  propagation;
- Python f-string suffix rendering for default, string, empty, and `None`
  suffixes;
- JSON-compatible injected shape behavior for top-level mapping and dynamic
  `.get` failures, `dictionaries.items()` failures, `None` skipping, and
  dictionary path checks;
- exact `AssertionError`, `AttributeError`, `TypeError`, and opaque loader
  exception projections;
- stateless repeated-call behavior;
- the legacy fact that `config.<suffix>` contents are not parsed by
  `check_configs`.

The unit does not parse YAML, claim PyYAML `safe_load` parity, route production
callers, change ONNX/model config loading, or replace `InferenceOnnx.load_config`.

## Verification

```bash
uv run python rewrite-in-rust/bootstrap/check_hfa_config_validation_core.py
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_config_validation_core
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml
cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check
cargo clippy --manifest-path rewrite-in-rust/rust/Cargo.toml --all-targets --all-features -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --manifest-path rewrite-in-rust/rust/Cargo.toml --no-deps
uv run python -m py_compile inference/HubertFA/tools/config_utils.py rewrite-in-rust/bootstrap/check_hfa_config_validation_core.py
uv run python scripts/audit_vendored_sources.py
git diff --check
```

The Python checker validates all 38 cases. The focused Rust gate passes one
targeted parity test, and the full Rust workspace passes 114 `v2m-core` tests
plus five quant bridge tests.

## Residual Risk

Full PyYAML 6.0.3 loader parity remains unproven and intentionally owned by the
planned `hfa_pyyaml_safe_load_contract`. Promotion must define the tagged
non-JSON value model, parser/constructor/error adapter, Python-facing exception
reconstruction, non-UTF-8 path policy, caller routing, and rollback.

## Reversal

Rollback remains keeping Python `config_utils.check_configs`,
`config_utils.load_yaml`, `InferenceOnnx.load_config`, and HFA API loading as
runtime owners. No production Python import, model path, GUI, Web, CLI, export,
or bridge route changed.
