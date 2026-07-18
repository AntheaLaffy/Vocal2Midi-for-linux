# hfa_config_validation_core - behavior_reviewer

Date: 2026-07-18
Decision: pass-with-followups

## Findings

- Severity: low
- Location: rewrite-in-rust/fixtures/hfa_config_validation_core.jsonl:9
- Issue: The fixture matrix does not explicitly lock every JSON-compatible numeric loader shape claimed by the seam.
- Evidence: The manifest requires injected JSON-compatible value/error handling for `check_configs` at `rewrite-in-rust/manifest.yaml:1498`, and the bootstrap defines the loaded value domain as JSON-compatible null, bool, number, string, list, and string-key mapping at `rewrite-in-rust/bootstrap/hfa_config_validation_core.md:31`. Rust exposes both `Int` and `Float` value kinds and projects their Python type names at `rewrite-in-rust/rust/crates/v2m-core/src/hfa_config.rs:18`, but the 34-case fixture covers top-level null/list/string/bool at `rewrite-in-rust/fixtures/hfa_config_validation_core.jsonl:9` through line 12 and dictionary int/bool/list values at line 28 through line 30 without top-level int/float or float dictionary cases. Source inspection shows the Rust numeric branches should project the same `'int' object has no attribute 'get'`, `'float' object has no attribute 'get'`, and unsupported `/` operand messages as Python, so this is a coverage gap rather than an observed parity failure.
- Required fix: Add targeted follow-up fixture cases for top-level int, top-level float, and float dictionary values before treating this seam as fully locked for arbitrary JSON-compatible injected loader values or cross-unit loader composition.

## Checks

- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_config_validation_core`: passed; 1 hfa_config fixture-parity test passed, 113 tests filtered.
- `uv run python rewrite-in-rust/bootstrap/check_hfa_config_validation_core.py`: passed; validated 34 fixtures against legacy Python 3.12 `check_configs`.
- `rg -n "check_configs_with_loader|v2m_core::hfa_config|hfa_config::|pub mod hfa_config|from .*hfa_config|import .*hfa_config" inference application web_server.py web_task_manager.py rewrite-in-rust/rust/crates/v2m-core/src`: only found the Rust library module, its test, and `pub mod hfa_config`; no production Python route or bridge was found.
- `git diff -- rewrite-in-rust/rust/crates/v2m-core/src/hfa_config.rs rewrite-in-rust/bootstrap/check_hfa_config_validation_core.py rewrite-in-rust/fixtures/hfa_config_validation_core.jsonl rewrite-in-rust/dependencies/hfa_config_validation_core.yaml rewrite-in-rust/bootstrap/hfa_config_validation_core.md inference/HubertFA/tools/config_utils.py inference/HubertFA/onnx_infer.py`: no scoped diff output.

## Behavior Evidence

Python `check_configs` checks `vocab.<suffix>` existence, then `config.<suffix>` existence, then calls `load_yaml(vocab_file)`, then evaluates `vocab.get("dictionaries", [])`, `dictionaries.items()`, skips only `None`, joins dictionary paths under `model_dir`, and raises exact assertion messages with `dictionary_path.absolute()` for missing dictionaries (`inference/HubertFA/tools/config_utils.py:13`).

Rust `check_configs_with_loader` preserves that order: vocab existence at `rewrite-in-rust/rust/crates/v2m-core/src/hfa_config.rs:158`, config existence at line 166, one injected loader call at line 174, `dictionaries` default/list behavior at line 175, ordered item iteration at line 180, `Null` skipping at line 182, dictionary path joining/existence at line 184, absolute assertion projection at line 186, and Python-compatible dynamic type failures at line 192.

The fixture/checker pair covers default/json/empty/None suffix rendering, missing-file order, loader call timing, loader error pass-through, top-level dynamic failures, missing/null/list/string/int `dictionaries` failures, `None` skipping, relative/absolute/Unicode dictionary paths, first dictionary failure, invalid config contents not being parsed, and repeated calls (`rewrite-in-rust/bootstrap/check_hfa_config_validation_core.py:73`, `rewrite-in-rust/fixtures/hfa_config_validation_core.jsonl:1`).

Runtime ownership remains legacy Python. `InferenceOnnx.load_config` still calls Python `check_configs(self.model_folder, suffix='json')` at `inference/HubertFA/onnx_infer.py:19`, and `load_hfa_model` continues through `InferenceOnnx.load_config` at `inference/API/hfa_api.py:99`.

## Residual Risk

This behavior review does not approve full PyYAML `safe_load` parity, non-JSON PyYAML values, loader parsing errors, `VERSION`/config assignment, ONNX session behavior, or production routing. Those remain in the planned `hfa_pyyaml_safe_load_contract` and later promotion work per `rewrite-in-rust/records/0079-recut-hfa-config-file-contract.md:56`.

## Promotion Note

This role does not block promotion on behavior parity for the current injected-loader validator, but it leaves a low-priority fixture follow-up for numeric JSON-compatible loader shapes. The coordinator can use this as behavior review evidence for state update only alongside the separate required dependency/bootstrap and error/tracing reviews; it is not a runtime-owner promotion approval.
