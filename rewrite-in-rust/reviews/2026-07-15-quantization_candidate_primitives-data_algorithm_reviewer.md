# quantization_candidate_primitives - data_algorithm_reviewer

Date: 2026-07-15
Decision: pass

## Findings

No findings.

Evidence:

- The reviewed boundary is the pure helper set only: grid-step resolution, segment shifts, nearest/mod/grid distance helpers, candidate values, note-pair construction, gap annotation, candidate pairs, and duration/gap candidate lists are documented in `bootstrap/quantization_candidate_primitives.md:5`. The same document keeps Bayesian filtering, local costs, metrical penalties, DP decode, note mutation, dispatch, and GUI/Web/application promotion out of scope at `bootstrap/quantization_candidate_primitives.md:38`.
- Numeric helper parity is covered against the Python source. Python uses half-even `round` for candidate centers at `../inference/quant/quantization.py:87`, positive-modulo/grid helpers at `../inference/quant/quantization.py:77`, and first-on-tie `min(..., key=...)` nearest selection at `../inference/quant/quantization.py:73`. Rust mirrors those with strict tie preservation in `nearest_candidate`, positive `rem_euclid` grid distance, and explicit half-even rounding at `rust/crates/v2m-core/src/quant.rs:138`, `rust/crates/v2m-core/src/quant.rs:150`, and `rust/crates/v2m-core/src/quant.rs:273`.
- Sorted unique behavior is preserved. Python uses `sorted(set(...))` for candidate values and candidate pairs at `../inference/quant/quantization.py:89` and `../inference/quant/quantization.py:121`; Rust uses `BTreeSet` for candidate values and pairs at `rust/crates/v2m-core/src/quant.rs:174` and `rust/crates/v2m-core/src/quant.rs:220`.
- Duration and gap scalar cap behavior matches the Python `np.ceil` formulas at `../inference/quant/quantization.py:440` and `../inference/quant/quantization.py:449`. Rust uses the same multiplier tables and scalar `ceil` caps at `rust/crates/v2m-core/src/quant.rs:233` and `rust/crates/v2m-core/src/quant.rs:251`.
- Note-pair and gap data representation is a narrow typed equivalent of the legacy dict shape. Python clamps `raw_dur` and lyric fallback at `../inference/quant/quantization.py:93`, then annotates non-negative previous-end gaps at `../inference/quant/quantization.py:102`; Rust exposes `RawNotePair` and `GapAnnotatedNotePair` with the same fields and calculations at `rust/crates/v2m-core/src/quant.rs:22`, `rust/crates/v2m-core/src/quant.rs:182`, and `rust/crates/v2m-core/src/quant.rs:196`.
- Fixture coverage hits the requested algorithm edges: nearest tie order, negative grid input, half-even positive and negative ties, negative radius, normal/zero/small/large duration and gap caps in `fixtures/quantization_candidate_scalar_primitives.tsv:9`; lyric fallback, minimum/reversed duration, overlapping/reversed gap annotation, and `end > start` candidate-pair filtering in `fixtures/quantization_candidate_pair_primitives.tsv:2`.
- The Rust module does not introduce Bayes filtering, local-cost evaluation, segmentation, DP decode, or new note mutation for this unit. Those algorithms remain in Python at `../inference/quant/quantization.py:128`, `../inference/quant/quantization.py:208`, `../inference/quant/quantization.py:311`, `../inference/quant/quantization.py:648`, and `../inference/quant/quantization.py:727`, and are explicitly listed as legacy-kept in `dependencies/quantization_candidate_primitives.yaml:44`.

## Checks

- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml quant_candidate`: passed; 3 tests passed, 28 filtered out.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml quant`: passed; 9 tests passed, 22 filtered out.
- `uv run python rewrite-in-rust/bootstrap/check_quantization_candidate_primitives.py`: passed; exited with status 0.

## Residual Risk

This is a fixture-bound helper review. It does not prove Python-compatible error mapping for invalid public inputs such as empty nearest-candidate lists, non-positive steps/moduli, arbitrary dynamic lyric objects, non-finite floats, or very large values outside normal MIDI tick ranges. It also does not review smart duration DP, phrase/asymmetric DP, Bayesian priors/decode, public `quantize_notes` dispatch, bridge behavior, or runtime promotion.

The manifest entry still shows `quantization_candidate_primitives` as `planned` and `provisional` at `manifest.yaml:192`; coordinator state update and verification evidence insertion remain separate from this reviewer role.

## Promotion Note

This `data_algorithm_reviewer` role does not block coordinator state update for `quantization_candidate_primitives`. The coordinator still needs the required behavior review before marking the unit verified, and production runtime ownership should remain legacy Python until a later promotion unit verifies a bridge.
