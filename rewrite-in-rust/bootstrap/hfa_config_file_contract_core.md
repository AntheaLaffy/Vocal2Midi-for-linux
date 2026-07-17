# hfa_config_file_contract_core Re-cut

## Decision

Split the provisional mixed unit into:

1. `hfa_config_validation_core`, a confirmed writer-ready library seam for
   `check_configs` over an injected loader outcome; and
2. `hfa_pyyaml_safe_load_contract`, an explicit planned unit retaining the full
   `load_yaml` plus PyYAML 6.0.3 `safe_load` value/tag/error surface.

The first child is not a substitute for the second. It proves only the
deterministic config validation phase after a loader outcome has been supplied.
Production composition and full YAML behavior remain Python-owned.

## Why The Split Is Required

`check_configs` has a small ordered control flow: render `vocab.<suffix>` and
`config.<suffix>`, require both in that order, call `load_yaml` once for vocab,
perform Python `.get(..., [])` then `.items()`, skip only `None` dictionary
values, join other values to the model path, and assert existence. It never
parses config contents.

`load_yaml` is qualitatively different. PyYAML 6.0.3 `SafeLoader` performs YAML
1.1 implicit resolution and constructs Python-specific values. The vendored
resolver and constructor are under
`third_party/sources/pyyaml-6.0.3/lib/yaml/`; the lock entry is in `uv.lock`.
Direct parser substitution would silently change behavior.

## Rollback

Keep `config_utils.load_yaml`, `check_configs`, and
`InferenceOnnx.load_config` Python-owned. Removing both independent Rust
candidates later would not change any current caller.
