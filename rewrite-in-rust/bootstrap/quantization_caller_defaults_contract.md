# quantization_caller_defaults_contract Bootstrap

## Boundary

`quantization_caller_defaults_contract` should lock the public caller behavior
that runtime promotion must preserve or intentionally change.

It covers:

- `inference.quant.quantization.should_apply_quantization`
- `inference.quant.quantization.quantize_notes`
- pipeline gating before export
- `PipelineConfig` defaults and `to_kwargs` pass-through
- GUI quantization parsing and current disabled AutoLyric behavior
- Web persisted/request settings and effective `PipelineConfig` defaults

It does not cover algorithm internals, model inference, slicing, ASR, GAME,
export format internals, Flask replacement, GUI rewrite, or final Rust bridge
implementation.

## Current Caller Behavior

- `should_apply_quantization` lowercases mode but does not trim whitespace.
  Exact `dp` applies even with step `0`; every other mode applies only when
  `quantization_step > 0`.
- `quantize_notes` lowercases mode but does not trim whitespace. It dispatches
  `smart`, `bayes`, and `dp`; every other mode uses simple fallback.
- The pipeline sorts notes before quantization, gates with
  `should_apply_quantization`, then exports the same mutated list to all output
  formats.
- `PipelineConfig` defaults to `quantization_step=16` and
  `quantization_mode="bayes"`.
- The desktop AutoLyric view currently disables quantization controls and passes
  `quantization_step=0`, `quantization_mode="simple"`.
- `gui/fluent_utils.py` maps visible labels to ticks and modes, including
  `"开发中"` to `bayes`, but that parser is not the active AutoLyric config path.
- Web settings expose `quantize_precision="none"` and
  `quantize_algorithm="dev"`, but the current Web task config builder does not
  map those values into `PipelineConfig`; effective Web runtime uses
  `PipelineConfig` defaults.

## Implemented Contract Artifacts

- `rewrite-in-rust/fixtures/quantization_caller_defaults_contract.tsv` covers
  activation and dispatch parity for `None`, empty, `simple`, `smart`, `bayes`,
  `dp`, unknown, uppercase, whitespace-padded modes, and positive/zero/negative
  steps.
- `tests/test_quantization_caller_defaults_contract.py` locks:
  - public `should_apply_quantization` and `quantize_notes` dispatch behavior;
  - pipeline no-quantize behavior, unknown positive-mode simple fallback, `dp`
    with step `0`, and export ordering after quantization mutation;
  - `PipelineConfig` defaults and explicit pass-through;
  - current GUI disabled quantization controls and parser mappings;
  - current Web `quantize_precision` / `quantize_algorithm` ignored-field
    behavior.

Copyable contract check:

```bash
uv run pytest tests/test_quantization_caller_defaults_contract.py
```

The contract intentionally preserves current behavior. Any later product change
to GUI or Web quantization defaults should update this unit before runtime
promotion.

## Rollback

Rollback is keeping all current Python callers unchanged and leaving
`inference.quant.quantization` as runtime owner.
