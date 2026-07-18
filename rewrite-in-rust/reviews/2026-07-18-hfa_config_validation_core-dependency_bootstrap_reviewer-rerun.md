# hfa_config_validation_core - dependency_bootstrap_reviewer

Date: 2026-07-18
Decision: fail

## Findings

- Severity: medium
- Location: rewrite-in-rust/manifest.yaml:1512
- Issue: The manifest marks `hfa_config_validation_core` as `verified` and cites `rewrite-in-rust/records/0082-close-hfa-config-validation-gate.md` as verification evidence, but that record file is missing from the current worktree.
- Evidence: `test -e rewrite-in-rust/records/0082-close-hfa-config-validation-gate.md` returned `record_0082_exists=1`; `rg -n "0082-close-hfa-config-validation-gate|hfa_config_validation_core" rewrite-in-rust/records rewrite-in-rust/reviews rewrite-in-rust/manifest.yaml` found the 0082 path only in the manifest and no matching record under `rewrite-in-rust/records/`; `nl -ba rewrite-in-rust/records/0082-close-hfa-config-validation-gate.md` failed with `No such file or directory`.
- Required fix: Create the cited durable close-gate record or remove the stale manifest citation before keeping this unit verified. No production-code or dependency-seam change is required for this fix.

## Closed Prior Findings

The two low findings from the original dependency/bootstrap review are closed.

- Fixture coverage is closed: `rewrite-in-rust/fixtures/hfa_config_validation_core.jsonl:13` and `:14` add top-level integer and float loader values; `:33` and `:34` add dictionary-entry float and mapping values. The dependency and bootstrap docs now describe 38 cases and numeric/mapping coverage at `rewrite-in-rust/dependencies/hfa_config_validation_core.yaml:30` and `rewrite-in-rust/bootstrap/hfa_config_validation_core.md:44`.
- PyYAML sibling inventory wording is closed: `rewrite-in-rust/dependencies/hfa_pyyaml_safe_load_contract.yaml:31` now says `decision: provisional`, matching the planned/provisional manifest state at `rewrite-in-rust/manifest.yaml:1522` and the bootstrap's pre-writer requirements at `rewrite-in-rust/bootstrap/hfa_pyyaml_safe_load_contract.md:35`.

## Boundary Decision

The `hfa_config_validation_core` unit boundary remains confirmed for this role.
The re-cut to injected loader validation remains justified: `config_utils.check_configs`
only validates ordered vocab/config existence, calls the injected loader after
both files exist, performs dynamic `get`/`items` access, and validates dictionary
paths. Full PyYAML `safe_load` value/tag/error behavior remains correctly split
into the planned and provisional `hfa_pyyaml_safe_load_contract`.

No dependency/bootstrap capability blocker remains. The remaining blocker is
state evidence consistency: the verified manifest names a record that is absent.

## Dependency And Bootstrap Assessment

Capability coverage is sufficient after the fixture expansion. The Python
checker executes the real `config_utils.check_configs` with a monkeypatched
loader, and the Rust fixture-parity test consumes the same 38-case JSONL corpus.
The fixture summary confirmed all previously missing cases are present:
top-level int/float loader values and dictionary-entry float/mapping values.

The kept-legacy decisions remain correct. `config_utils.load_yaml`, complete
PyYAML 6.0.3 `SafeLoader` behavior, non-JSON PyYAML value composition,
`InferenceOnnx.load_config`, ONNX/model sessions, API/model routing, and
production imports remain Python-owned. `hfa_config_validation_core` adds no
bridge, no production caller route, and no YAML parser dependency.

The crate reuse policy remains aligned with record 0080. No Rust YAML crate is
added for this validator; candidate YAML crates are deferred to the sibling
loader unit as possible parser/event substrates with a PyYAML compatibility
adapter.

Writer/reviewer separation is intact for this pass. I wrote only this rerun
report.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_hfa_config_validation_core.py`: passed, `validated 38 hfa_config_validation_core fixtures`.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_config_validation_core`: passed, 1 matching Rust fixture-parity test passed.
- `uv run python -m py_compile inference/HubertFA/tools/config_utils.py rewrite-in-rust/bootstrap/check_hfa_config_validation_core.py`: passed.
- `uv run python -c "import yaml; print(yaml.__version__)"`: passed, printed `6.0.3`.
- `uv run python scripts/audit_vendored_sources.py`: passed, source audit reported 135 Python packages, 41 native-extension packages, 269 foreign runtime native binaries, and 0 third_party binary artifacts.
- `git diff --check`: passed.
- `uv run python -c "...fixture summary..."`: passed, reported 38 cases and confirmed `loaded_integer_has_no_get`, `loaded_float_has_no_get`, `float_dictionary_value_type_error`, and `mapping_dictionary_value_type_error` are present.
- `rg -n "name = \"(serde_yaml_ng|rust-yaml|yaml-rust|unsafe-libyaml|serde_json)\"|serde_yaml_ng|rust-yaml|yaml-rust|unsafe-libyaml" rewrite-in-rust/rust/Cargo.lock rewrite-in-rust/rust/Cargo.toml rewrite-in-rust/rust/crates/*/Cargo.toml`: found only `serde_json`, no Rust YAML parser dependency.
- `test -e rewrite-in-rust/records/0082-close-hfa-config-validation-gate.md`: failed; the manifest-cited record is absent.

## Residual Risk

This role still does not prove full PyYAML loader parity, non-JSON PyYAML value
composition, arbitrary mapping keys, source-marked parser errors, production
bridge/caller routing, or runtime promotion. Those remain outside this validation
unit by design.

## Promotion Note

Do not keep `hfa_config_validation_core` verified as-is, because the current
manifest verification list cites a missing record. After the missing
`rewrite-in-rust/records/0082-close-hfa-config-validation-gate.md` artifact is
created or the stale citation is removed, this dependency/bootstrap role has no
remaining blocker for the unit's verified state.
