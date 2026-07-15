# quantization_bayesian_core - rust_style_reviewer

Date: 2026-07-15
Decision: pass-with-followups

## Findings

- Severity: low
- Location: rewrite-in-rust/rust/crates/v2m-core/src/quant.rs:1
- Issue: `quant.rs` now owns activation policy, simple grid, candidate primitives, smart duration DP, phrase DP, Bayesian quantization, and all fixture parsers/tests in one broad module. The Bayesian implementation follows the existing pattern and does not introduce public helper leakage, but the module shape is becoming harder to navigate and review as more quantization units land.
- Evidence: Bayesian public entrypoint is scoped to `quantize_notes_bayesian` at `rewrite-in-rust/rust/crates/v2m-core/src/quant.rs:478`; Bayes helper types/functions remain private around `rewrite-in-rust/rust/crates/v2m-core/src/quant.rs:114` and `rewrite-in-rust/rust/crates/v2m-core/src/quant.rs:861`; fixture-backed Bayes tests are embedded in the shared `quant::tests` module at `rewrite-in-rust/rust/crates/v2m-core/src/quant.rs:1969` and `rewrite-in-rust/rust/crates/v2m-core/src/quant.rs:2125`.
- Required fix: Non-blocking follow-up. After the active quantization unit queue stabilizes, split `quant.rs` into smaller internal modules, for example shared primitives plus simple/smart/phrase/bayesian implementation modules, while keeping the public API unchanged.

## Checks

- `cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --check`: pass.
- `uv run python rewrite-in-rust/bootstrap/check_quantization_bayesian_core.py`: pass.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml bayesian`: pass, 2 Bayesian tests passed.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml`: pass, 37 tests passed.
- `cargo clippy --manifest-path rewrite-in-rust/rust/Cargo.toml --lib -- -D warnings`: fail only on out-of-scope `slice_bounds.rs` `clippy::manual_range_contains` warnings at lines 54 and 57.
- `cargo clippy --manifest-path rewrite-in-rust/rust/Cargo.toml --lib -- -D warnings -A clippy::manual_range_contains`: pass.

## Residual Risk

This review covered Rust style only: module shape, ownership, visibility, warning hygiene, maintainability, and tests. It did not judge Python/Rust behavior parity, numerical algorithm quality, bridge architecture, runtime promotion, or user-facing ergonomics. The unit remains fixture-bound and assumes finite note timings plus positive finite tempo when quantization runs, as documented in the bootstrap record.

## Promotion Note

This Rust-style role does not block coordinator state update. The coordinator can mark `quantization_bayesian_core` verified after accepting all required role reports. Runtime promotion remains out of scope for this review.
