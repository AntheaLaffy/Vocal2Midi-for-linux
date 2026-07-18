PASS

# asr_resample_poly_contract - data_algorithm_reviewer

Date: 2026-07-18
Decision: pass

## Findings

No blocking findings.

- Severity: none
- Location: rust/crates/v2m-core/src/asr_resample_poly.rs:37
- Issue: Rate validation, GCD reduction, identity handling, and output length math match the scoped SciPy default path.
- Evidence: Rust validates positive rates before the identity shortcut at `rust/crates/v2m-core/src/asr_resample_poly.rs:37`, reduces `target_rate/source_rate` by `gcd_i64` at `rust/crates/v2m-core/src/asr_resample_poly.rs:41`, returns an identity copy only after validation at `rust/crates/v2m-core/src/asr_resample_poly.rs:45`, computes `ceil(input.len() * up / down)` at `rust/crates/v2m-core/src/asr_resample_poly.rs:52`, and implements SciPy `_output_len` at `rust/crates/v2m-core/src/asr_resample_poly.rs:92`. SciPy validates before reduction/identity at `../third_party/sources/scipy-1.17.1/scipy/signal/_signaltools.py:3993`, reduces by `math.gcd` at `../third_party/sources/scipy-1.17.1/scipy/signal/_signaltools.py:4007`, returns an identity copy at `../third_party/sources/scipy-1.17.1/scipy/signal/_signaltools.py:4010`, and computes `n_out` at `../third_party/sources/scipy-1.17.1/scipy/signal/_signaltools.py:4013`.
- Required fix: none.

- Severity: none
- Location: rust/crates/v2m-core/src/asr_resample_poly.rs:53
- Issue: FIR design, scale point, float32 cast point, and Kaiser/Bessel I0 path match the contract.
- Evidence: The Rust path uses `half_len = 10 * max(up, down)`, cutoff `1 / max_rate`, `2 * half_len + 1` taps, Kaiser beta `5.0`, DC scaling by coefficient sum, float32 coefficient projection, and float32 multiplication by `up` at `rust/crates/v2m-core/src/asr_resample_poly.rs:53`, `rust/crates/v2m-core/src/asr_resample_poly.rs:56`, `rust/crates/v2m-core/src/asr_resample_poly.rs:57`, and `rust/crates/v2m-core/src/asr_resample_poly.rs:96`. SciPy's default path uses the same `max_rate`, cutoff, `half_len`, `firwin`, float32 cast, and `h *= up` order at `../third_party/sources/scipy-1.17.1/scipy/signal/_signaltools.py:4023`. `firwin` constructs lowpass bands, applies `get_window`, and scales at `../third_party/sources/scipy-1.17.1/scipy/signal/_fir_filter_design.py:535`, `../third_party/sources/scipy-1.17.1/scipy/signal/_fir_filter_design.py:553`, and `../third_party/sources/scipy-1.17.1/scipy/signal/_fir_filter_design.py:557`. Kaiser uses `special.i0` at `../third_party/sources/scipy-1.17.1/scipy/signal/windows/_windows.py:1317`; Rust's local I0 series is bounded to this beta-5 path at `rust/crates/v2m-core/src/asr_resample_poly.rs:128`.
- Required fix: none.

- Severity: none
- Location: rust/crates/v2m-core/src/asr_resample_poly.rs:61
- Issue: Pre-padding, post-padding, trim offset, phase layout, and constant-zero upfirdn behavior match SciPy's 1D default.
- Evidence: Rust uses SciPy's non-modulo `n_pre_pad = down - (half_len % down)`, `n_pre_remove = (half_len + n_pre_pad) / down`, post-pad loop, and final `n_pre_remove..n_pre_remove+n_out` slice at `rust/crates/v2m-core/src/asr_resample_poly.rs:61`. SciPy does the same at `../third_party/sources/scipy-1.17.1/scipy/signal/_signaltools.py:4036` and keeps the same slice at `../third_party/sources/scipy-1.17.1/scipy/signal/_signaltools.py:4072`. Rust's transposed/flipped phase layout at `rust/crates/v2m-core/src/asr_resample_poly.rs:151` matches SciPy `_pad_h` at `../third_party/sources/scipy-1.17.1/scipy/signal/_upfirdn.py:47`. Rust's two upfirdn loops at `rust/crates/v2m-core/src/asr_resample_poly.rs:164` follow the Cython constant-zero fast path and accumulation order at `../third_party/sources/scipy-1.17.1/scipy/signal/_upfirdn_apply.pyx:423`.
- Required fix: none.

