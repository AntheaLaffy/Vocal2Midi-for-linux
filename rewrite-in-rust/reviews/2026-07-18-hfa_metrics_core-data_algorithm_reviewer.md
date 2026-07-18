# hfa_metrics_core - data_algorithm_reviewer

Date: 2026-07-18
Decision: fail

## Findings

- Severity: medium
- Location: rust/crates/v2m-core/src/hfa_metrics.rs:588
- Issue: `python_round_6` does not implement Python `round(..., 6)` tie behavior. The Python reference uses `round(..., 6)` for edit ratios, IoU, boundary distance, boundary ratio, and weighted ratio, while the Rust helper multiplies by `1_000_000.0` and calls `f64::round()`. That rounds exact halfway values away from zero, not Python's half-even result.
- Evidence: `inference/HubertFA/tools/metrics.py:102`, `:156`, `:162`, `:172`, `:255`, `:278`, and `:305` all use Python `round(..., 6)`. `rust/crates/v2m-core/src/hfa_metrics.rs:239`, `:327`, `:348`, `:462`, `:511`, and `:573` route through `python_round_6`; `:588-589` implements the incompatible formula. `uv run python -c 'import math; x=1/128; print("python", round(x,6)); print("rust_formula", math.floor(x*1_000_000+0.5)/1_000_000); print("x_scaled", x*1_000_000)'` prints `python 0.007812`, `rust_formula 0.007813`, `x_scaled 7812.5`. This is reachable through `VlabelerEditRatio` because the Rust ratio divides integer edit distance by `2 * target.len()` at `rust/crates/v2m-core/src/hfa_metrics.rs:231` and `:239`; an accumulated edit distance of `1` with total `128` reaches `1/128`.
- Required fix: replace `python_round_6` with a Python-compatible half-even implementation for six decimal places, then add fixture coverage for at least one half-even tie through a public metric computation. Re-run the Python fixture checker and targeted Rust tests after regenerating the affected expected values.

- Severity: medium
- Location: rust/crates/v2m-core/src/hfa_metrics.rs:353
- Issue: `IntersectionOverUnion::compute_list` cannot preserve caller-provided order. The fixture and bootstrap contract require list-mode IoU to preserve requested phoneme order, matching Python's dict-comprehension order, but the public Rust method returns a `BTreeMap`, which sorts by key and also cannot represent repeated requested phonemes as repeated results.
- Evidence: the Python list-mode branch iterates `for ph in phonemes` at `inference/HubertFA/tools/metrics.py:170-180`; the bootstrap requires list lookup order preservation at `bootstrap/hfa_metrics_core.md:53-55`; the generated fixture requests `["c", "a", "missing", "b"]` and expects that order at `fixtures/hfa_metrics_core.jsonl:5`. The Rust method is documented as caller-ordered at `rust/crates/v2m-core/src/hfa_metrics.rs:352` but returns `BTreeMap<String, Result<Option<f64>, HfaMetricError>>` at `:353-360`. The fixture replay test does not call `compute_list`; it manually loops over fixture items and inserts into a JSON map at `rust/crates/v2m-core/src/hfa_metrics.rs:788-809`, so the public method's data-structure bug is currently untested.
- Required fix: change list-mode output to an order-preserving shape such as `Vec<(String, Result<Option<f64>, HfaMetricError>)>` or another explicit ordered projection, and add a Rust test that calls the public `compute_list` API with non-sorted and duplicate requested phonemes.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_hfa_metrics_core.py`: passed, validated 11 fixtures.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_metrics_core`: passed, 1 targeted fixture parity test passed.
- `cargo clippy --manifest-path rewrite-in-rust/rust/Cargo.toml -p v2m-core --all-targets -- -D warnings`: passed.
- `uv run python -c 'import math; x=1/128; print("python", round(x,6)); print("rust_formula", math.floor(x*1_000_000+0.5)/1_000_000); print("x_scaled", x*1_000_000)'`: demonstrated the rounding mismatch for the Rust formula used by `python_round_6`.

## Residual Risk

The dynamic-programming implementations for vlabeler edit distance and LCS are O(m*n) memory and time, matching the Python algorithms. I did not find an algorithmic parity issue there, but the current fixture set has no large-input or benchmark case, so promotion would still rely on callers keeping these metric tiers fixture-sized or modest.

`ndarray` use is narrow and limited to one-dimensional absolute time-difference sums in `BoundaryEditDistance`; I did not find a data-structure reason to reject that dependency choice for this unit.

## Promotion Note

This role blocks promotion. Do not mark `hfa_metrics_core` verified until the rounding and IoU list output issues are fixed and independently rechecked.
