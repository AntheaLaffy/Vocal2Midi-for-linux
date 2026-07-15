# quantization_pipeline_promotion - product_ergonomics_reviewer

Date: 2026-07-15
Decision: pass-with-followups

## Findings

- Severity: medium
- Location: inference/quant/rust_bridge.py:42
- Issue: `rust-json` remains an explicit operator/config backend and requires a bridge binary path from `V2M_QUANT_BRIDGE_BIN` or `executable=`. This is acceptable for verification while the effective default remains legacy, but a future default-Rust release would create operator burden unless packaging/configuration is handled before users encounter the path error.
- Evidence: Backend selection defaults to `legacy` when no backend or `V2M_QUANT_BACKEND` is provided at `inference/quant/rust_bridge.py:42`, and the explicit legacy path still calls the Python backend at `inference/quant/rust_bridge.py:43` through `inference/quant/rust_bridge.py:47`. `rust-json` requires an executable and raises a bridge-missing message if neither explicit config nor `V2M_QUANT_BRIDGE_BIN` exists at `inference/quant/rust_bridge.py:67` through `inference/quant/rust_bridge.py:72`. `PipelineConfig` keeps backend/bin/timeout defaults legacy-compatible at `application/config.py:62` through `application/config.py:64` and passes them through at `application/config.py:100` through `application/config.py:102`; the pipeline forwards them to the wrapper at `inference/pipeline/auto_lyric_hybrid.py:447` through `inference/pipeline/auto_lyric_hybrid.py:457`. The bootstrap explicitly forbids runtime `cargo build` and requires explicit config or `V2M_QUANT_BRIDGE_BIN` at `rewrite-in-rust/bootstrap/quantization_pipeline_promotion.md:99` through `rewrite-in-rust/bootstrap/quantization_pipeline_promotion.md:106`. Tests cover explicit Rust configuration, explicit legacy rollback, and missing/non-executable bridge paths at `tests/test_quantization_pipeline_promotion.py:150`, `tests/test_quantization_pipeline_promotion.py:178`, and `tests/test_quantization_pipeline_promotion.py:279`.
- Required fix: Before user-facing default-Rust promotion, package or configure `v2m-quant-bridge`, document the operator path requirement, and prefer a concise bridge-missing/error message in GUI/Web flows. Keep `legacy` as the explicit rollback path.

- Severity: low
- Location: web_task_manager.py:399
- Issue: GUI/Web visible disabled quantization controls remain intentionally unchanged. Desktop hardcodes no quantization from disabled controls, while Web submits/persists disabled `none/dev` fields that are still ignored by `PipelineConfig`. This does not block this review because the manifest and bootstrap explicitly require preserving or separately updating that contract before treating those settings as user-effective.
- Evidence: Desktop controls are disabled at `gui/auto_lyric_view.py:189` and `gui/auto_lyric_view.py:197`, then the active AutoLyric config passes `quantization_step=0` and `quantization_mode="simple"` at `gui/auto_lyric_view.py:394` and `gui/auto_lyric_view.py:395`. Web controls are disabled at `Vocal2Midi Web.html:1245` through `Vocal2Midi Web.html:1258`, yet their values are collected/persisted at `Vocal2Midi Web.html:2064` and `Vocal2Midi Web.html:2065`; `web_task_manager.py` builds `PipelineConfig` without mapping those Web quantization fields at `web_task_manager.py:399` through `web_task_manager.py:429`, leaving effective defaults from `application/config.py:60` and `application/config.py:61`. The manifest names this as a required preservation/resolution condition at `rewrite-in-rust/manifest.yaml:359`, and the bootstrap records the intended preservation at `rewrite-in-rust/bootstrap/quantization_pipeline_promotion.md:150` through `rewrite-in-rust/bootstrap/quantization_pipeline_promotion.md:165`. Contract tests lock the GUI parser/disabled state and Web ignored-field behavior at `tests/test_quantization_caller_defaults_contract.py:289`, `tests/test_quantization_caller_defaults_contract.py:305`, and `tests/test_quantization_caller_defaults_contract.py:345`.
- Required fix: Keep these controls disabled/ignored for this promotion path, or update `quantization_caller_defaults_contract`, its tests, and product ergonomics review before enabling or advertising GUI/Web quantization settings as user-effective.

