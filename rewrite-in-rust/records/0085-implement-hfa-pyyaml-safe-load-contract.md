# 0085 - Implement HFA PyYAML Safe Load Contract

Date: 2026-07-18

## Context

Records 0083 and 0084 closed the dependency/bootstrap path for
`hfa_pyyaml_safe_load_contract`: the unit has a Python 3.12/PyYAML 6.0.3
fixture projection and a pinned lower-layer parser strategy using
`saphyr-parser` 0.0.11. The remaining writer task was to implement the Rust
library seam without changing production config loading.

## Implementation

Added `v2m-core::hfa_pyyaml` as an independent Rust library module.

The implementation uses `saphyr-parser` only for YAML syntax events, tags,
aliases, and source spans. The adapter owns the PyYAML compatibility behavior:

- UTF-8 file-open and decode error projection before YAML parsing;
- single-document composition, duplicate anchor handling, alias identity, and
  recursive aliases;
- YAML 1.1 implicit resolution for PyYAML SafeLoader scalars;
- SafeConstructor-compatible values for null, booleans, integers, floats,
  strings, bytes, dates, datetimes, lists, tuples, dictionaries, sets, omap,
  pairs, aliases, merges, and duplicate replacement;
- unsupported tag rejection for local, global, and Python-specific tags;
- structured scanner/parser/composer/constructor/file/decode errors matching
  the tagged fixture surface.

The Rust test consumes
`rewrite-in-rust/fixtures/hfa_pyyaml_safe_load_contract.jsonl` directly and
compares the Rust projection with the Python-generated expected payloads.

## Boundary

This is a writer-owned `reimplemented` state only. Python `config_utils.load_yaml`
and PyYAML 6.0.3 remain production runtime owners. No bridge, Python import,
model config route, GUI, Web, CLI, export, or inference caller changed.

Resource-limit fixtures for large aliases, deeply nested inputs, and
scanner/parser limits remain deferred to production-facing promotion planning as
recorded in 0084.

## Verification

Current passing evidence:

```bash
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_pyyaml
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml
cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check
cargo clippy --manifest-path rewrite-in-rust/rust/Cargo.toml --all-targets --all-features -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --manifest-path rewrite-in-rust/rust/Cargo.toml --no-deps
uv run python rewrite-in-rust/bootstrap/check_hfa_pyyaml_safe_load_contract.py
uv run python -m py_compile inference/HubertFA/tools/config_utils.py rewrite-in-rust/bootstrap/check_hfa_pyyaml_safe_load_contract.py
uv run python scripts/audit_vendored_sources.py
git diff --check
```

The focused Rust gate passes the tagged parity test. The full Rust
workspace passes 115 `v2m-core` tests plus five quant bridge tests. Independent
behavior and error/tracing reviews are still required before this unit may be
marked `verified`.

## Reversal

Rollback remains keeping Python `config_utils.load_yaml` and PyYAML 6.0.3 as
runtime owners. Since this record adds only an independent Rust library seam and
tests, runtime reversal does not require a production route change.
