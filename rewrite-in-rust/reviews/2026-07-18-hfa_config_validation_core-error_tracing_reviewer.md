# hfa_config_validation_core - error_tracing_reviewer

Date: 2026-07-18
Decision: pass

Unit: `hfa_config_validation_core`
Role: `error_tracing_reviewer`

## Findings

No findings.

The reviewed Rust seam keeps the compatibility error surface explicit for the
current injected-loader boundary. The legacy source raises only assertion,
loader, attribute, and type failures from `check_configs`
(`inference/HubertFA/tools/config_utils.py:13`). Rust models those projections as
`HfaConfigValidationError::{Assertion, Attribute, Type, Loader}` and exposes the
Python exception class plus exact message through `exception_type()` and
`message()` (`rewrite-in-rust/rust/crates/v2m-core/src/hfa_config.rs:67`,
line 109, and line 120). The fixture encoder compares exactly the same
`{type,message}` shape against the Python checker at
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_config.rs:354`.

Loader error propagation matches the declared intermediate contract. The Python
checker monkeypatches `load_yaml`, records the loader path, and raises the
fixture-supplied exception unchanged (`rewrite-in-rust/bootstrap/check_hfa_config_validation_core.py:80`).
Rust calls the injected loader only after vocab/config existence succeeds and
uses `?` to return the loader error without rewriting it
(`rewrite-in-rust/rust/crates/v2m-core/src/hfa_config.rs:174`). Fixture lines 8
and 34 prove a single loader error and repeated uncached loader errors with
exact exception type/message and path order.

Path context is preserved at the compatibility-message level required by this
unit. Vocab and config existence failures include the checked path at
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_config.rs:158` and line 166.
Dictionary failures use the legacy absolute assertion path at line 184 and
line 186. Fixture lines 5 through 7 cover vocab/config order and messages;
lines 24 through 27 cover relative, absolute, and first-failing dictionary
paths.

Invalid loaded shapes are diagnosable and Python-compatible inside the declared
JSON-compatible value domain. Top-level non-mappings project exact
`AttributeError` messages through `get_mapping_value`
(`rewrite-in-rust/rust/crates/v2m-core/src/hfa_config.rs:41`), non-mapping
`dictionaries` values project exact `.items` `AttributeError` messages through
`mapping_items` at line 56, and non-path dictionary values project the legacy
`TypeError` at line 192. Fixture lines 9 through 17 and 28 through 30 bind those
invalid-shape diagnostics.

The implementation does not add a diagnostic sink or production route. Static
inspection found no production `println!`, `eprintln!`, `dbg!`, `tracing`, or
`log` calls in the module; the only panic sites are in the test harness. Routing
inspection found the Rust module exported from `v2m-core`, but no application,
GUI, Web, or inference caller using it. Production still calls legacy Python
`check_configs` from `InferenceOnnx.load_config`
(`inference/HubertFA/onnx_infer.py:19`) and from the HFA API load path
(`inference/API/hfa_api.py:99`). This matches the manifest rollback statement
at `rewrite-in-rust/manifest.yaml:1515`.

The remaining promotion-time error mapping is explicit rather than hidden in
this unit. Record 0079 splits deterministic validation from full PyYAML loading
and states that this unit must not be promoted as `load_yaml` parity
(`rewrite-in-rust/records/0079-recut-hfa-config-file-contract.md:50`). The
planned loader unit requires path, operation, marks, Python exception
class/message, file/decode/parser/composer/constructor errors, and safe-tag
rejection before writer work (`rewrite-in-rust/dependencies/hfa_pyyaml_safe_load_contract.yaml:20`
and `rewrite-in-rust/bootstrap/hfa_pyyaml_safe_load_contract.md:35`).

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_hfa_config_validation_core.py`: passed, validated 34 fixtures.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_config_validation_core`: passed, 1 targeted Rust parity test.
- `uv run python -m py_compile inference/HubertFA/tools/config_utils.py rewrite-in-rust/bootstrap/check_hfa_config_validation_core.py`: passed.
- `rg -n "println!|eprintln!|dbg!|tracing::|log::|todo!|unimplemented!|panic!" rewrite-in-rust/rust/crates/v2m-core/src/hfa_config.rs`: found only test-harness panic sites, no production diagnostic sink.
- `rg -n "hfa_config|check_configs_with_loader|v2m_core::hfa_config|HfaConfig" application inference gui web_server.py web_task_manager.py rewrite-in-rust/rust/crates`: found the Rust library module/export only; no production Python caller route.
- `git diff --check`: passed before this report was written.

## Residual Risk

This pass approves only the injected JSON-compatible validation seam. Full
PyYAML value construction, file/decode/parser errors, source marks,
non-JSON-safe values, arbitrary mapping keys, and Python exception
reconstruction remain owned by `hfa_pyyaml_safe_load_contract` and legacy Python.

Compatibility messages can contain local filesystem paths and loader-supplied
messages. The current Rust unit stores and returns them but does not log or
emit them. A future bridge must define display, redaction, retention,
non-UTF-8 path policy, and traceback/context reconstruction before production
routing.

## Promotion Note

This error/tracing role does not block coordinator state update for the current
independent, legacy-owned library seam. The coordinator may use this report as
passing evidence together with the dependency/bootstrap and behavior reviews.
This report does not approve production routing, a YAML loader replacement, or a
runtime-owner change.
