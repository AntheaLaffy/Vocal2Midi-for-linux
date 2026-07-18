# hfa_pyyaml_safe_load_contract - behavior_reviewer rerun

Date: 2026-07-18
Decision: pass

## Findings

No blocking behavior-parity findings in this rerun.

The original behavior blockers are covered by current fixtures and passing Rust/Python checks:

- Duplicate-looking anchors in comments, quoted scalars, and block scalars are fixture-covered as successful loads at `rewrite-in-rust/fixtures/hfa_pyyaml_safe_load_contract.jsonl:47`, `rewrite-in-rust/fixtures/hfa_pyyaml_safe_load_contract.jsonl:48`, and `rewrite-in-rust/fixtures/hfa_pyyaml_safe_load_contract.jsonl:49`.
- Real duplicate anchors remain fixture-covered as a `yaml.composer.ComposerError` at `rewrite-in-rust/fixtures/hfa_pyyaml_safe_load_contract.jsonl:25`.
- The Rust loader now composes first and no longer runs a raw input duplicate-anchor scan before parsing (`rewrite-in-rust/rust/crates/v2m-core/src/hfa_pyyaml.rs:193`); duplicate anchor-name handling is reached only from parser events with anchors (`rewrite-in-rust/rust/crates/v2m-core/src/hfa_pyyaml.rs:292`, `rewrite-in-rust/rust/crates/v2m-core/src/hfa_pyyaml.rs:304`, `rewrite-in-rust/rust/crates/v2m-core/src/hfa_pyyaml.rs:329`, `rewrite-in-rust/rust/crates/v2m-core/src/hfa_pyyaml.rs:364`), matching the PyYAML composer event boundary at `third_party/sources/pyyaml-6.0.3/lib/yaml/composer.py:63`.
- No-space timestamp offsets, short minute/second strings, and invalid hour/minute/second ranges are covered at `rewrite-in-rust/fixtures/hfa_pyyaml_safe_load_contract.jsonl:50`, `rewrite-in-rust/fixtures/hfa_pyyaml_safe_load_contract.jsonl:51`, `rewrite-in-rust/fixtures/hfa_pyyaml_safe_load_contract.jsonl:52`, `rewrite-in-rust/fixtures/hfa_pyyaml_safe_load_contract.jsonl:53`, and `rewrite-in-rust/fixtures/hfa_pyyaml_safe_load_contract.jsonl:54`. The Rust resolver/constructor now requires two-digit minute/second fields, accepts no-space timezone suffixes, and validates hour/minute/second ranges at `rewrite-in-rust/rust/crates/v2m-core/src/hfa_pyyaml.rs:1561`, `rewrite-in-rust/rust/crates/v2m-core/src/hfa_pyyaml.rs:1599`, `rewrite-in-rust/rust/crates/v2m-core/src/hfa_pyyaml.rs:1631`, and `rewrite-in-rust/rust/crates/v2m-core/src/hfa_pyyaml.rs:1533`, aligning with the PyYAML timestamp resolver/constructor references at `third_party/sources/pyyaml-6.0.3/lib/yaml/resolver.py:207` and `third_party/sources/pyyaml-6.0.3/lib/yaml/constructor.py:310`.
- Non-ASCII `!!binary` is covered as a constructor error at `rewrite-in-rust/fixtures/hfa_pyyaml_safe_load_contract.jsonl:55`. The Rust constructor now performs the PyYAML ASCII conversion step before base64 decoding at `rewrite-in-rust/rust/crates/v2m-core/src/hfa_pyyaml.rs:496` and `rewrite-in-rust/rust/crates/v2m-core/src/hfa_pyyaml.rs:1733`, matching `third_party/sources/pyyaml-6.0.3/lib/yaml/constructor.py:294`.
- The fixture table has 56 rows and the Python checker consumes real `config_utils.load_yaml` through `rewrite-in-rust/bootstrap/check_hfa_pyyaml_safe_load_contract.py:217`. The Rust fixture parity test iterates every non-empty JSONL row and calls `load_yaml_path` at `rewrite-in-rust/rust/crates/v2m-core/src/hfa_pyyaml.rs:1977` and `rewrite-in-rust/rust/crates/v2m-core/src/hfa_pyyaml.rs:1987`.
- Production ownership remains unchanged: `config_utils.load_yaml` still calls `yaml.safe_load` at `inference/HubertFA/tools/config_utils.py:7`, while `v2m-core` documents that the Rust workspace is not wired into the Python runtime at `rewrite-in-rust/rust/crates/v2m-core/src/lib.rs:3`. The Rust `hfa_config` module still states that Python owns `load_yaml` and PyYAML behavior at `rewrite-in-rust/rust/crates/v2m-core/src/hfa_config.rs:5`.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_hfa_pyyaml_safe_load_contract.py`: passed, validated 56 fixtures.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_pyyaml`: passed; `hfa_pyyaml::tests::hfa_pyyaml_safe_load_contract_fixture_parity` passed.
- Targeted PyYAML probe through `inference.HubertFA.tools.config_utils.load_yaml`: passed; comments/quoted/block scalar `&dup` text loaded successfully, real duplicate anchors raised `yaml.composer.ComposerError`, `2024-02-29 12:34:56-05:30` loaded as `datetime`, `2024-02-29 1:2:03` loaded as `str`, invalid hour/minute/second values raised `builtins.ValueError`, and `!!binary 你好` raised `yaml.constructor.ConstructorError`.
- `wc -l rewrite-in-rust/fixtures/hfa_pyyaml_safe_load_contract.jsonl`: passed, 56 rows.
- `rg -n "anchor_text_in_comment_ok|anchor_text_in_quoted_scalars_ok|anchor_text_in_block_scalar_ok|duplicate_anchor_error|timestamp_no_space_timezone_offset_datetime|timestamp_short_minute_second_stays_string|timestamp_invalid_hour_value_error|timestamp_invalid_minute_value_error|timestamp_invalid_second_value_error|binary_non_ascii_constructor_error" rewrite-in-rust/fixtures/hfa_pyyaml_safe_load_contract.jsonl`: passed, all prior blocker regression fixtures present.
- `rg -n "hfa_pyyaml|load_yaml\\(|config_utils\\.load_yaml|yaml\\.safe_load|v2m_core|v2m-core|PyYAML" inference application scripts web_server.py web_task_manager.py rewrite-in-rust/rust/crates/v2m-core/src/lib.rs rewrite-in-rust/rust/crates/v2m-core/src/hfa_config.rs`: passed for behavior boundary review; no production Python caller is routed to Rust.
- `rg -n "from .*hfa_pyyaml|import .*hfa_pyyaml|v2m_core|v2m-core|rewrite-in-rust/rust|load_yaml_path\\(|load_yaml_str\\(" --glob '!rewrite-in-rust/**' --glob '!third_party/**' --glob '!settings/**' .`: passed; no non-rewrite production import or caller of the Rust loader was found.

## Residual Risk

This review only reruns the `behavior_reviewer` role for the post-0086 implementation. It does not approve dependency strategy, error/tracing presentation quality, Rust style, architecture, or production owner-switch readiness.

Resource-limit fixtures for large aliases, deeply nested inputs, and scanner/parser limits remain deferred in the control plane before any production-facing owner switch. That deferred production-routing risk does not contradict this behavior rerun because `config_utils.load_yaml` and PyYAML remain the runtime owners.

## Promotion Note

This behavior rerun does not block coordinator state update for `hfa_pyyaml_safe_load_contract`. The behavior role can be treated as passed for the current `reimplemented` unit, subject to the separate required review roles and coordinator-owned manifest update.
