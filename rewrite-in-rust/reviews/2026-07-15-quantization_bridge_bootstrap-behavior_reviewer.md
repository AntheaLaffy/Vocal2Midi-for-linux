# quantization_bridge_bootstrap - behavior_reviewer

Date: 2026-07-15
Unit: quantization_bridge_bootstrap
Role: behavior_reviewer

## Findings

No current behavior findings.

The previous medium finding about timing-only note objects is resolved. The wrapper now builds bridge payloads with `_coerce_optional_pitch`, which sends `0.0` when `pitch` is missing, `None`, non-numeric, or non-finite (`inference/quant/rust_bridge.py:88`, `inference/quant/rust_bridge.py:170`). `_apply_response` still mutates only `onset` and `offset`, then reorders the original note objects (`inference/quant/rust_bridge.py:187`, `inference/quant/rust_bridge.py:197`). The bootstrap checker now includes a `TimingOnlyNote` fixture and asserts that the `rust-json` path does not add `pitch` metadata (`rewrite-in-rust/bootstrap/check_quantization_bridge_bootstrap.py:42`, `rewrite-in-rust/bootstrap/check_quantization_bridge_bootstrap.py:243`, `rewrite-in-rust/bootstrap/check_quantization_bridge_bootstrap.py:254`).

## Decision

pass

The behavior review passes for the bridge bootstrap scope. Legacy Python remains the default runtime owner, and the `rust-json` wrapper now preserves the public timing-only accepted-input behavior while leaving note metadata untouched.

## Checks Run

- `cargo build --manifest-path rewrite-in-rust/rust/Cargo.toml --bin v2m-quant-bridge`: passed.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml`: passed. Rust workspace ran 37 `v2m-core` unit tests, 5 `v2m-quant-bridge` unit tests, and doc tests.
- `uv run python rewrite-in-rust/bootstrap/check_quantization_bridge_bootstrap.py`: passed. The harness produced no output on success.
- `uv run pytest tests/test_web_api.py`: passed, 53 tests.
- Targeted timing-only simple-mode probe: passed. `rust-json` matched legacy timing output for a note object without `pitch` or `lyric`, retained the original object, and did not add either metadata attribute.
- Targeted metadata-preservation probe: passed. Existing `pitch`, `lyric`, and unrelated `marker` metadata were preserved on the original objects after `rust-json`, including a non-numeric `pitch` value that is only coerced inside the JSON payload.
- Targeted timing-only cross-mode probe: passed for simple, smart, bayes, dp with step zero, and disabled simple mode. `rust-json` matched legacy onset/offset output, retained original object identities, and did not add `pitch` or `lyric`.

## Residual Risk

The behavior fixture set covers simple, smart, bayes, dp, disabled, unknown, uppercase, padded, null mode, empty-note, schema, overflow, missing-binary, wrapper fallback, non-finite wrapper input, startup failure, and timing-only note paths. Error behavior for malformed successful bridge responses and cancellation remains outside this behavior pass and should be covered by the error-tracing or architecture review before runtime promotion.

## Promotion Note

This behavior review does not mark the manifest verified. From the behavior role only, `quantization_bridge_bootstrap` is ready for coordinator state update after the remaining required review roles are satisfied.
