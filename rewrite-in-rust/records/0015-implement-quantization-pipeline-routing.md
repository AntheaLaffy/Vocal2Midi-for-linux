# 0015 - Implement Quantization Pipeline Routing

## Context

`quantization_pipeline_promotion` was confirmed as the final quantization
runtime seam after the algorithm units, JSON bridge bootstrap, and
caller/defaults contract were verified.

The accepted seam is the existing post-GAME, pre-export quantization point in
`inference/pipeline/auto_lyric_hybrid.py`. The first implementation needed to
make that seam route through the verified wrapper without making Rust the
unconditional default before independent review.

## Decision

Route production quantization through
`inference.quant.rust_bridge.quantize_notes_with_backend`.

Add optional pipeline/application config fields for:

- quantization backend;
- bridge executable path;
- bridge timeout.

Keep the effective default rollbackable. An empty backend defers to the wrapper,
which defaults to legacy unless `V2M_QUANT_BACKEND` is explicitly set. Passing
`legacy` remains the explicit rollback path.

Extend the wrapper to support cancellation around the subprocess bridge. The
Rust JSON backend now starts a child process and communicates with bounded
`communicate` calls so timeout and cancellation cover stdin delivery, stdout and
stderr collection, and child termination. The bridge remains one process per
quantization operation.

## Consequences

- The unit can move to `reimplemented`; it is not verified or promoted until
  behavior, architecture, and product ergonomics reviews pass.
- GUI and Web quantization settings remain locked by the verified caller/default
  contract. The writer did not make disabled/ignored settings user-effective.
- Missing bridge executables, non-executable paths, unsupported backends,
  timeouts, non-reading child processes, and cancellation are now tested at the
  promotion seam.

## Reversal

Rollback is selecting the `legacy` backend, or leaving backend unset without
`V2M_QUANT_BACKEND=rust-json`. `inference.quant.quantization` remains available
as the legacy implementation.
