# 0014 - Confirm Quantization Pipeline Promotion Seam

## Context

`quantization_bridge_bootstrap` is verified and proves the CLI/subprocess JSON
bridge with `v2m-quant-bridge` and `inference/quant/rust_bridge.py`.

`quantization_caller_defaults_contract` is verified and locks current caller
behavior, including the GUI/Web quantization setting mismatch:

- desktop AutoLyric controls are disabled and pass `0/simple`;
- Web disabled controls submit/persist `none/dev`, but `PipelineConfig` still
  runs with effective `bayes/16` defaults.

The remaining manifest unit is `quantization_pipeline_promotion`. Source
inspection shows production routing still calls legacy `quantize_notes` directly
from `inference/pipeline/auto_lyric_hybrid.py` after sorting and before export.

## Decision

Confirm `quantization_pipeline_promotion` as one pipeline-seam unit. Do not
split it again and do not broaden it into GUI/Web/product setting work, model
runtime work, or another quantization algorithm port.

The accepted promotion seam is:

- keep `should_apply_quantization` as the activation gate;
- route the existing post-GAME, pre-export quantization call through
  `quantize_notes_with_backend`;
- use the already verified `rust-json` CLI bridge as the Rust backend;
- expose backend selection, bridge executable path, timeout, and cancellation
  context at the application/pipeline boundary;
- preserve a tested `legacy` backend for rollback;
- preserve disabled/ignored GUI/Web quantization settings unless a later product
  record updates `quantization_caller_defaults_contract`.

Do not change this unit to PyO3/maturin, ctypes/cdylib, or an HTTP/service
bridge without a new record.

## Consequences

- Future writer work can implement production routing without choosing a new
  architecture.
- A missing bridge executable is now a packaging/configuration problem for the
  promotion unit, not a reason to re-open algorithm work.
- Cancellation support is a promotion requirement because the current wrapper
  uses `subprocess.run` with timeout but no `cancel_checker`.
- The first implementation should remain reversible and should be marked
  `reimplemented`, not `verified` or `promoted`, until behavior, architecture,
  and product ergonomics reviews pass.

## Reversal

Rollback is selecting `legacy` backend and leaving
`inference.quant.quantization` as the runtime implementation.

If CLI/subprocess JSON proves too slow or too difficult to package, record a new
bridge decision before introducing PyO3/maturin, ctypes/cdylib, or a service
bridge. If GUI/Web quantization settings become user-effective, update and
review `quantization_caller_defaults_contract` before promoting runtime
ownership.
