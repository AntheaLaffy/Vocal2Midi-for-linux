# asr_qwen_language_schema_contract - behavior_reviewer

Date: 2026-07-18
Decision: pass

## Findings

No behavior-parity findings.

Evidence reviewed:

- Language list/order: Python `inference/qwen3asr_dml/utils.py:9`, Rust `rewrite-in-rust/rust/crates/v2m-core/src/asr_qwen_language_schema.rs:9`, and fixture `rewrite-in-rust/fixtures/asr_qwen_language_schema_contract.jsonl:10`.
- Language normalization and validation behavior/error text: Python `inference/qwen3asr_dml/utils.py:43`, Rust `rewrite-in-rust/rust/crates/v2m-core/src/asr_qwen_language_schema.rs:64`, and fixtures `rewrite-in-rust/fixtures/asr_qwen_language_schema_contract.jsonl:1`.
- `MsgType` auto order/name/value/`str`/`repr`: Python `inference/qwen3asr_dml/schema.py:7`, Rust `rewrite-in-rust/rust/crates/v2m-core/src/asr_qwen_language_schema.rs:129`, and fixture `rewrite-in-rust/fixtures/asr_qwen_language_schema_contract.jsonl:45`.
- Dataclass defaults/custom snapshots/reprs/default_factory behavior and constructor TypeError messages: Python `inference/qwen3asr_dml/schema.py:16`, Rust `rewrite-in-rust/rust/crates/v2m-core/src/asr_qwen_language_schema.rs:182`, and fixtures `rewrite-in-rust/fixtures/asr_qwen_language_schema_contract.jsonl:46`.
- Fixture checker compares the golden JSONL against current Python behavior at `rewrite-in-rust/bootstrap/check_asr_qwen_language_schema_contract.py:47` and Rust compares the same fixtures at `rewrite-in-rust/rust/crates/v2m-core/src/asr_qwen_language_schema.rs:401`.
- Boundary and rollback are recorded: the parent ASR text-postprocess unit is split at `rewrite-in-rust/manifest.yaml:1701`, this child is `reimplemented` at `rewrite-in-rust/manifest.yaml:1757`, and rollback keeps Python utils/schema as runtime owners at `rewrite-in-rust/manifest.yaml:1778` and `rewrite-in-rust/records/0108-implement-asr-qwen-language-schema-contract.md:54`.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_asr_qwen_language_schema_contract.py`: passed, `asr_qwen_language_schema_contract fixtures ok: 57 cases`.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml asr_qwen_language_schema_contract`: passed, 1 matching Rust fixture-parity test passed.
- `uv run python -m py_compile inference/qwen3asr_dml/utils.py inference/qwen3asr_dml/schema.py`: passed.
- `uv run python scripts/audit_vendored_sources.py`: passed, 0 third_party binary artifacts.
- `rg -n "asr_qwen_language_schema|qwen_language_schema_contract" .. -g '*.py' -g '*.rs' -g '*.yaml' -g '*.md' -g '*.jsonl'`: found only rewrite control-plane artifacts, the Rust library module export, fixtures, and checks; no production Python caller wiring.

## Residual Risk

The fixtures intentionally cover the declared contract surface, not arbitrary Python object `str`/`repr` behavior for every possible value that could be placed in `StreamingMessage.data` or `TranscribeResult.performance`. This is acceptable for this behavior gate because the manifest and implementation record scope the unit to language validation plus schema DTO defaults/snapshots, while production runtime ownership remains Python.

## Promotion Note

This behavior review does not block promotion. The unit is ready for the coordinator to consider state update after any separately required review roles pass; this report does not mark the manifest verified.
