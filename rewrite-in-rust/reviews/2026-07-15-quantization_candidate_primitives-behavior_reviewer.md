# quantization_candidate_primitives - behavior_reviewer

Date: 2026-07-15
Decision: pass

## Findings

No findings.

Behavior parity for the selected helpers is supported by direct legacy/Rust
inspection and fixture-backed checks:

- `../inference/quant/quantization.py:61` through `../inference/quant/quantization.py:121` define the selected scalar, distance, note-pair, gap, and candidate-pair helpers; `../inference/quant/quantization.py:440` through `../inference/quant/quantization.py:455` define the selected duration/gap candidate helpers.
- `rust/crates/v2m-core/src/quant.rs:116` through `rust/crates/v2m-core/src/quant.rs:267` mirrors those helpers with the expected half-even rounding, Python positive-modulo behavior for positive step/modulo values, sorted unique vectors, first-tie nearest-candidate behavior, raw duration clamping, previous-end gap clamping, `end > start` pair filtering, and multiplier/ceil cap rules.
- `fixtures/quantization_candidate_scalar_primitives.tsv:2` through `fixtures/quantization_candidate_scalar_primitives.tsv:30` cover default/positive DP grid resolution, default/scaled segment shifts, nearest-candidate ties including unsorted first-tie order, modular/grid distances including a negative grid input, half-even candidate center rounding including positive and negative ties, negative radius, and duration/gap normal, zero, small, and large caps.
- `fixtures/quantization_candidate_pair_primitives.tsv:2` through `fixtures/quantization_candidate_pair_primitives.tsv:10` cover note-pair duration clamping, reversed ranges, missing and empty lyrics, increasing/overlapping/reversed gap annotation, candidate-pair sorting, and exclusion of `end <= start`.
- `bootstrap/quantization_candidate_primitives.md:38` through `bootstrap/quantization_candidate_primitives.md:40` and `dependencies/quantization_candidate_primitives.yaml:44` through `dependencies/quantization_candidate_primitives.yaml:56` keep Bayes candidate filtering, cost functions, DP decode, segmentation, note mutation, runtime dispatch, and GUI/Web/application promotion outside this unit. A grep of `rust/crates/v2m-core/src/quant.rs` found no Bayes/DP cost or decode implementation folded into this unit.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_quantization_candidate_primitives.py`: passed.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml quant_candidate`: passed; 3 tests passed, 0 failed.
- `rg -n "bayes|Bayes|decode|local_cost|metrical|note_value|smart|phrase|dp_asym|segment_split|piece_specific|prior" rust/crates/v2m-core/src/quant.rs bootstrap/quantization_candidate_primitives.md dependencies/quantization_candidate_primitives.yaml fixtures/quantization_candidate_*`: only boundary documentation and the `resolve_segment_shift_candidates` doc comment mention deferred DP/Bayes terms; no folded cost/decode code found in Rust or fixtures.

## Residual Risk

This review covers the valid internal helper inputs represented by the fixtures.
It does not prove Python exception parity for invalid private-helper inputs such
as empty nearest-candidate lists or non-positive step/modulo values, which the
bootstrap record leaves to a future bridge or promotion boundary. Very large
tick values beyond normal quantizer ranges are not separately stress-tested for
Python float/NumPy ceil versus Rust `f64::ceil` precision. Smart duration DP,
phrase DP, Bayesian candidate filtering, priors, decode, overlap repair, and
runtime dispatch remain unreviewed here by design.

## Promotion Note

This behavior review does not block promotion of
`quantization_candidate_primitives`. The coordinator still needs the required
data/algorithm review and should update manifest state separately; this report
does not mark the manifest verified.
