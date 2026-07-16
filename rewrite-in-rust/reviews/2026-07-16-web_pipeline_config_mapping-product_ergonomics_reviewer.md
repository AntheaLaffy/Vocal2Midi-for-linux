# web_pipeline_config_mapping - product_ergonomics_reviewer

Date: 2026-07-16
Decision: pass-with-followups

## Findings

- Severity: medium
- Location: web_task_manager.py:399
- Issue: Web quantization controls remain visible, persisted, and submitted, but this mapping intentionally ignores `quantize_precision` and `quantize_algorithm`, leaving effective Web runtime quantization at `PipelineConfig` defaults of `16` / `"bayes"`. This is accepted for the current legacy-owned contract because the manifest explicitly preserves it, but it remains a user-facing mismatch before any UI copy, Web workflow, or Rust promotion treats those controls as effective.
- Evidence: The Web controls are rendered disabled with visible `"不量化"` / `"开发中"` choices at `Vocal2Midi Web.html:1245` through `Vocal2Midi Web.html:1259`; `collectConfig` submits those values at `Vocal2Midi Web.html:2064` and `Vocal2Midi Web.html:2065`; settings persistence also stores them at `Vocal2Midi Web.html:2950` and `Vocal2Midi Web.html:2951`; server defaults persist `quantize_precision="none"` and `quantize_algorithm="dev"` at `web_server.py:101` and `web_server.py:102`; `_build_config` constructs `PipelineConfig` without quantization overrides at `web_task_manager.py:399` through `web_task_manager.py:429`, so the effective defaults come from `application/config.py:60` and `application/config.py:61`; Rust preserves this at `rewrite-in-rust/rust/crates/v2m-core/src/web_config.rs:274` and `rewrite-in-rust/rust/crates/v2m-core/src/web_config.rs:275`; fixture line `rewrite-in-rust/fixtures/web_pipeline_config_mapping.jsonl:4` proves submitted `"1/64"` / `"dp"` still maps to `16` / `"bayes"`; `tests/test_quantization_caller_defaults_contract.py:345` locks the same ignored-field contract.
- Required fix: Before Web quantization settings are enabled, advertised, or made Rust-owned as user-effective behavior, update the product contract to either hide/document the no-op fields, change visible defaults to match the effective runtime, or map Web values into `PipelineConfig`; then refresh fixtures and rerun behavior/product reviews.

- Severity: low
- Location: web_task_manager.py:376
- Issue: Invalid Web slice bounds fall back to application defaults, but the only warning is printed during config construction before stdout/stderr are redirected to WebSocket logs. Operators may see the console warning, but the Web task log does not get a recovery message for the fallback.
- Evidence: `_build_config` prints the fallback warning at `web_task_manager.py:373` through `web_task_manager.py:378`; `_execute_pipeline` calls `_build_config` at `web_task_manager.py:202` through `web_task_manager.py:204`; stdout/stderr redirection is installed only later at `web_task_manager.py:209` through `web_task_manager.py:214`; fixture line `rewrite-in-rust/fixtures/web_pipeline_config_mapping.jsonl:3` proves the fallback fields become `5.0` / `10.0`; the Python checker passed while emitting the expected console warning. Rust preserves fallback without a message at `rewrite-in-rust/rust/crates/v2m-core/src/web_config.rs:228` through `rewrite-in-rust/rust/crates/v2m-core/src/web_config.rs:233`.
- Required fix: No blocking fix for this legacy-owned mapping review. If the Web workflow needs visible recovery feedback, move the warning into task logging or return structured fallback diagnostics in a future bridge/product contract, then update this unit's fixtures.

Reviewer separation was preserved. This report covers exactly `product_ergonomics_reviewer` for `web_pipeline_config_mapping`; I reviewed only and did not edit production code, fixtures, bootstrap scripts, Rust source, or the manifest.

## Checks

- `UV_CACHE_DIR=/tmp/v2m-uv-cache uv run python rewrite-in-rust/bootstrap/check_web_pipeline_config_mapping.py`: passed; emitted the expected invalid-slice fallback warning.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml web_config`: passed, 2 tests.
- `UV_CACHE_DIR=/tmp/v2m-uv-cache uv run pytest tests/test_web_api.py tests/test_quantization_caller_defaults_contract.py`: passed, 90 tests with 3 existing `audioread` stdlib deprecation warnings.
- Targeted source/fixture inspection covered `rewrite-in-rust/manifest.yaml:414`, `rewrite-in-rust/records/0018-confirm-web-pipeline-config-mapping-boundary.md`, `rewrite-in-rust/dependencies/web_pipeline_config_mapping.yaml`, `rewrite-in-rust/bootstrap/web_pipeline_config_mapping.md`, `web_task_manager.py`, `web_server.py`, `application/config.py`, `inference/device_utils.py`, `rewrite-in-rust/rust/crates/v2m-core/src/web_config.rs`, `rewrite-in-rust/fixtures/web_pipeline_config_mapping.jsonl`, and the relevant quantization/Web records.

## Residual Risk

This review is source/test based. I did not run a browser session, inspect live UI accessibility state, run a real model pipeline, or verify full Flask multipart request behavior beyond the existing Web API tests. Persisted settings merge remains a separate Web settings/API boundary, so this report should not be used as proof that a direct Rust mapping call by itself reproduces `/api/pipeline/start` merged defaults.

## Promotion Note

This role does not block coordinator use of the current unit as legacy-owned reimplementation evidence. It should block any promotion or product messaging that treats Web quantization controls as effective, or promises user-visible slice fallback recovery, until the follow-ups above are resolved or explicitly accepted. Runtime ownership remains legacy; the manifest was not marked verified.
