# 0008 - Confirm Smart Duration DP Boundary

## Context

The manifest split quantization into smaller units, leaving
`quantization_smart_duration_dp` provisional until candidate primitives were
verified. Dependency discovery showed that `_quantize_notes_smart` uses NumPy
for local DP storage and `argmin`, but it does not require NumPy array kernels,
SciPy, model inference, export code, or runtime bridge behavior.

The smart path depends on already verified primitives:

- half-even tick conversion from the simple-grid unit
- `_build_duration_candidates` from the candidate-primitives unit

Rest placement is inline in `_quantize_notes_smart`; it does not use
`_build_gap_candidates`.

## Decision

Confirm `quantization_smart_duration_dp` as a narrow implementation unit
covering only `_quantize_notes_smart`.

Implement the dynamic programming table directly in Rust using vectors rather
than adding `ndarray`, NumPy bindings, PyO3, or a subprocess bridge. Preserve
NumPy `argmin` first-min tie behavior with explicit first-min scans.

Keep these capabilities separate:

- phrase DP/asymmetric quantization
- Bayesian quantization
- public `quantize_notes` dispatch
- GUI/Web/application quantization settings
- runtime promotion and bridge design

## Consequences

- Smart mode can be fixture-bound with golden note-sequence outputs.
- No new Rust dependencies are needed for this unit.
- Future phrase DP and Bayesian units may reuse the same note fixture shape but
  must receive their own data/algorithm review.
- Runtime ownership remains legacy Python until a promotion unit explicitly
  changes production routing.

## Reversal

If later promotion requires shared quantization data structures, extract that
shared model as a separate unit and keep the smart-duration fixtures as parity
evidence. Do not expand this unit silently to include public dispatch or other
quantization modes.
