# quantization_activation_policy - behavior_reviewer

Date: 2026-07-15
Decision: pass

## Findings

No findings.

Evidence:

- The Python compatibility source normalizes with `(mode or "simple").lower()`, returns `True` for exact normalized `dp`, and otherwise returns `quantization_step > 0` at `../inference/quant/quantization.py:806`.
- The Rust implementation preserves the same boundary for `None`, empty string, lowercase-only normalization without trimming, exact `dp`, and positive-step fallback at `rust/crates/v2m-core/src/quant.rs:7`.
- The parity fixture covers `None`, empty string, simple, smart, bayes, unknown, `dp`, uppercase `DP`, and whitespace-padded `dp` cases at `fixtures/quantization_activation_policy.tsv:2`.
- The production caller still imports and calls the legacy Python predicate before `quantize_notes` at `../inference/pipeline/auto_lyric_hybrid.py:19` and `../inference/pipeline/auto_lyric_hybrid.py:443`.
- The rollback route remains legacy-owned in `manifest.yaml:169`, and the Rust crate has no bridge or dependency addition in `rust/crates/v2m-core/Cargo.toml:1`.

## Checks

- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml quant`: passed; 3 quant tests passed, 22 filtered out.
- `uv run python rewrite-in-rust/bootstrap/check_quantization_activation_policy.py`: passed; exited with status 0.

## Residual Risk

This review covers `should_apply_quantization` only. It does not prove behavior for `quantize_notes`, simple/smart/dp/bayes note mutation, GUI/Web/application defaults, or a future Python/Rust bridge. The Rust API accepts `Option<&str>` and `i64`, matching the documented fixture boundary but not arbitrary dynamic Python objects outside the current caller contract.

## Promotion Note

This behavior review does not block promotion of the `quantization_activation_policy` unit to the next coordinator state. The coordinator should keep runtime ownership with legacy Python until a separate promotion unit introduces and reviews a bridge.
