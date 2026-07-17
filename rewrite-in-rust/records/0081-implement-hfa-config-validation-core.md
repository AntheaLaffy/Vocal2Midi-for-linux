# 0081 - Implement HFA Config Validation Core

Date: 2026-07-17

## Context

Record 0079 re-cut the mixed HFA config unit into a writer-ready
`hfa_config_validation_core` and a separate planned
`hfa_pyyaml_safe_load_contract`. The validation child has a 34-case Python
fixture harness that monkeypatches `config_utils.load_yaml` and proves only the
deterministic `check_configs` phase after a loader result or loader error
exists.

## Implementation

Added `v2m-core::hfa_config` as an independent Rust library module with:

- `HfaConfigValue`, a JSON-compatible injected-loader value domain;
- `HfaConfigValidationError`, retaining Python exception type/message
  projections for assertion, attribute, type, and opaque loader failures;
- `check_configs_with_loader`, which mirrors vocab/config existence order,
  suffix text rendering at the call boundary, one loader call after existence
  checks, `vocab.get("dictionaries", [])`, `dictionaries.items()`, `None`
  skipping, dictionary path joining, absolute-path assertion messages, and
  stateless repeated calls.

The implementation does not parse YAML, add a YAML crate, introduce a bridge, or
change production Python imports. `load_yaml`, PyYAML SafeLoader behavior, ONNX
session/config assignment, and callers remain legacy-owned.

## Verification

```bash
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_config_validation_core
uv run python rewrite-in-rust/bootstrap/check_hfa_config_validation_core.py
```

## Review Required

The unit is `reimplemented`, not verified. Required review roles remain:

- dependency_bootstrap_reviewer
- stage_behavior_reviewer
- error_tracing_reviewer
