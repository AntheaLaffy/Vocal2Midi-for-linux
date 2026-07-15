# web_pipeline_config_mapping - dependency_bootstrap_reviewer

Date: 2026-07-15
Decision: pass

## Findings

No findings.

Boundary decision: confirmed. The manifest unit should not be split, merged,
deferred, or replaced for this dependency/bootstrap gate.

Evidence:

- `rewrite-in-rust/manifest.yaml:411` defines the unit as
  `web_pipeline_config_mapping`, keeps `web_task_manager.py`,
  `application/config.py`, and `inference/device_utils.py` as the source refs,
  and keeps `TaskManager._build_config` as the rollback owner.
- `rewrite-in-rust/dependencies/web_pipeline_config_mapping.yaml:3` covers the
  needed capabilities: frontend defaults, device normalization, slice-bound
  fallback, output-format construction, and timestamp mapping.
- `rewrite-in-rust/dependencies/web_pipeline_config_mapping.yaml:34` confirms
  inventory impact for `_build_config` only and explicitly keeps settings merge,
  task lifecycle, SocketIO, downloads, filesystem browsing, GUI behavior, and
  pipeline execution out of scope.
- `rewrite-in-rust/bootstrap/web_pipeline_config_mapping.md:35` narrows the
  dependency expansion to stdlib path handling plus `PipelineConfig`,
  `validate_slice_bounds`, slice defaults, and `normalize_runtime_device`; it
  does not require Flask, Flask-SocketIO, threading, WebSocket, ONNX Runtime,
  Qwen ASR, PyQt, or network crates.
- `rewrite-in-rust/records/0018-confirm-web-pipeline-config-mapping-boundary.md:21`
  records the seam as a narrow Web-to-application mapping unit with no bridge.
- `web_task_manager.py:318` shows `_build_config` as the selected legacy seam;
  `web_task_manager.py:331` through `web_task_manager.py:392` perform the
  deterministic config mapping, slice fallback, output format construction, and
  timestamp construction; `web_task_manager.py:399` returns `PipelineConfig`.
- `web_server.py:234` shows persisted settings merge happens before
  `TaskManager` receives the dict, matching the kept-legacy decision.
- `application/config.py:17` and `application/config.py:33` provide the local
  validation/defaults and `PipelineConfig` shape reused by the unit.
- `inference/device_utils.py:76` is the local device-normalization dependency,
  not the ONNX provider-selection path.
- `rewrite-in-rust/rust/crates/v2m-core/src/web_config.rs:176` implements the
  independent Rust mapping, reusing the existing Rust device and slice-bounds
  modules; `rewrite-in-rust/rust/crates/v2m-core/src/web_config.rs:274` keeps
  Web quantization fields ignored by preserving `16` and `"bayes"`.
- `rewrite-in-rust/rust/crates/v2m-core/Cargo.toml:12` adds only `serde_json`
  as a dev-dependency for fixture tests, and `cargo tree --edges normal` shows
  no normal dependency for `v2m-core`.
- `rewrite-in-rust/fixtures/web_pipeline_config_mapping.jsonl:1` covers minimal
  defaults, full typed Web config, invalid slice fallback, USTX-gated pitch
  curve, zero/one timestamp counts, ignored Web quantization fields, and output
  directory creation failure.
- `tests/test_quantization_caller_defaults_contract.py:345` independently pins
  the visible-but-ignored Web quantization settings contract.

## Checks

- `git status --short`: reviewed dirty tree and untracked unit files; report
  write scope stayed limited to this review file.
- `git diff --stat` and focused tracked/untracked diffs: reviewed manifest,
  resource, Cargo, dependency, bootstrap, fixture, record, and Rust module
  changes relevant to this unit.
- `uv run python rewrite-in-rust/bootstrap/check_web_pipeline_config_mapping.py`:
  passed; emitted the expected invalid-slice fallback warning.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml web_config`:
  passed, 2 tests.
- `uv run pytest tests/test_web_api.py tests/test_quantization_caller_defaults_contract.py`:
  passed, 90 tests with 3 existing deprecation warnings from audioread stdlib
  imports.
- `cargo tree --manifest-path rewrite-in-rust/rust/Cargo.toml -p v2m-core --edges normal`:
  passed; no normal crate dependencies listed for `v2m-core`.
- `cargo tree --manifest-path rewrite-in-rust/rust/Cargo.toml -p v2m-core --edges dev`:
  passed; `serde_json` appears only under dev-dependencies.
- `cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check`:
  passed.

## Residual Risk

Malformed or intentionally adversarial frontend values, including nulls and
wrong JSON types, remain legacy-owned by this dependency record rather than
fixture-proven. Flask request parsing, settings persistence/merge, task
lifecycle, SocketIO behavior, downloads, and model inference are intentionally
outside this unit and need their own reviews before any runtime bridge or
promotion.

## Promotion Note

This dependency/bootstrap role does not block the unit. It is ready for the
coordinator to use as dependency-bootstrap evidence, but it is not sufficient by
itself to mark the unit verified; the manifest still requires the separate
behavior and product-ergonomics reviews.
