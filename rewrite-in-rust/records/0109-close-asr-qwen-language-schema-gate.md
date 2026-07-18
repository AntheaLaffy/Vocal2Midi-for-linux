# 0109 - Close ASR Qwen Language Schema Gate

Date: 2026-07-18

## Unit

`asr_qwen_language_schema_contract`

## Decision

Mark `asr_qwen_language_schema_contract` as `verified`.

Python remains the runtime owner. This gate verifies the independent Rust
library seam and fixture parity only.

## Review Evidence

Required reviews passed:

- `reviews/2026-07-18-asr_qwen_language_schema_contract-behavior_reviewer.md`
- `reviews/2026-07-18-asr_qwen_language_schema_contract-error_tracing_reviewer.md`

Both reviews reported no findings.

## Fixture Evidence

`fixtures/asr_qwen_language_schema_contract.jsonl` contains 57 Python 3.12
golden cases covering language normalization, supported-language validation,
error messages, `MsgType` enum order/projections, dataclass default/custom
snapshots, reprs, `DecodeResult` default_factory independence, and required
constructor errors.

## Verification

Final commands:

```bash
uv run python rewrite-in-rust/bootstrap/check_asr_qwen_language_schema_contract.py
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml asr_qwen_language_schema_contract
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml
cargo clippy --manifest-path rewrite-in-rust/rust/Cargo.toml --all-targets --all-features -- -D warnings
uv run python -m py_compile inference/qwen3asr_dml/utils.py inference/qwen3asr_dml/schema.py
uv run python scripts/audit_vendored_sources.py
```

## Rollback

Keep `inference.qwen3asr_dml.utils` and `inference.qwen3asr_dml.schema` as
runtime owners.
