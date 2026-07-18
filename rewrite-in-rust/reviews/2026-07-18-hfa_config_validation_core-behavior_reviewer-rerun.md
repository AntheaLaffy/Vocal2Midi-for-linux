# hfa_config_validation_core - behavior_reviewer

Date: 2026-07-18
Decision: pass

## Findings

No behavior findings.

The prior low fixture-coverage finding is closed. The fixture table now includes top-level integer and float loader values at `rewrite-in-rust/fixtures/hfa_config_validation_core.jsonl:13` and line 14, plus dictionary-entry float and nested-mapping non-path values at line 33 and line 34. The dependency and bootstrap records now describe 38 cases and explicitly call out numeric and mapping coverage at `rewrite-in-rust/dependencies/hfa_config_validation_core.yaml:30` and `rewrite-in-rust/bootstrap/hfa_config_validation_core.md:44`.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_hfa_config_validation_core.py`: passed; validated 38 fixtures against legacy Python 3.12 `check_configs`.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_config_validation_core`: passed; the Rust fixture-parity test passed against the same 38-case JSONL table.
- `uv run python -m py_compile inference/HubertFA/tools/config_utils.py rewrite-in-rust/bootstrap/check_hfa_config_validation_core.py`: passed.
- `rg -n "check_configs_with_loader|v2m_core::hfa_config|hfa_config::|pub mod hfa_config|from .*hfa_config|import .*hfa_config" inference application web_server.py web_task_manager.py rewrite-in-rust/rust/crates/v2m-core/src`: only found the independent Rust module export and its internal test call sites; no production Python bridge or route was found.
- `git diff --check -- rewrite-in-rust/fixtures/hfa_config_validation_core.jsonl rewrite-in-rust/dependencies/hfa_config_validation_core.yaml rewrite-in-rust/bootstrap/hfa_config_validation_core.md rewrite-in-rust/manifest.yaml`: passed with no whitespace errors.

## Behavior Evidence

Python `check_configs` still validates `vocab.<suffix>` before `config.<suffix>`, calls `load_yaml` only after those existence checks, evaluates `vocab.get("dictionaries", [])`, iterates `dictionaries.items()`, skips only `None`, joins non-`None` dictionary values with the model directory, and raises exact assertion messages from `dictionary_path.absolute()` (`inference/HubertFA/tools/config_utils.py:13`).

Rust `check_configs_with_loader` preserves that public seam over an injected loader result: vocab and config existence at `rewrite-in-rust/rust/crates/v2m-core/src/hfa_config.rs:158` and line 166, loader call at line 174, `dictionaries` lookup/default at line 175, ordered item iteration at line 180, `Null` skipping at line 182, string path validation at line 183, absolute assertion projection at line 186, and dynamic `TypeError` projection for non-path JSON-compatible values at line 192.

The added rows exercise the exact numeric and mapping branches that were previously only source-inspected. The Python checker proves the expected exception type/message and loader call path, and the Rust test proves the Rust implementation emits the same normalized results.

## Residual Risk

This review remains scoped to the injected-loader `check_configs` validator. It does not approve PyYAML `safe_load` parity, non-JSON PyYAML values, loader parser errors, `VERSION`/config assignment, ONNX session behavior, or production routing. Those remain outside `hfa_config_validation_core` per `rewrite-in-rust/records/0079-recut-hfa-config-file-contract.md:56`.

## Promotion Note

Behavior parity is sufficient for this role. The coordinator can keep `hfa_config_validation_core` verified from the behavior-review perspective, assuming the separate dependency/bootstrap and error/tracing rerun evidence remains acceptable. This report is rerun evidence only; it does not edit or revalidate the manifest state.
