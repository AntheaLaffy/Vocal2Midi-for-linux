# 0007 - Split Quantization Core

## Context

The provisional `quantization_core_later` unit pointed at
`inference/quant/quantization.py`. After earlier fixture workflow was proven,
dependency discovery showed that this file contains several different behavior
shapes:

- activation policy: `should_apply_quantization`
- public dispatch: `quantize_notes`
- simple grid snapping
- candidate and pair primitives
- smart duration dynamic programming
- phrase DP/asymmetric quantization
- Bayesian segmentation, priors, and decode
- later GUI/Web/application/pipeline promotion behavior

The file imports NumPy, but not every capability needs NumPy. The activation
policy is pure string/integer logic, while smart, DP, and Bayesian algorithms
need separate numeric fixture coverage and review.

## Decision

Replace `quantization_core_later` with smaller migration units:

- `quantization_activation_policy`
- `quantization_simple_grid_core`
- `quantization_candidate_primitives`
- `quantization_smart_duration_dp`
- `quantization_phrase_dp_core`
- `quantization_bayesian_core`
- `quantization_pipeline_promotion`

Start with `quantization_activation_policy`, covering only
`should_apply_quantization`.

## Consequences

- The next implementation target is a small policy predicate with a clear TSV
  fixture table and no Rust dependency additions.
- Simple grid quantization should be next after activation policy; it must
  preserve Python half-even rounding and in-place note mutation semantics.
- Smart, DP, and Bayesian units require data/algorithm review before they can
  be verified.
- Runtime promotion remains separate because GUI/Web/application defaults and
  pipeline routing are product behavior, not just algorithm behavior.

## Reversal

If future fixture work proves two quantization units need a shared Rust data
model, add that model as a small prerequisite unit or merge those specific
units in a new record. Do not silently re-expand the activation-policy unit.
