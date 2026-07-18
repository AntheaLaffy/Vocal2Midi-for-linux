# hfa_metrics_core - behavior_reviewer

Date: 2026-07-18
Decision: pass

## Findings

No findings.

- Severity: none
- Location: rust/crates/v2m-core/src/hfa_metrics.rs:588
- Issue: The prior Python `round(value, 6)` parity blocker is fixed for the reviewed public metric outputs.
- Evidence: Python still rounds public metric results with `round(..., 6)` in `inference/HubertFA/tools/metrics.py:102`, `:156`, `:162`, `:172`, `:255`, `:278`, and `:305`. Rust now routes those six-decimal outputs through `python_round_6`, whose tie branch rounds half values to an even scaled integer at `rust/crates/v2m-core/src/hfa_metrics.rs:588`. The generated fixture includes `boundary_distance_half_even_rounding_tie`, where a `0.0000005` boundary distance computes to `0.0` at `fixtures/hfa_metrics_core.jsonl:10`.
- Required fix: none.

- Severity: none
- Location: rust/crates/v2m-core/src/hfa_metrics.rs:352
- Issue: The prior `IntersectionOverUnion::compute_list` ordering blocker is fixed for the Rust public helper.
- Evidence: Python list-mode IoU iterates the caller's `phonemes` list when building the result at `inference/HubertFA/tools/metrics.py:169`. Rust `compute_list` now returns an ordered `Vec<(String, Result<Option<f64>, HfaMetricError>)>` by iterating the request slice at `rust/crates/v2m-core/src/hfa_metrics.rs:352`. The fixture runner now calls `metric.compute_list(&phonemes)` instead of manually bypassing the helper at `rust/crates/v2m-core/src/hfa_metrics.rs:808`, and the direct Rust test verifies request order plus a repeated phoneme at `rust/crates/v2m-core/src/hfa_metrics.rs:1057`.
- Required fix: none.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_hfa_metrics_core.py`: passed, validated 12 `hfa_metrics_core` fixtures.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_metrics`: passed, 2 targeted tests passed: `hfa_metrics_core_fixture_parity` and `iou_compute_list_preserves_request_order_and_duplicates`.

## Residual Risk

This rerun reviewed behavior parity only for `hfa_metrics_core`, with special attention to the two fixed findings. It did not re-review dependency strategy, Rust style, error/tracing, architecture, or data/algorithm concerns beyond behavior-observable parity. Python remains the runtime owner and the manifest remains `status: reimplemented` with `current_owner: legacy` at `manifest.yaml:1666`.

## Promotion Note

This behavior-reviewer rerun does not block promotion. Do not mark the manifest verified from this report alone; the coordinator should consume this as one role's evidence and wait for any other required reruns or promotion gates.