- Severity: none
- Location: fixtures/asr_resample_poly_contract.jsonl:1
- Issue: Fixture strength is sufficient for this narrow data/algorithm gate.
- Evidence: The fixture file has 18 cases covering identity, short boundary ratios, GCD reduction, upsampling, single-sample, empty input, NaN/Inf propagation, invalid/equal invalid-rate errors, and four long steady-state projected cases. The long cases at `fixtures/asr_resample_poly_contract.jsonl:10`, `fixtures/asr_resample_poly_contract.jsonl:11`, `fixtures/asr_resample_poly_contract.jsonl:12`, and `fixtures/asr_resample_poly_contract.jsonl:13` assert output shape, head/interior/tail selected values, finite sum, and finite absolute sum. The Rust test uses absolute tolerance `2e-7` for selected finite samples and `1e-5` for finite sums at `rust/crates/v2m-core/src/asr_resample_poly.rs:229`, matching the bootstrap tolerance rationale at `bootstrap/asr_resample_poly_contract.md:105`.
- Required fix: none.

- Severity: none
- Location: rust/crates/v2m-core/src/asr_resample_poly.rs:48
- Issue: Empty input and non-finite values are handled consistently with the fixture-bound public projection.
- Evidence: Empty valid non-identity inputs return an empty vector at `rust/crates/v2m-core/src/asr_resample_poly.rs:48`, matching the SciPy fixture projection at `fixtures/asr_resample_poly_contract.jsonl:8`. NaN/Inf propagation is not special-cased, so f32 multiply/add naturally propagates non-finite values through the same FIR loop; fixture coverage at `fixtures/asr_resample_poly_contract.jsonl:9` and test comparison at `rust/crates/v2m-core/src/asr_resample_poly.rs:339` verify public sentinels rather than payload bits, consistent with `dependencies/asr_resample_poly_contract.yaml:68`.
- Required fix: none.

- Severity: none
- Location: rust/crates/v2m-core/src/asr_resample_poly.rs:52
- Issue: Complexity risk is acceptable for the scoped ASR sample rates.
- Evidence: The implementation has the same rate-reduced FIR/upfirdn asymptotic behavior as SciPy: filter length scales with `10 * max(up, down)` after GCD reduction, and output work scales with produced samples times taps per phase. The dependency record explicitly restricts this unit to project sample-rate integers and 1D float32 arrays at `dependencies/asr_resample_poly_contract.yaml:4`, rejects generic resampler crates because exact SciPy FIR/upfirdn parity is the public behavior at `dependencies/asr_resample_poly_contract.yaml:24`, and keeps broader resampling modes legacy-owned at `dependencies/asr_resample_poly_contract.yaml:89`.
- Required fix: none.

## Checks

- `cargo test --manifest-path rust/Cargo.toml asr_resample_poly_contract`: passed; 1 `v2m_core` test passed, 123 filtered out, 0 failed; bridge crate had 0 matching tests.
- `uv run python rewrite-in-rust/bootstrap/check_asr_resample_poly_contract.py`: passed; `asr_resample_poly_contract fixtures ok: 18 cases`.
- Source audit by inspection: compared Rust algorithm lines against SciPy 1.17.1 `_signaltools.py`, `_fir_filter_design.py`, `windows/_windows.py`, `_upfirdn.py`, and `_upfirdn_apply.pyx` for the scoped default path.

## Residual Risk

The long fixtures use selected samples plus finite aggregate sums instead of full-output golden arrays. That leaves a small chance of a localized mismatch outside selected indices, but the line-by-line implementation audit of FIR construction, phase layout, and upfirdn loops substantially lowers that risk for this narrow default path.

Very large artificial `target_rate/source_rate` values could overflow Rust `usize` products or allocate impractically large filters before returning an allocation failure or panic. The public seam is project ASR audio rates, so this is not a promotion blocker for this unit.

## Promotion Note

The `data_algorithm_reviewer` gate passes. Coordinator recommendation: record this review as passed for `asr_resample_poly_contract`, keep `current_owner: legacy`, and do not mark the unit `verified` until the remaining required `stage_behavior_reviewer` gate also passes.
