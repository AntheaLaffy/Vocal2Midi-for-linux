# 0010 - Confirm Bayesian Quantization Boundary

## Context

After `quantization_phrase_dp_core` was verified, the next manifest unit is
`quantization_bayesian_core`. The Python source groups Bayes-specific candidate
filtering, phase-center estimation, piece-specific priors, local costs,
segment-level decode, final overlap repair, and note mutation around
`_quantize_notes_bayesian`.

The file imports NumPy. Unlike the phrase-DP path, this unit does call NumPy,
but only for small-vector median, mean, absolute-difference, and scalar ceil
behavior. It does not call model runtimes, BLAS-scale array kernels, audio
libraries, GUI/Web handlers, or application routing.

## Decision

Confirm `quantization_bayesian_core` as one coherent algorithm unit covering
`_quantize_notes_bayesian` and its Bayes-specific helpers.

Do not split the private Bayes helpers into separate manifest units now. The
candidate filtering, priors, phase center, local cost, decode, and overlap
repair are tightly coupled to the final Bayesian quantized note sequence and do
not expose independent runtime promotion boundaries.

Do not add NumPy, ndarray, PyO3, subprocess, CLI, HTTP, or runtime-router
dependencies. The future writer should implement the algorithm directly in the
independent Rust crate with helper-level and golden sequence fixtures.

Keep these capabilities separate:

- public `quantize_notes` mode dispatch
- GUI/Web/application quantization settings
- runtime promotion and bridge design
- simple grid, candidate primitives, smart duration DP, and phrase DP behavior
  already verified in earlier units

## Consequences

- Writer work must start by adding Bayesian helper fixtures, golden note
  sequence fixtures, and a Python checker before implementing Rust logic.
- A small NumPy-compatible median helper is enough for this unit; no Rust
  numerical dependency is justified by the selected behavior.
- Data/algorithm and Rust-style reviews remain required because this unit
  contains grouping, numeric penalties, nested DP, ordering-sensitive ties, and
  overlap repair.
- Runtime ownership remains legacy Python until a promotion unit explicitly
  changes production routing.

## Reversal

If fixture work shows that one helper needs a reusable public Rust data model,
extract that model as a prerequisite unit. If future review finds the helper set
too broad to verify, split only the helper with independent fixture value; do
not expand this unit into public dispatch or pipeline promotion.
