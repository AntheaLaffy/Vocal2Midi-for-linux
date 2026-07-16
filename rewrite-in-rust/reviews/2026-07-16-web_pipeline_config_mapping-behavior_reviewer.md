# web_pipeline_config_mapping - behavior_reviewer

Date: 2026-07-16
Decision: pass-with-followups

## Findings

- Severity: low
- Location: rewrite-in-rust/fixtures/web_pipeline_config_mapping.jsonl:1
- Issue: The JSONL fixture table verifies Web defaults through a minimal config that still supplies `save_dir`, so the `_build_config` default of `./output` is source-inspected but not fixture-proven as an omitted-field case.
- Evidence: `web_task_manager.py:337` through `web_task_manager.py:341` default `save_dir` to `./output` and create it; `rewrite-in-rust/rust/crates/v2m-core/src/web_config.rs:184` through `rewrite-in-rust/rust/crates/v2m-core/src/web_config.rs:190` mirror that default and directory creation. The normal Web route also merges `web_server.DEFAULT_SETTINGS["pipeline"]["save_dir"]` before task creation at `web_server.py:94` through `web_server.py:100` and `web_server.py:234` through `web_server.py:249`, so this does not affect the current legacy-owned Web path.
- Required fix: Add a future fixture case that omits `save_dir` under an isolated working directory before promoting any Rust-owned caller path that can receive unmerged frontend config directly.

## Checks

- `UV_CACHE_DIR=/tmp/v2m-uv-cache uv run python rewrite-in-rust/bootstrap/check_web_pipeline_config_mapping.py`: passed; emitted the expected invalid-slice fallback warning.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml web_config`: passed, 2 tests.
- `UV_CACHE_DIR=/tmp/v2m-uv-cache uv run pytest tests/test_quantization_caller_defaults_contract.py -k web_quantize_settings_remain_visible_but_ignored_by_pipeline_config`: passed, 1 selected test.
- `git diff --check`: passed.

## Residual Risk

Malformed or adversarial frontend values, including explicit JSON nulls and wrong types, remain outside this unit per `rewrite-in-rust/dependencies/web_pipeline_config_mapping.yaml:51` through `rewrite-in-rust/dependencies/web_pipeline_config_mapping.yaml:52`. Flask request parsing, settings persistence/merge, task lifecycle, SocketIO behavior, and model inference remain legacy-owned or covered by separate units.

## Promotion Note

The behavior gate does not block promotion evidence: Python and Rust match for defaults, slice-bound fallback, output format construction, timestamp generation, device normalization, USTX-gated pitch curves, ignored Web quantization fields, and output-directory creation failure. Keep runtime ownership legacy and do not mark the manifest verified until the coordinator has the remaining required review evidence, including product ergonomics.
