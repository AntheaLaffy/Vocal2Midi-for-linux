# asr_chinese_itn_core - data_algorithm_reviewer rerun2

Date: 2026-07-18
Decision: fail

## Findings

- Severity: high
- Location: rust/crates/v2m-core/src/asr_chinese_itn.rs:106
- Issue: Leading malformed decimal spans are still split and partially converted, while Python consumes the full rejected regex span and leaves it unchanged. Rust excludes `点` from candidate starts, emits it as plain text, then starts a new candidate at the following digit and converts that digit through the pure-number path. That makes inputs such as `点三`, `我有点三`, and `点三个人` become `点3`, `我有点3`, and `点3个人` by source inspection, but Python leaves the whole matched span unchanged.
- Evidence: Python's master pattern includes `点` inside group 2 at `../inference/qwen3asr_dml/chinese_itn.py:204`, `../inference/qwen3asr_dml/chinese_itn.py:208`, and `../inference/qwen3asr_dml/chinese_itn.py:220`, so `点三` is one replacement span. Python's `pure_num` and `value_num` grammars require digits before/after decimal markers at `../inference/qwen3asr_dml/chinese_itn.py:229` and `../inference/qwen3asr_dml/chinese_itn.py:232`, and unsupported replacements fall through to the original span at `../inference/qwen3asr_dml/chinese_itn.py:482`. The reference probe `uv run python - <<'PY' ... PY` returned `点三 => 点三`, `我有点三 => 我有点三`, `点三个人 => 点三个人`, `百分之点三 => 百分之点三`, `三比点四 => 三比点四`, and `三分之点四 => 三分之点四`. Rust's scanner emits non-starting characters at `rust/crates/v2m-core/src/asr_chinese_itn.rs:104` through `rust/crates/v2m-core/src/asr_chinese_itn.rs:109`, `is_numeric_start` omits `点` at `rust/crates/v2m-core/src/asr_chinese_itn.rs:365`, and the later single digit satisfies `is_pure_num` at `rust/crates/v2m-core/src/asr_chinese_itn.rs:650` before `convert_pure_num` maps it at `rust/crates/v2m-core/src/asr_chinese_itn.rs:501`. Fixtures cover trailing-dot malformed decimals at `fixtures/asr_chinese_itn_core.jsonl:38` through `fixtures/asr_chinese_itn_core.jsonl:40` and malformed time at `fixtures/asr_chinese_itn_core.jsonl:41`, but not leading-dot malformed spans.
- Required fix: Treat leading `点` plus following Chinese numeric text as one Python-equivalent rejected candidate, or otherwise prevent conversion of the suffix inside that rejected regex span. Add Python-generated fixtures for `点三`, sentence-embedded `点三`, and leading-dot forms with unit/operator context.

## Resolved Findings Checked

- The suffix-retry scanner blocker for startable rejected spans remains fixed: Rust now attempts the full candidate once at `rust/crates/v2m-core/src/asr_chinese_itn.rs:124` through `rust/crates/v2m-core/src/asr_chinese_itn.rs:131` and preserves it on `None`.
- The trailing malformed-decimal cases are fixture-backed at `fixtures/asr_chinese_itn_core.jsonl:38` through `fixtures/asr_chinese_itn_core.jsonl:40`, with empty decimal segments rejected in `is_pure_num` and `is_value_num` at `rust/crates/v2m-core/src/asr_chinese_itn.rs:650` through `rust/crates/v2m-core/src/asr_chinese_itn.rs:678`.
- The first rerun's idiom/ASCII lookback blocker is fixed for the reviewed shapes: `lookback_start_byte` uses character positions at `rust/crates/v2m-core/src/asr_chinese_itn.rs:225`, immediate idiom-adjacent no-op consumption is implemented at `rust/crates/v2m-core/src/asr_chinese_itn.rs:91` through `rust/crates/v2m-core/src/asr_chinese_itn.rs:100` and `rust/crates/v2m-core/src/asr_chinese_itn.rs:238` through `rust/crates/v2m-core/src/asr_chinese_itn.rs:256`, and fixtures cover both no-op and ASCII-separated conversion cases at `fixtures/asr_chinese_itn_core.jsonl:56` through `fixtures/asr_chinese_itn_core.jsonl:60`.
- The large-place two-digit range branch is fixture-backed at `fixtures/asr_chinese_itn_core.jsonl:49` through `fixtures/asr_chinese_itn_core.jsonl:55` and implemented by `is_large_place_two_digit_range` at `rust/crates/v2m-core/src/asr_chinese_itn.rs:897` through `rust/crates/v2m-core/src/asr_chinese_itn.rs:930`, with conversion through `convert_range_pattern_2` at `rust/crates/v2m-core/src/asr_chinese_itn.rs:1012` through `rust/crates/v2m-core/src/asr_chinese_itn.rs:1087`.

## Checks

- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml asr_chinese_itn_core`: passed, 1 targeted fixture test passed against 60 JSONL cases.
- `uv run python rewrite-in-rust/bootstrap/check_asr_chinese_itn_core.py`: passed, `asr_chinese_itn_core fixtures ok: 60 cases`.
- `cargo clippy --manifest-path rewrite-in-rust/rust/Cargo.toml --all-targets --all-features -- -D warnings`: passed.
- `uv run python - <<'PY' ... PY`: confirmed Python preserves leading-dot malformed spans and related operator/unit contexts unchanged.

## Residual Risk

This rerun stayed within the `data_algorithm_reviewer` role for `asr_chinese_itn_core`. The unit boundary remains confirmed by `manifest.yaml:1725` through `manifest.yaml:1749`, and writer/reviewer separation is preserved by this read-only review. The remaining risk is limited to scanner approximation around Python regex matches that begin with characters Rust does not allow as candidate starts.

## Promotion Note

This role blocks promotion. The prior idiom-lookback and range-branch blockers are fixed, but malformed decimal handling is not fully fixed because leading-dot rejected spans can still be partially converted.
