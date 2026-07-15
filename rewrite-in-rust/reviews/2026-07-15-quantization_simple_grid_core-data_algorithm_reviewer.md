# quantization_simple_grid_core - data_algorithm_reviewer

Date: 2026-07-15
Decision: pass

## Findings

No findings.

## Follow-up Re-review Evidence

- The previous medium fixture gap is resolved. The durable table now includes a seconds-to-ticks half-even tie case at `rewrite-in-rust/fixtures/quantization_simple_grid_core.tsv:5`: tempo `100.0`, step `25`, onset `0.015625` converts to `12.5` ticks and rounds to even `12`, while offset `0.046875` converts to `37.5` ticks and rounds to even `38`. The table also includes a non-default positive tempo and step at `rewrite-in-rust/fixtures/quantization_simple_grid_core.tsv:6` with tempo `90.0` and step `45`, so the shared parity fixture would catch hard-coded `120.0` tempo or `60` step behavior.
- The Python reference still performs seconds-to-ticks conversion with `int(round(t * tempo * 8))` at `../inference/quant/quantization.py:8`, then applies grid snapping in `_quantize_notes_simple` at `../inference/quant/quantization.py:697` and `../inference/quant/quantization.py:708`. The Rust implementation mirrors the same two-stage rounding path at `rewrite-in-rust/rust/crates/v2m-core/src/quant.rs:52`, `rewrite-in-rust/rust/crates/v2m-core/src/quant.rs:66`, and `rewrite-in-rust/rust/crates/v2m-core/src/quant.rs:92`.
- The previous low numeric-precondition follow-up is resolved for this pre-promotion unit. Finite note timings and positive finite tempo are documented in the bootstrap boundary at `rewrite-in-rust/bootstrap/quantization_simple_grid_core.md:22`, repeated for bridge/promotion work at `rewrite-in-rust/bootstrap/quantization_simple_grid_core.md:102`, recorded as fixture/pre-promotion input policy at `rewrite-in-rust/dependencies/quantization_simple_grid_core.yaml:27`, and called out as future invalid-input mapping outside this unit at `rewrite-in-rust/dependencies/quantization_simple_grid_core.yaml:51`. The Rust helper documents the same precondition at `rewrite-in-rust/rust/crates/v2m-core/src/quant.rs:31`.
- This remains a fixture-bound independent Rust library unit. The manifest keeps legacy Python as current owner and lists no runtime promotion for this unit at `rewrite-in-rust/manifest.yaml:172`, `rewrite-in-rust/manifest.yaml:176`, and `rewrite-in-rust/manifest.yaml:188`.

## Checks

- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml simple_grid`: passed; 2 tests run, 26 filtered out.
- `uv run python rewrite-in-rust/bootstrap/check_quantization_simple_grid_core.py`: passed.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml tick_conversion`: passed; 1 test run, 27 filtered out.

## Algorithm Notes

- The reviewed unit is explicitly `quantization_simple_grid_core`; this report covers only the `data_algorithm_reviewer` role.
- Writer/reviewer separation is intact for this pass: no production code was modified during review, and only this report was updated.
- The Rust implementation stays inside the documented boundary from `records/0007-split-quantization-core.md:25`: `_ticks_from_sec` and `_quantize_notes_simple` behavior only, with public dispatch, smart duration DP, phrase DP, Bayesian quantization, GUI/Web settings, and runtime promotion left outside this unit.
- For finite positive inputs covered by the fixtures, the implementation matches the Python algorithm shape: sort by onset, snapshot original timings, half-even tick/grid rounding, monotonic onset bump, touching-note glue, one-step minimum duration, next-onset clipping, and metadata preservation.
- Complexity remains acceptable and matches the Python path: stable sort dominates at `O(n log n)`, followed by linear passes and `O(n)` temporary storage for original timings and quantized ticks.

## Residual Risk

This pass does not review public dispatch, smart duration DP, phrase DP, Bayesian quantization, GUI/Web defaults, or runtime promotion. The current Rust boundary assumes finite note timings and positive finite tempo; if a future bridge accepts arbitrary numeric inputs, promotion work must add validation or Python-compatible error mapping before Rust becomes runtime owner.

## Promotion Note

This `data_algorithm_reviewer` role no longer blocks coordinator state update for `quantization_simple_grid_core`. Runtime promotion is still out of scope; production ownership remains legacy Python until a separate promotion unit verifies a bridge and invalid-input handling.
