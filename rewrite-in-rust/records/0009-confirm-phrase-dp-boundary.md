# 0009 - Confirm Phrase DP Boundary

## Context

After `quantization_smart_duration_dp` was verified, the next manifest unit is
`quantization_phrase_dp_core`. The Python source groups several private helper
functions around `_quantize_notes_phrase_hybrid`:

- asymmetric local cost
- phrase segment splitting
- center adjustment
- segment decode with center candidates
- segment-level center-option dynamic programming
- final overlap repair and note mutation

The file imports NumPy, but this phrase-DP path does not call NumPy APIs. Its
behavior depends on Python scalar arithmetic, list/dict ordering, tuple
candidate ordering, and helper behavior already fixture-tested in earlier
quantization units.

## Decision

Confirm `quantization_phrase_dp_core` as one coherent algorithm unit covering
`_quantize_notes_phrase_hybrid` and its phrase-specific helpers.

Do not split the private helpers into separate manifest units at this point.
Although the boundary is larger than the earlier scalar units, the helpers are
tightly coupled to the phrase-DP output and do not expose an independent runtime
promotion boundary.

Do not add NumPy, ndarray, PyO3, subprocess, CLI, HTTP, or runtime-router
dependencies. The future writer should implement the algorithm directly in the
independent Rust crate with helper-level and golden sequence fixtures.

Keep these capabilities separate:

- Bayesian candidate filtering, priors, phase-center estimation, local costs,
  and decode
- public `quantize_notes` mode dispatch
- GUI/Web/application quantization settings
- runtime promotion and bridge design

## Consequences

- Writer work must start by adding phrase-DP helper fixtures and golden
  note-sequence fixtures before implementing Rust logic.
- Data/algorithm review remains required because this unit contains nested DP,
  cost weighting, center switching, and overlap repair.
- Runtime ownership remains legacy Python until a promotion unit explicitly
  changes production routing.

## Reversal

If fixture work shows that a helper needs a reusable public Rust data model,
extract that model as a prerequisite unit. If a future review finds the helper
set too broad to verify, split only the helper with independent fixture value;
do not expand this unit into Bayesian quantization or public dispatch.
