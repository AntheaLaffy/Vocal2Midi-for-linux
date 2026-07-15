# quantization_pipeline_promotion - behavior_reviewer

Date: 2026-07-15
Decision: pass

## Findings

No behavior findings.

The previous high finding is fixed. `_run_bridge` now starts the child with
`subprocess.Popen`, establishes the deadline before communication, and polls
cancellation and timeout around `process.communicate(input=...)`
(`inference/quant/rust_bridge.py:113`, `inference/quant/rust_bridge.py:127`,
`inference/quant/rust_bridge.py:130`, `inference/quant/rust_bridge.py:134`,
`inference/quant/rust_bridge.py:140`). On timeout or cancellation it kills and
waits for the child (`inference/quant/rust_bridge.py:131`,
`inference/quant/rust_bridge.py:136`, `inference/quant/rust_bridge.py:171`).
Focused tests now cover a child that starts but never reads stdin for both the
timeout and cancellation paths (`tests/test_quantization_pipeline_promotion.py:312`,
`tests/test_quantization_pipeline_promotion.py:365`). A separate targeted probe
with a larger stdin payload returned through the timeout path in 0.121s and the
cancel path in 0.070s, with both child processes killed.

The reviewed behavior scope matches the promotion seam:

- Pipeline routing remains at the post-GAME, pre-export seam, gated by
  `should_apply_quantization`, then routed through `quantize_notes_with_backend`
  with mode, step, backend, executable, timeout, and cancel context
  (`inference/pipeline/auto_lyric_hybrid.py:439`,
  `inference/pipeline/auto_lyric_hybrid.py:447`,
  `inference/pipeline/auto_lyric_hybrid.py:448`,
  `inference/pipeline/auto_lyric_hybrid.py:452`,
  `inference/pipeline/auto_lyric_hybrid.py:453`,
  `inference/pipeline/auto_lyric_hybrid.py:454`,
  `inference/pipeline/auto_lyric_hybrid.py:455`,
  `inference/pipeline/auto_lyric_hybrid.py:456`).
- The same mutated/reordered `all_notes` list is exported after quantization to
  MIDI, TXT, CSV, and USTX (`inference/pipeline/auto_lyric_hybrid.py:458`,
  `inference/pipeline/auto_lyric_hybrid.py:463`,
  `inference/pipeline/auto_lyric_hybrid.py:466`,
  `inference/pipeline/auto_lyric_hybrid.py:468`,
  `inference/pipeline/auto_lyric_hybrid.py:470`), with focused coverage at
  `tests/test_quantization_pipeline_promotion.py:123` and
  `tests/test_quantization_caller_defaults_contract.py:211`.
- Default behavior remains legacy/rollbackable. Empty pipeline config is passed
  as `None`, and the wrapper selects `legacy` unless `V2M_QUANT_BACKEND` is set
  (`application/config.py:62`, `inference/pipeline/auto_lyric_hybrid.py:453`,
  `inference/quant/rust_bridge.py:42`, `inference/quant/rust_bridge.py:43`).
  Explicit `legacy` rollback with bridge configuration present is covered at
  `tests/test_quantization_pipeline_promotion.py:178`.
- Explicit `rust-json` routing requires a configured executable path or
  `V2M_QUANT_BRIDGE_BIN`, preserves bridge errors instead of silent legacy
  fallback, and maps nonzero/invalid bridge responses through
  `QuantizationBridgeError` (`inference/quant/rust_bridge.py:49`,
  `inference/quant/rust_bridge.py:52`, `inference/quant/rust_bridge.py:53`,
  `inference/quant/rust_bridge.py:71`, `inference/quant/rust_bridge.py:148`,
  `inference/quant/rust_bridge.py:157`). Missing and non-executable paths,
  timeout, and in-flight cancellation are covered at
  `tests/test_quantization_pipeline_promotion.py:279`,
  `tests/test_quantization_pipeline_promotion.py:291`,
  `tests/test_quantization_pipeline_promotion.py:334`.
