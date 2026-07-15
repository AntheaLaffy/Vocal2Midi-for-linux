# 0013 - Lock Quantization Caller Defaults Contract

## Context

`quantization_bridge_bootstrap` proved an opt-in Python-to-Rust JSON bridge, but
runtime promotion still depends on caller behavior outside the algorithm core.
The current Python callers expose several separate defaults:

- public dispatch lowercases modes without trimming whitespace;
- `dp` applies even with step `0`;
- `PipelineConfig` defaults to Bayes mode with step `16`;
- desktop AutoLyric disables quantization controls and passes simple mode with
  step `0`;
- Web settings expose quantization fields that are currently not mapped into
  `PipelineConfig`.

## Decision

Keep the current behavior as the compatibility contract for this unit. Add a
fixture table and focused Python tests instead of changing GUI, Web, pipeline,
or bridge routing behavior.

The contract tests use fake notes and monkeypatched pipeline dependencies so
they can reach the quantization/export boundary without model inference.

## Consequences

- `quantization_caller_defaults_contract` can move to `reimplemented` and wait
  for behavior and product-ergonomics review.
- `quantization_pipeline_promotion` remains planned and must preserve this
  caller/default contract or record an intentional product change first.
- No production caller now routes to Rust by default.

## Reversal

If product owners decide to enable GUI/Web quantization controls or map Web
quantization fields into `PipelineConfig`, update this contract and rerun review
before promoting runtime ownership.
