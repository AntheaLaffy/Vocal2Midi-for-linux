# hfa_config_validation_core - dependency_bootstrap_reviewer

Date: 2026-07-18
Decision: pass-with-followups

## Findings

- Severity: low
- Location: rewrite-in-rust/fixtures/hfa_config_validation_core.jsonl:9; rewrite-in-rust/fixtures/hfa_config_validation_core.jsonl:28; rewrite-in-rust/rust/crates/v2m-core/src/hfa_config.rs:18
- Issue: The fixture corpus proves representative dynamic Python failures, but it does not exhaust the JSON-compatible value kinds now represented by the Rust seam. Top-level numeric values and dictionary-entry `float`/mapping values are implemented through `HfaConfigValue::{Int, Float, Mapping}` but are not explicit fixture cases.
- Evidence: `uv run python` fixture summary reported 34 cases with top-level injected value types `dict`, `NoneType`, `list`, `str`, and `bool`; dictionary-entry value types were `str`, `NoneType`, `int`, `bool`, and `list`. The Rust enum includes `Int`, `Float`, and nested `Mapping` variants.
- Required fix: Before cross-unit composition or promotion, either add explicit Python/Rust fixture rows for top-level numeric values and dictionary-entry float/mapping values, or narrow the dependency/bootstrap wording from JSON-compatible coverage to representative JSON-compatible failures. Rerun the Python checker and Rust unit test after that change.

- Severity: low
- Location: rewrite-in-rust/dependencies/hfa_pyyaml_safe_load_contract.yaml:31; rewrite-in-rust/manifest.yaml:1517; rewrite-in-rust/bootstrap/hfa_pyyaml_safe_load_contract.md:35
- Issue: The sibling loader dependency record says `inventory_impact.decision: confirmed`, while the manifest and bootstrap keep `hfa_pyyaml_safe_load_contract` planned/provisional until a tagged value/error fixture and parser/adapter strategy exist. This does not undermine the validation child, but it can confuse future coordinator state updates for the loader child.
- Evidence: Record 0079 says the loader child "remains planned and provisional" and forbids writer work before its tagged fixture and parser requirements are satisfied. The bootstrap repeats those pre-writer requirements. The manifest has `status: planned` and `inventory_status: provisional`.
- Required fix: Align the loader child dependency record with the manifest wording, or explicitly define "confirmed" there as "real future unit, not writer-ready." Keep this validation unit separate from full PyYAML loader ownership.

## Boundary Decision

The `hfa_config_validation_core` manifest boundary is confirmed for this role.
The older mixed `hfa_config_file_contract_core` should remain split/replaced:
this unit covers only deterministic `check_configs` behavior over an injected
loader outcome, and `hfa_pyyaml_safe_load_contract` must remain separate and
legacy-owned until its full PyYAML value/tag/error contract is bootstrapped.

The re-cut is justified. `config_utils.check_configs` only checks ordered file
existence, calls `load_yaml` once after both files exist, performs dynamic
`.get(..., [])` and `.items()` access, and validates dictionary paths. PyYAML
6.0.3 `safe_load` is materially broader: YAML 1.1 implicit resolution, Python
date/datetime/bytes/set/omap/pairs construction, aliases/merges, duplicate
replacement, single-document enforcement, tag rejection, and marked parser or
constructor errors.

## Dependency And Bootstrap Assessment

Capability coverage is sufficient for dependency-bootstrap promotion evidence
with the followups above tracked. The unit has no bridge dependencies, no YAML
crate, and no production caller route. `v2m-core` depends on `serde_json` with
`preserve_order`, which is appropriate for fixture parsing and first-failure
dictionary order; no `serde_yaml_ng`, `rust-yaml`, `yaml-rust`, or
`unsafe-libyaml` dependency is present in the Rust workspace.

The kept-legacy decisions are correct: `config_utils.load_yaml`, complete
PyYAML `SafeLoader` behavior, `InferenceOnnx.load_config` JSON/VERSION parsing,
ONNX/model sessions, API/model routing, and production imports remain Python
owned. The partial crate reuse policy from record 0080 is represented correctly:
candidate YAML crates are deferred to the loader child as possible lower-layer
parser/event substrates, not rejected merely because their high-level `Value`
APIs are not PyYAML-compatible.

Writer/reviewer separation is intact for this pass. I wrote only this report.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_hfa_config_validation_core.py`: passed, `validated 34 hfa_config_validation_core fixtures`.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_config_validation_core`: passed, 1 matching Rust fixture parity test passed.
- `uv run python -m py_compile inference/HubertFA/tools/config_utils.py rewrite-in-rust/bootstrap/check_hfa_config_validation_core.py`: passed.
- `uv run python -c "import yaml; print(yaml.__version__)"`: passed, printed `6.0.3`.
- `uv run python scripts/audit_vendored_sources.py`: passed, source audit reported 135 Python packages, 41 native-extension packages, 269 foreign runtime native binaries, and 0 third_party binary artifacts.
- `git diff --check`: passed.
- `rg -n "serde_yaml_ng|rust-yaml|yaml-rust|unsafe-libyaml|YamlLoader|SafeLoader|safe_load" rewrite-in-rust/rust rewrite-in-rust/dependencies rewrite-in-rust/bootstrap rewrite-in-rust/records`: confirmed no YAML parser dependency in Rust and only documented PyYAML references for this scope.

## Residual Risk

This role does not prove stage behavior parity or error/tracing quality. Full
PyYAML loader parity, non-JSON PyYAML value composition, arbitrary mapping keys,
source-marked parser errors, and production bridge/caller routing remain
unproven by design and belong to later reviews or the separate loader unit.

## Promotion Note

This dependency/bootstrap review does not block the coordinator from using the
report as state-update evidence for `hfa_config_validation_core`, provided the
two low-severity followups are tracked. Do not mark the unit fully verified
until the remaining required stage behavior and error/tracing reviews pass.