- Activation parity is preserved: non-`dp` step zero is a no-op, `dp` step zero
  still quantizes, and unknown positive modes fall back to simple
  (`inference/quant/quantization.py:793`,
  `inference/quant/quantization.py:806`,
  `inference/quant/quantization.py:808`,
  `inference/quant/quantization.py:810`,
  `rewrite-in-rust/rust/crates/v2m-quant-bridge/src/main.rs:126`,
  `rewrite-in-rust/rust/crates/v2m-quant-bridge/src/main.rs:145`,
  `rewrite-in-rust/rust/crates/v2m-quant-bridge/src/main.rs:151`). The bridge
  fixtures include dp step-zero, disabled non-dp, unknown positive fallback,
  uppercase mode, padded dp disabled, null mode, and empty-note cases
  (`rewrite-in-rust/fixtures/quantization_bridge_payloads.jsonl:1`).
- GUI and Web caller/default behavior remains intentionally unchanged for this
  promotion unit: desktop quantization controls are disabled and still build
  `PipelineConfig` with `0/simple`; Web defaults persist `none/dev`, and
  `web_task_manager.py` still builds `PipelineConfig` without mapping those
  fields into effective quantization settings (`gui/auto_lyric_view.py:189`,
  `gui/auto_lyric_view.py:197`, `gui/auto_lyric_view.py:394`,
  `gui/auto_lyric_view.py:395`, `web_server.py:101`, `web_server.py:102`,
  `web_task_manager.py:399`, `web_task_manager.py:429`).

Reviewer separation was preserved. This report covers exactly
`behavior_reviewer` for `quantization_pipeline_promotion`; I reviewed only and
did not edit production code, tests, manifest, bootstrap/dependency records, or
Rust code.

## Checks

- `uv run pytest tests/test_quantization_pipeline_promotion.py tests/test_quantization_caller_defaults_contract.py -q`: passed, 50 tests.
- `uv run python rewrite-in-rust/bootstrap/check_quantization_bridge_bootstrap.py`: passed with no output.
- Targeted non-reading child timeout/cancel probe: passed. A fake bridge that starts and never reads stdin returned `bridge timed out` in 0.121s with `timeout_sec=0.1`; a matching cancellation probe returned `InterruptedError` in 0.070s. Both child processes were dead after the wrapper returned.
- `git diff --check`: passed.
- `rg -n "quantize_notes_with_backend|legacy_quantize_notes|V2M_QUANT_BACKEND|V2M_QUANT_BRIDGE_BIN|quantization_backend|quantization_bridge_bin|quantization_timeout_sec|rust-json" --glob '!third_party/**' --glob '!rewrite-in-rust/rust/target/**' --glob '!*.pyc' .`: inspected backend routing/config references.
- `rg -n "quantize_precision|quantize_algorithm|quantization_step|quantization_mode|PipelineConfig" web_server.py web_task_manager.py gui/auto_lyric_view.py gui/fluent_utils.py application inference tests/test_quantization_pipeline_promotion.py tests/test_quantization_caller_defaults_contract.py`: inspected GUI/Web caller/default preservation.
- `rg -n "Popen|communicate|TimeoutExpired|_kill_process|cancel_checker|deadline|communicate_input" inference/quant/rust_bridge.py`: inspected subprocess timeout/cancellation ownership.
- `rg -n "unknown|dp|step zero|fallback|reorder|mutat|export|rust-json" tests/test_quantization_pipeline_promotion.py rewrite-in-rust/fixtures/quantization_bridge_payloads.jsonl rewrite-in-rust/fixtures/quantization_caller_defaults_contract.tsv rewrite-in-rust/rust/crates/v2m-quant-bridge/src/main.rs rewrite-in-rust/rust/crates/v2m-core/src/quant.rs`: inspected activation, fallback, ordering, and bridge dispatch evidence.

## Residual Risk

This pass did not run a full real-audio model pipeline, live PyQt GUI, browser
workflow, packaging build, or cargo rebuild. The bootstrap checker exercised the
available `v2m-quant-bridge` binary and the source inspections matched the
reviewed seam. Product ergonomics follow-ups around bridge binary packaging and
visible-but-ignored GUI/Web controls remain outside this behavior role.

## Promotion Note

This behavior role no longer blocks `quantization_pipeline_promotion`
verification. The manifest was not marked verified. Coordinator state updates
must still account for the separate required review roles and any follow-ups
before promotion.
