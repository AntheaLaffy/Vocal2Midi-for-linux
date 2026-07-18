# asr_qwen_language_schema_contract - error_tracing_reviewer

Date: 2026-07-18
Decision: pass

## Findings

No findings.

Evidence checked:

- Python reference raises `ValueError("language is None")`, `ValueError("language is empty")`, and unsupported-language `ValueError` text with the full `SUPPORTED_LANGUAGES` list repr in `../inference/qwen3asr_dml/utils.py:43`.
- Python schema constructors have required positional fields for `StreamingMessage`, `ASREngineConfig`, and `TranscribeResult` in `../inference/qwen3asr_dml/schema.py:16`, `../inference/qwen3asr_dml/schema.py:35`, and `../inference/qwen3asr_dml/schema.py:50`.
- Golden fixtures capture `None`/empty normalization, unsupported-language messages including `None` and empty string, and constructor `TypeError` projections in `fixtures/asr_qwen_language_schema_contract.jsonl:1`, `fixtures/asr_qwen_language_schema_contract.jsonl:41`, and `fixtures/asr_qwen_language_schema_contract.jsonl:55`.
- Rust projects legacy-compatible structured errors as `{ error_type, message }` in `rust/crates/v2m-core/src/asr_qwen_language_schema.rs:42`, maps normalization failures in `rust/crates/v2m-core/src/asr_qwen_language_schema.rs:64`, formats the unsupported-language list repr in `rust/crates/v2m-core/src/asr_qwen_language_schema.rs:92`, and exposes explicit constructor-missing `TypeError` messages in `rust/crates/v2m-core/src/asr_qwen_language_schema.rs:336`.
- The Rust fixture test serializes errors through the same structured projection in `rust/crates/v2m-core/src/asr_qwen_language_schema.rs:516`.
- Runtime ownership and rollback remain clear: the unit is reimplemented but still legacy-owned in `manifest.yaml:1757`, and record 0108 states the Rust module is not wired into production Python callers in `records/0108-implement-asr-qwen-language-schema-contract.md:19` with rollback to Python owners in `records/0108-implement-asr-qwen-language-schema-contract.md:54`.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_asr_qwen_language_schema_contract.py`: passed; `asr_qwen_language_schema_contract fixtures ok: 57 cases`.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml asr_qwen_language_schema_contract`: passed; 1 targeted Rust fixture-parity test passed, 0 failed.

## Residual Risk

This review covered the fixture-backed error and DTO contract only. The Rust helpers are not in the production path, so this pass does not prove future bridge-level redaction, logging policy, or caller context once runtime routing is introduced. The legacy unsupported-language error echoes the supplied language value; Rust mirrors that behavior and does not add logging, so no new sensitive-data exposure was found in this unit.

## Promotion Note

This error/tracing role does not block promotion. Coordinator state should still wait for the required behavior review before marking the unit verified.
