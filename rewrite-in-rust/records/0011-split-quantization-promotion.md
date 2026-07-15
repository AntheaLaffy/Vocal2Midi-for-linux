# 0011 - Split Quantization Promotion

## Context

All algorithm units under `inference/quant/quantization.py` are now verified in
Rust:

- activation policy
- simple grid
- candidate primitives
- smart duration DP
- phrase DP/asymmetric
- Bayesian core

The remaining manifest unit, `quantization_pipeline_promotion`, is not another
algorithm port. It combines several distinct risks:

- no Python-to-Rust runtime bridge exists
- no payload, error, cancellation, or fallback contract exists
- public `quantize_notes` dispatch has mode and no-op edge cases
- GUI, Web, and `PipelineConfig` disagree on visible/effective quantization
  defaults
- production routing touches pipeline export timing and rollback behavior

## Decision

Split `quantization_pipeline_promotion` into prerequisites:

- `quantization_bridge_bootstrap`: prove a Python-to-Rust bridge contract while
  keeping legacy Python as the default runtime owner.
- `quantization_caller_defaults_contract`: lock or intentionally change public
  dispatch, application defaults, GUI behavior, and Web settings behavior.
- `quantization_pipeline_promotion`: final caller routing and runtime ownership
  change after the prerequisites pass.

Use a CLI/subprocess JSON bridge as the first bridge proof unless a later record
replaces that decision. Do not add PyO3/maturin, ctypes, or a service bridge
before the bootstrap unit proves a smaller seam is insufficient.

## Consequences

- No production Python caller is changed by this discovery pass.
- `quantization_pipeline_promotion` remains planned/provisional and must not be
  promoted until bridge and caller/default contracts are verified.
- The next implementation target should be either the bridge bootstrap proof or
  the caller/default contract tests, not direct runtime routing.
- Promotion review must include behavior, architecture, and product ergonomics.

## Reversal

If a future bridge proof shows that CLI/subprocess JSON is too slow or too
awkward to package, record a new seam decision before switching to PyO3/maturin
or another FFI strategy. If product owners decide to change GUI/Web defaults,
record that as an intentional product behavior change before promotion.
