# slice_method_and_bounds_contract - behavior_reviewer

Date: 2026-07-17
Decision: fail

## Findings

- Severity: medium
- Location: rewrite-in-rust/rust/crates/v2m-core/src/slice_method.rs:311
- Issue: Rust does not preserve the exact Python unsupported-method error message for string inputs containing a single quote.
- Evidence: The unit requires exact unsupported-method errors at rewrite-in-rust/bootstrap/slice_method_and_bounds_contract.md:20 and rewrite-in-rust/manifest.yaml:963. Both legacy call sites format the original input with Python repr at inference/API/slicer_api.py:64 and scripts/slice_asr_cli.py:135. A read-only legacy spot check showed both Python functions raise `Unsupported slicing method: "can't". Supported values: default, smart, heuristic, grid` for input `can't`. The Rust formatter always starts and ends with a single quote at rewrite-in-rust/rust/crates/v2m-core/src/slice_method.rs:313 and rewrite-in-rust/rust/crates/v2m-core/src/slice_method.rs:327, and escapes single quotes at rewrite-in-rust/rust/crates/v2m-core/src/slice_method.rs:317, so the same input would be formatted as `Unsupported slicing method: 'can\'t'. Supported values: default, smart, heuristic, grid`. Existing fixtures only cover `unknown` and the empty string at rewrite-in-rust/fixtures/slice_method_and_bounds_contract.jsonl:17 and rewrite-in-rust/fixtures/slice_method_and_bounds_contract.jsonl:18, and the focused Rust repr test covers newline but not Python quote-selection behavior at rewrite-in-rust/rust/crates/v2m-core/src/slice_method.rs:446.
- Required fix: Implement Python-compatible string repr selection/escaping for unsupported method messages, or record a narrower promoted error contract before integration. Add a fixture for an unsupported method containing a single quote, then rerun the legacy fixture harness and Rust `slice_method` tests.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_slice_method_and_bounds_contract.py`: pass.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml slice_method`: pass; 3 `slice_method` tests passed, 73 tests filtered out.
- `uv run python - <<'PY' ...`: pass; confirmed both legacy Python normalization functions format unsupported input `can't` with double-quoted Python repr in the error message.

## Residual Risk

The fixture table does not yet cover Python repr edge cases beyond newline and simple ASCII strings. The Rust method surface accepts `Option<&str>`, while legacy Python coerces non-`None` method objects through `str(method)` but formats errors with the original object repr; any future bridge must either preserve or explicitly narrow that behavior. The Rust bounds surface accepts `Option<f64>`, so Python `float(...)` coercion and conversion-failure behavior remains an adapter/promotion concern. Product ergonomics and actual audio slicing behavior were outside this `behavior_reviewer` pass.

## Promotion Note

This role blocks verification for `slice_method_and_bounds_contract` until the unsupported-method repr mismatch is fixed and covered by Python/Rust parity checks. The unit otherwise stays inside the confirmed minimum boundary: no production bridge is introduced, audio/model/filesystem behavior remains legacy-owned, and the manifest should remain under coordinator control.