- Severity: low
- Location: web_task_manager.py:279
- Issue: User-visible error and cancellation paths are acceptable for explicit `rust-json` verification, including the stdin-delivery timeout/cancel fix, but they still use the existing generic pipeline error style. This is adequate while Rust is opt-in; default-Rust promotion should make bridge-missing/timeout errors clearer for non-operator users.
- Evidence: The fixed bridge starts a child process at `inference/quant/rust_bridge.py:112` through `inference/quant/rust_bridge.py:119`, polls cancellation before each bounded communication attempt at `inference/quant/rust_bridge.py:127` through `inference/quant/rust_bridge.py:147`, and kills the child at `inference/quant/rust_bridge.py:171` through `inference/quant/rust_bridge.py:179`. The application layer maps `InterruptedError` into `CancellationError` at `application/pipeline.py:52` through `application/pipeline.py:55`, Web tasks surface cancellation or `task.error` over SocketIO at `web_task_manager.py:261` through `web_task_manager.py:296`, and the desktop worker emits cancellation or traceback-backed errors at `gui/fluent_worker.py:81` through `gui/fluent_worker.py:84`. Tests now cover timeout on a non-reading child during stdin delivery and cancellation of a non-reading child at `tests/test_quantization_pipeline_promotion.py:312` and `tests/test_quantization_pipeline_promotion.py:365`.
- Required fix: No blocking fix for this explicit-backend verification. Before default-Rust promotion, add a product-facing bridge error mapping if the generic pipeline traceback/message is not acceptable for GUI/Web users.

Reviewer separation was preserved. I reviewed exactly `product_ergonomics_reviewer` for `quantization_pipeline_promotion` and wrote only this report. I did not edit production code, tests, manifest, bootstrap/dependency records, or Rust code.

## Checks

- `PYTHONDONTWRITEBYTECODE=1 uv run pytest tests/test_quantization_pipeline_promotion.py tests/test_web_api.py -q -p no:cacheprovider`: passed, 66 tests, 3 deprecation warnings from `audioread`.
- `PYTHONDONTWRITEBYTECODE=1 uv run pytest tests/test_quantization_caller_defaults_contract.py -q -p no:cacheprovider`: passed, 37 tests.
- `PYTHONDONTWRITEBYTECODE=1 uv run python rewrite-in-rust/bootstrap/check_quantization_bridge_bootstrap.py`: passed with no output.
- `rg -n "quantization_backend|quantization_bridge_bin|quantization_timeout_sec|V2M_QUANT_BACKEND|V2M_QUANT_BRIDGE_BIN|rust-json|quantize_notes_with_backend|quantize_precision|quantize_algorithm" application inference gui web_server.py web_task_manager.py 'Vocal2Midi Web.html' tests/test_quantization_pipeline_promotion.py tests/test_quantization_caller_defaults_contract.py`: inspected backend selection, bridge-bin config, Web ignored fields, and test coverage.
- `rg -n "setEnabled\\(False\\)|quantization_step=0|quantization_mode=\\\"simple\\\"|quantizePrecision|quantizeAlgorithm|_build_config|PipelineConfig\\(" gui/auto_lyric_view.py 'Vocal2Midi Web.html' web_task_manager.py tests/test_quantization_caller_defaults_contract.py application/config.py inference/pipeline/auto_lyric_hybrid.py`: inspected disabled GUI/Web controls and ignored Web config mapping.
- `git diff --check`: passed.
- `git status --short`: inspected the dirty worktree before writing this report; existing code/test/record artifacts were left untouched.

## Residual Risk

This review is source/test based. I did not launch the PyQt GUI, run a browser session, build a distributable package, or run a full model pipeline with a real `v2m-quant-bridge` binary through Web/GUI. Packaging and final user-facing error wording remain unproven, so they should be handled before making Rust the default backend.

## Promotion Note

This role does not block coordinator verification of the current reimplemented unit while the effective default remains legacy and `rust-json` is explicit. It should block a user-facing default-Rust promotion until the bridge binary packaging/configuration requirement, generic GUI/Web bridge error wording, and preserved GUI/Web quantization-control mismatch are resolved or explicitly accepted for that release. The manifest was not marked verified.
