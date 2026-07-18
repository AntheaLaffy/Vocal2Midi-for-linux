# hfa_config_validation_core - dependency_bootstrap_reviewer

Date: 2026-07-18
Decision: pass

## Findings

No findings.

## Closed Prior Findings

The two low findings from the original dependency/bootstrap review remain
closed, and the medium blocker from the first rerun is now closed.

- Fixture coverage is closed: `rewrite-in-rust/fixtures/hfa_config_validation_core.jsonl:13`
  and `:14` cover top-level integer and float loader values; `:33` and `:34`
  cover dictionary-entry float and mapping values. The dependency/bootstrap docs
  now describe 38 cases and numeric/mapping coverage at
  `rewrite-in-rust/dependencies/hfa_config_validation_core.yaml:30` and
  `rewrite-in-rust/bootstrap/hfa_config_validation_core.md:44`.
- PyYAML sibling inventory wording is closed:
  `rewrite-in-rust/dependencies/hfa_pyyaml_safe_load_contract.yaml:31` now marks
  inventory impact as `provisional`, matching the planned/provisional manifest
  state at `rewrite-in-rust/manifest.yaml:1522` and the bootstrap pre-writer
  requirements at `rewrite-in-rust/bootstrap/hfa_pyyaml_safe_load_contract.md:35`.
- Missing-record blocker is closed: `rewrite-in-rust/manifest.yaml:1512` cites
  `rewrite-in-rust/records/0082-close-hfa-config-validation-gate.md`, and that
  record now exists with gate-closure evidence at
  `rewrite-in-rust/records/0082-close-hfa-config-validation-gate.md:1`.

## Boundary Decision

The `hfa_config_validation_core` manifest boundary is confirmed for this role.
The re-cut to injected loader validation remains justified and should stay split
from `hfa_pyyaml_safe_load_contract`.

This unit covers deterministic `config_utils.check_configs` behavior over an
injected loader outcome: vocab/config existence order, loader call order, dynamic
`get`/`items` failures, dictionary path validation, suffix rendering, exact
error projections, unparsed config contents, and repeated calls. It does not
parse YAML, claim PyYAML `safe_load` parity, route production callers, replace
`InferenceOnnx.load_config`, or add a bridge.

## Dependency And Bootstrap Assessment

Capability coverage is sufficient for dependency/bootstrap promotion evidence.
The Python checker executes the real legacy `check_configs` with a monkeypatched
loader, and the Rust test consumes the same 38-case JSONL corpus. The fixture
summary confirmed all previously missing JSON-compatible variants are present.

The kept-legacy decisions remain correct: `config_utils.load_yaml`, full PyYAML
6.0.3 `SafeLoader` behavior, non-JSON PyYAML values, `InferenceOnnx.load_config`,
ONNX/model sessions, API/model routing, and production imports remain
Python-owned. The sibling loader unit retains a provisional parser/event plus
PyYAML compatibility-adapter plan. No Rust YAML parser dependency is introduced
for this validator.

Writer/reviewer separation is intact for this pass. I wrote only this rerun2
report.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_hfa_config_validation_core.py`: passed, `validated 38 hfa_config_validation_core fixtures`.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_config_validation_core`: passed, 1 targeted fixture-parity test passed.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml`: passed, 114 `v2m-core` tests and 5 quant bridge tests passed.
- `cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check`: passed.
- `uv run python -m py_compile inference/HubertFA/tools/config_utils.py rewrite-in-rust/bootstrap/check_hfa_config_validation_core.py`: passed.
- `uv run python -c "import yaml; print(yaml.__version__)"`: passed, printed `6.0.3`.
- `uv run python scripts/audit_vendored_sources.py`: passed, source audit reported 135 Python packages, 41 native-extension packages, 269 foreign runtime native binaries, and 0 third_party binary artifacts.
- `git diff --check`: passed.
- `uv run python -c "...fixture summary..."`: passed, reported 38 cases and confirmed `loaded_integer_has_no_get`, `loaded_float_has_no_get`, `float_dictionary_value_type_error`, and `mapping_dictionary_value_type_error` are present.
- `rg -n "name = \"(serde_yaml_ng|rust-yaml|yaml-rust|unsafe-libyaml|serde_json)\"|serde_yaml_ng|rust-yaml|yaml-rust|unsafe-libyaml" rewrite-in-rust/rust/Cargo.lock rewrite-in-rust/rust/Cargo.toml rewrite-in-rust/rust/crates/*/Cargo.toml`: found only `serde_json`, no Rust YAML parser dependency.
- `test -e rewrite-in-rust/records/0082-close-hfa-config-validation-gate.md`: passed, `record_0082_exists=0`.

## Residual Risk

This review does not prove full PyYAML loader parity, non-JSON PyYAML value
composition, arbitrary mapping keys, source-marked parser errors, production
bridge/caller routing, or runtime promotion. Those remain outside this
validation unit and are reserved for `hfa_pyyaml_safe_load_contract` or future
promotion work.

## Promotion Note

This dependency/bootstrap role no longer blocks the verified state. The
coordinator can keep `hfa_config_validation_core` verified for the current
legacy-owned, no-bridge Rust library seam.
