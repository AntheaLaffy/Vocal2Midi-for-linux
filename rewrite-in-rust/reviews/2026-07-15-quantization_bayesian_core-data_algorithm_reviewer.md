# quantization_bayesian_core - data_algorithm_reviewer

Date: 2026-07-15
Decision: pass-with-followups

## Findings

- Severity: low
- Location: rewrite-in-rust/fixtures/quantization_bayesian_helpers.tsv:35
- Issue: The Bayes decode fixture table covers non-overlap predecessor filtering and overlap fallback, but it does not isolate equal-cost first-min predecessor ties or final-min ties. Those ties are algorithmically important because Python keeps the first candidate when costs compare equal.
- Evidence: `rewrite-in-rust/bootstrap/quantization_bayesian_core.md:120` calls out first-min and final-min tie behavior as helper fixture scope, while the only `decode_segment_bayesian` rows are `decode_non_overlap_filter` and `decode_overlap_fallback` at `rewrite-in-rust/fixtures/quantization_bayesian_helpers.tsv:35` and `rewrite-in-rust/fixtures/quantization_bayesian_helpers.tsv:36`. Code inspection shows Rust uses strict `<` scans for predecessor and final selection at `rewrite-in-rust/rust/crates/v2m-core/src/quant.rs:1259` and `rewrite-in-rust/rust/crates/v2m-core/src/quant.rs:1277`, matching Python's strict `<` / `min` first-selection behavior at `inference/quant/quantization.py:631` and `inference/quant/quantization.py:639`.
- Required fix: Add helper fixture rows that force equal total costs for predecessor selection and final candidate selection, so the ordering-sensitive DP behavior is locked by durable parity data.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_quantization_bayesian_core.py`: passed.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml bayesian`: passed, 2 tests.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml`: passed, 37 tests.

## Residual Risk

The Rust implementation uses sorted `Vec` candidate lists, strict first-min scans, Vec-backed median/mean helpers, bounded candidate lists, and a 48-note Bayesian segment cap. That preserves the Python data-shape and DP complexity assumptions for fixture-covered finite inputs. Remaining risk is fixture hardening for exact tie paths and broader mixed prior groups; promotion work still needs explicit invalid numeric input validation before Rust owns runtime calls.

## Promotion Note

This role does not block promotion. The coordinator can promote after all required roles pass, with the tie-fixture hardening tracked as a follow-up.
