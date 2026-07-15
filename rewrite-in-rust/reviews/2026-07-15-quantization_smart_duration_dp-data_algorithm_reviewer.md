# quantization_smart_duration_dp - data_algorithm_reviewer

Date: 2026-07-15
Role: data_algorithm_reviewer
Unit: quantization_smart_duration_dp
Decision: pass

## Findings

No open findings.

The prior low-severity fixture-strength finding is resolved. The fixture table now includes `three_note_traceback_rest_mix` at rewrite-in-rust/fixtures/quantization_smart_duration_dp.tsv:11, which covers a three-note smart-duration path. The bootstrap and dependency coverage text now explicitly require that path at rewrite-in-rust/bootstrap/quantization_smart_duration_dp.md:112 and rewrite-in-rust/dependencies/quantization_smart_duration_dp.yaml:23.

## Checks

- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml smart_duration`: passed; 2 tests passed, 0 failed.
- `uv run python rewrite-in-rust/bootstrap/check_quantization_smart_duration_dp.py`: passed with no output.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml quant_candidate`: passed; 3 tests passed, 0 failed.
- Inline `uv run python` DP probes for `best_last_argmin_tie_shorter`, `transition_argmin_tie`, `rest_30_half_even_zero`, `rest_31_rounds_one`, and `rest_90_rounds_two`: passed; confirmed Python/NumPy first-min and rest-rounding behavior used by the fixture cases.
- Follow-up re-review `uv run python rewrite-in-rust/bootstrap/check_quantization_smart_duration_dp.py`: passed with no output.
- Follow-up re-review `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml smart_duration`: passed; 2 tests passed, 0 failed.
- Follow-up inline `uv run python` DP probe for `three_note_traceback_rest_mix`: passed; confirmed three notes, `q_durs=[120, 180, 120]`, two predecessor hops, and rounded rests of one and two steps.

## Evidence

The unit boundary is narrow and matches the records. `0008-confirm-smart-duration-boundary` limits this unit to `_quantize_notes_smart`, keeps phrase DP, Bayesian quantization, public dispatch, runtime routing, and bridge design out of scope, and requires Vec-backed DP plus explicit first-min scans instead of NumPy bindings at rewrite-in-rust/records/0008-confirm-smart-duration-boundary.md:21 and rewrite-in-rust/records/0008-confirm-smart-duration-boundary.md:24. The dependency record confirms the same capability boundary and no bridge dependencies at rewrite-in-rust/dependencies/quantization_smart_duration_dp.yaml:4 and rewrite-in-rust/dependencies/quantization_smart_duration_dp.yaml:16.

The Python source builds duration candidates once from `_build_duration_candidates`, fills an `np.float64` DP table, stores `np.int32` predecessors, uses `np.argmin` for transition and final first-min selection, then places onsets with half-even rounding and rests with the `< 0.5 step` threshold at inference/quant/quantization.py:739, inference/quant/quantization.py:742, inference/quant/quantization.py:759, inference/quant/quantization.py:763, inference/quant/quantization.py:775, and inference/quant/quantization.py:780.

The Rust implementation reuses `build_duration_candidates`, stores the DP and predecessor tables as flattened vectors, preserves first-min behavior by updating only on strict `<`, reconstructs durations through the predecessor table, and applies the same first-onset and rest formulas at rewrite-in-rust/rust/crates/v2m-core/src/quant.rs:147, rewrite-in-rust/rust/crates/v2m-core/src/quant.rs:150, rewrite-in-rust/rust/crates/v2m-core/src/quant.rs:178, rewrite-in-rust/rust/crates/v2m-core/src/quant.rs:195, rewrite-in-rust/rust/crates/v2m-core/src/quant.rs:204, rewrite-in-rust/rust/crates/v2m-core/src/quant.rs:212, and rewrite-in-rust/rust/crates/v2m-core/src/quant.rs:218.

The focused Python probe confirmed the critical numeric cases:

```text
best_last_argmin_tie_shorter: raw_durs=[90] candidates=[60, 120, 180, 240] first_row=[30.0, 30.0, 94.8, 150.0] best_last=0 q_durs=[60] rests=[]
transition_argmin_tie: candidates=[60, 120, 180, 240], transitions_to_180=[129.4, 64.6, 64.6, 124.6], argmin=1
rest_30_half_even_zero: rests=[(30, 0)]
rest_31_rounds_one: rests=[(31, 60)]
rest_90_rounds_two: rests=[(90, 120)]
```

The follow-up Python probe confirmed the new traceback fixture exercises the previously missing multi-row path:

```text
three_note_traceback_rest_mix: note_count=3 raw_durs=[150, 180, 90] candidates=[60, 120, 180, 240]
best_last_index=1 q_durs=[120, 180, 120] prev_hops=[1, 2] rests=[(33, 60), (117, 120)]
```

I did not find a data-structure, numeric-parity, first-min tie, duration-candidate reuse, rest-quantization, or complexity defect in the inspected implementation.

## Residual Risk

This review assumes the bootstrap boundary stated for the unit: finite note timings and positive finite tempo when quantization runs. Invalid numeric inputs such as NaN, infinities, very large tick values, or non-positive tempo remain promotion-time validation/error-mapping work, as recorded at rewrite-in-rust/bootstrap/quantization_smart_duration_dp.md:124 and rewrite-in-rust/dependencies/quantization_smart_duration_dp.yaml:42.

The algorithm remains O(n*m^2) like the Python implementation, where `m` is bounded by the fixed duration multiplier table. No benchmark is required for the current small candidate set, but promotion should avoid silently widening the candidate table without revisiting complexity.

## Promotion Note

This data/algorithm review does not block coordinator state update for `quantization_smart_duration_dp` after the required behavior review is also satisfactory. The fixture-strength follow-up identified in the first pass is resolved.
