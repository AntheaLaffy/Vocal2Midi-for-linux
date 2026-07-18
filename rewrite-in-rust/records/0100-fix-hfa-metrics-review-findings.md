# 0100 - Fix HFA Metrics Review Findings

Date: 2026-07-18

## Context

The first independent behavior and data/algorithm reviews for
`hfa_metrics_core` failed on two parity issues:

- the Rust `python_round_6` helper used Rust away-from-zero rounding instead of
  Python `round(value, 6)` half-even tie behavior;
- `IntersectionOverUnion::compute_list` returned a `BTreeMap`, which sorted
  caller requests and could not preserve repeated list-mode phonemes.

## Fix

`python_round_6` now applies a six-decimal half-even tie rule for finite f64
values. The fixture set includes `boundary_distance_half_even_rounding_tie`,
where Python computes a distance of `0.0000005` and returns `0.0`.

`IntersectionOverUnion::compute_list` now returns an ordered
`Vec<(String, Result<Option<f64>, HfaMetricError>)>`. A direct Rust unit test
checks a non-sorted request with a repeated phoneme:

```text
["c", "a", "missing", "a", "b"]
```

The JSON fixture projection still mirrors Python dict output, while the public
Rust API now has an order-preserving shape for future callers.

## Verification

```bash
uv run python rewrite-in-rust/bootstrap/check_hfa_metrics_core.py --write
uv run python rewrite-in-rust/bootstrap/check_hfa_metrics_core.py
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_metrics
cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check
cargo clippy --manifest-path rewrite-in-rust/rust/Cargo.toml --all-targets --all-features -- -D warnings
```

All passed.

## Review State

The first failed reports remain durable audit evidence. Rerun behavior and
data/algorithm reviews before moving `hfa_metrics_core` beyond
`reimplemented`.
