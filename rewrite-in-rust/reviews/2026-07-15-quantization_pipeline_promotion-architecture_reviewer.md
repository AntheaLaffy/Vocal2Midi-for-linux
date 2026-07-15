# quantization_pipeline_promotion - architecture_reviewer

Date: 2026-07-15
Decision: pass

## Findings

No architecture findings.

The previous high finding is fixed. `inference/quant/rust_bridge.py` no longer
writes the full JSON payload to child stdin before timeout/cancel ownership is
active. `_run_bridge` now sets the deadline before communication, passes the
payload to `process.communicate(..., timeout=min(0.05, remaining))` inside the
bounded loop, clears the per-call input after `TimeoutExpired`, checks
`cancel_checker` before each bounded communicate call, and kills the child on
cancel or timeout (`inference/quant/rust_bridge.py:127`,
`inference/quant/rust_bridge.py:130`, `inference/quant/rust_bridge.py:136`,
`inference/quant/rust_bridge.py:140`, `inference/quant/rust_bridge.py:145`).
The non-reading child regressions now cover both timeout and cancellation while
stdin delivery is in flight (`tests/test_quantization_pipeline_promotion.py:312`,
`tests/test_quantization_pipeline_promotion.py:365`).

The owner boundary remains narrow. The production seam is still the existing
post-GAME, pre-export quantization point, using the legacy activation gate and
then `quantize_notes_with_backend` before all exports consume the same
`all_notes` list (`inference/pipeline/auto_lyric_hybrid.py:447`,
`inference/pipeline/auto_lyric_hybrid.py:448`,
`inference/pipeline/auto_lyric_hybrid.py:463`). `PipelineConfig` carries only
backend, bridge path, and timeout control-plane fields through to the pipeline
(`application/config.py:62`, `application/config.py:100`). The wrapper keeps
`legacy` as the default/rollback backend unless `V2M_QUANT_BACKEND` or an
explicit backend selects `rust-json`, and `rust-json` requires an explicit
executable path or `V2M_QUANT_BRIDGE_BIN` (`inference/quant/rust_bridge.py:42`,
`inference/quant/rust_bridge.py:67`, `inference/quant/rust_bridge.py:71`).

The implementation did not introduce runtime Cargo/build behavior, a new bridge
architecture, GUI/Web setting activation, Flask routing migration, model
migration, or export ownership changes. GUI quantization controls remain
disabled and hardcode `0/simple` (`gui/auto_lyric_view.py:189`,
`gui/auto_lyric_view.py:197`, `gui/auto_lyric_view.py:394`), while Web still
persists disabled quantization fields but does not map them into `PipelineConfig`
(`web_server.py:101`, `web_task_manager.py:399`).

Reviewer separation was preserved. This report covers exactly
`architecture_reviewer` for `quantization_pipeline_promotion`; I wrote only this
assigned report and did not edit production code, tests, manifest,
bootstrap/dependency records, or Rust code.

## Checks

- `env PYTHONDONTWRITEBYTECODE=1 uv run pytest tests/test_quantization_pipeline_promotion.py tests/test_quantization_caller_defaults_contract.py -q -p no:cacheprovider`: passed, 50 tests.
- `env PYTHONDONTWRITEBYTECODE=1 uv run pytest tests/test_quantization_pipeline_promotion.py -q -p no:cacheprovider -k "non_reading or timeout or cancel"`: passed, 5 tests selected and 8 deselected.
- `env PYTHONDONTWRITEBYTECODE=1 uv run pytest tests/test_web_api.py -q -p no:cacheprovider`: passed, 53 tests.
- `env PYTHONDONTWRITEBYTECODE=1 uv run python rewrite-in-rust/bootstrap/check_quantization_bridge_bootstrap.py`: passed with no output.
- `cargo build --manifest-path rewrite-in-rust/rust/Cargo.toml --bin v2m-quant-bridge`: passed.
- `git diff --check`: passed.
- `rg -n "quantization_backend|quantization_bridge_bin|quantization_timeout_sec|quantize_precision|quantize_algorithm|quantize_notes_with_backend|V2M_QUANT_BACKEND|V2M_QUANT_BRIDGE_BIN|cargo|target/debug|subprocess|Popen|communicate|cancel_checker|_check_cancel|rust-json|legacy" application inference gui web_server.py web_task_manager.py tests/test_quantization_pipeline_promotion.py rewrite-in-rust/bootstrap/quantization_pipeline_promotion.md rewrite-in-rust/records/0014-confirm-quantization-pipeline-promotion-seam.md rewrite-in-rust/records/0015-implement-quantization-pipeline-routing.md`: inspected backend/config, runtime build, subprocess, cancellation, and GUI/Web mismatch references.
- `rg -n "quantize_notes_with_backend|rust_bridge|quantization_backend|quantization_bridge_bin|quantization_timeout_sec" --glob '!rewrite-in-rust/**' --glob '!third_party/**' --glob '!*.pyc' .`: confirmed production usage is limited to `application/config.py`, `inference/pipeline/auto_lyric_hybrid.py`, and `inference/quant/rust_bridge.py`.
- `rg -n "cargo build|cargo test|target/debug|v2m-quant-bridge|V2M_QUANT_BRIDGE_BIN" --glob '!rewrite-in-rust/**' --glob '!third_party/**' --glob '!*.pyc' .`: confirmed runtime code does not run Cargo or depend on a development `target/debug` path.
- `rg -n "from inference\\.quant\\.quantization import|\\bquantize_notes\\(|\\bshould_apply_quantization\\(" --glob '!third_party/**' --glob '!rewrite-in-rust/rust/target/**' --glob '!*.pyc' .`: inspected remaining direct quantization references; the production pipeline keeps only the activation gate and routes quantization through the wrapper.

## Residual Risk

This was a source/test architecture review. I did not launch the PyQt GUI, run a
live Flask server, run full model inference, build a distributable package, or
benchmark real large note sets. The bridge binary packaging/operator
configuration requirement remains a release/promotion risk before a default-Rust
user-facing promotion, but it does not block this architecture pass while the
effective default remains rollbackable to legacy.

## Promotion Note

This architecture role no longer blocks `quantization_pipeline_promotion`
verification after the stdin-delivery timeout/cancel fix. Do not mark the
manifest verified from this report alone; coordinator state updates still need
the required review set and current behavior evidence.
