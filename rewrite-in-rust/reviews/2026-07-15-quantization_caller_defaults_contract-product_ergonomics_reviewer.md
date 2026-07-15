# quantization_caller_defaults_contract - product_ergonomics_reviewer

Date: 2026-07-15
Decision: pass-with-followups

## Findings

- Severity: medium
- Location: Vocal2Midi Web.html:1245
- Issue: The Web UI presents disabled quantization settings that default to "none" / "dev", persists and submits those field values, but the Web task builder ignores them and the effective runtime falls back to `PipelineConfig` defaults of `quantization_step=16` and `quantization_mode="bayes"`. That mismatch is now explicit and tested, so it does not fail this contract unit, but it is a user-visible workflow risk before runtime promotion.
- Evidence: The disabled Web controls are rendered at `Vocal2Midi Web.html:1245` and `Vocal2Midi Web.html:1258`; `collectConfig` and settings persistence still read those values at `Vocal2Midi Web.html:2064`, `Vocal2Midi Web.html:2065`, `Vocal2Midi Web.html:2950`, and `Vocal2Midi Web.html:2951`; defaults persist `quantize_precision="none"` and `quantize_algorithm="dev"` at `web_server.py:101` and `web_server.py:102`; `_build_config` constructs `PipelineConfig` without quantization overrides at `web_task_manager.py:399` through `web_task_manager.py:429`; the effective defaults are defined at `application/config.py:60` and `application/config.py:61`; the ignored-field contract is locked by `tests/test_quantization_caller_defaults_contract.py:345`.
- Required fix: Before `quantization_pipeline_promotion` maps Web calls to Rust or enables these controls, make an explicit product decision: either keep the fields disabled and document/hide their no-op status, map Web `quantize_precision` / `quantize_algorithm` into `PipelineConfig`, or change the visible defaults to match the effective runtime. Update this contract and rerun review before promotion.

- Severity: low
- Location: gui/auto_lyric_view.py:186
- Issue: Desktop GUI quantization controls are visible but disabled, while the current run path hardcodes no quantization (`0` / `simple`). Separately, the parser maps the placeholder algorithm label to `bayes`. This is acceptable as a locked-current-behavior contract, but enabling the controls by only flipping `setEnabled(True)` would still ignore user selections unless the parser outputs are wired into `PipelineConfig`.
- Evidence: The controls are created and disabled at `gui/auto_lyric_view.py:186`, `gui/auto_lyric_view.py:189`, `gui/auto_lyric_view.py:194`, and `gui/auto_lyric_view.py:197`; the active config path hardcodes `quantization_step=0` and `quantization_mode="simple"` at `gui/auto_lyric_view.py:394` and `gui/auto_lyric_view.py:395`; parser mappings live at `gui/fluent_utils.py:1` and `gui/fluent_utils.py:15`; the contract locks both the disabled state and parser behavior at `tests/test_quantization_caller_defaults_contract.py:289` and `tests/test_quantization_caller_defaults_contract.py:305`.
- Required fix: If desktop quantization controls become active, route `parse_quantization` and `parse_quantization_mode` into `PipelineConfig`, update the visible labels to reflect effective algorithms, and refresh this contract before runtime promotion.

Reviewer separation was preserved. This report covers exactly `product_ergonomics_reviewer` for `quantization_caller_defaults_contract`; I reviewed only and did not edit production code, tests, manifest, bootstrap, dependency records, or Rust code.

## Checks

- `PYTHONDONTWRITEBYTECODE=1 uv run pytest tests/test_quantization_caller_defaults_contract.py -q -p no:cacheprovider`: passed, 37 tests.
- `PYTHONDONTWRITEBYTECODE=1 uv run pytest tests/test_web_api.py -q -p no:cacheprovider`: passed, 53 tests.
- `rg -n "quantization_step=|quantization_mode=|quantize_precision|quantize_algorithm|quantize_notes_with_backend|rust_bridge|V2M_QUANT_BACKEND|V2M_QUANT_BRIDGE_BIN" application inference gui web_server.py web_task_manager.py 'Vocal2Midi Web.html' tests/test_quantization_caller_defaults_contract.py rewrite-in-rust/bootstrap/quantization_caller_defaults_contract.md rewrite-in-rust/records/0013-quantization-caller-defaults-contract.md`: inspected; production callers still use legacy quantization defaults, and only the opt-in wrapper references `rust_bridge`.
- `git diff --name-only -- inference/quant/quantization.py inference/pipeline/auto_lyric_hybrid.py application/config.py gui/auto_lyric_view.py gui/fluent_utils.py web_server.py web_task_manager.py application/pipeline.py 'Vocal2Midi Web.html'`: no output; reviewed source refs had no unstaged edits.
- `git diff --check`: passed.

## Residual Risk

This review is source/test based. It did not launch the PyQt GUI or browser UI, did not run a full model pipeline, and did not validate a live user's perception of disabled controls. The undocumented Click entrypoint inside `inference/pipeline/auto_lyric_hybrid.py` still has simplified quantization defaults, but the documented batch CLI is `scripts/slice_asr_cli.py`; this contract should be revisited if that Click path becomes a supported user workflow.

## Promotion Note

This role does not block verifying `quantization_caller_defaults_contract` as a locked-current-behavior unit. It does block any later `quantization_pipeline_promotion` that treats GUI/Web quantization settings as user-effective without first resolving or explicitly preserving the mismatches above. Do not mark the manifest verified from this report alone; coordinator state updates remain separate.
