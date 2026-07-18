# asr_chinese_itn_core - data_algorithm_reviewer rerun3

Date: 2026-07-18
Decision: pass-with-followups

## Findings

- Severity: low
- Location: fixtures/asr_chinese_itn_core.jsonl:62
- Issue: The leading-dot fix is now source-covered and fixture-backed for bare, sentence, unit, ASCII-suffix, and percent contexts, but the exact ratio/fraction samples named in the rerun2 evidence (`三比点四`, `三分之点四`) are not fixture rows. This is not a blocker because the current scanner starts one-shot candidates at `点`, and ratio/fraction candidates that contain a leading-dot side reject through `is_value_num` and preserve the full candidate. Adding the two rows would make the previously sampled operator context explicit in the fixture suite.
- Evidence: `fixtures/asr_chinese_itn_core.jsonl:62` through `fixtures/asr_chinese_itn_core.jsonl:66` cover the 0106 leading-dot additions. Python defines `点` as part of the master numeric span at `../inference/qwen3asr_dml/chinese_itn.py:203` through `../inference/qwen3asr_dml/chinese_itn.py:225`, rejects empty decimal sides in `pure_num`/`value_num` at `../inference/qwen3asr_dml/chinese_itn.py:228` through `../inference/qwen3asr_dml/chinese_itn.py:232`, and falls back to the original span at `../inference/qwen3asr_dml/chinese_itn.py:482` through `../inference/qwen3asr_dml/chinese_itn.py:496`. Rust now includes `点` in `is_numeric_start` at `rust/crates/v2m-core/src/asr_chinese_itn.rs:365` through `rust/crates/v2m-core/src/asr_chinese_itn.rs:383`, preserves a failed full candidate at `rust/crates/v2m-core/src/asr_chinese_itn.rs:124` through `rust/crates/v2m-core/src/asr_chinese_itn.rs:133`, rejects empty decimal sides at `rust/crates/v2m-core/src/asr_chinese_itn.rs:651` through `rust/crates/v2m-core/src/asr_chinese_itn.rs:679`, and returns `None` for malformed fraction/ratio sides at `rust/crates/v2m-core/src/asr_chinese_itn.rs:163` through `rust/crates/v2m-core/src/asr_chinese_itn.rs:174`. A Python probe from the repo root returned `三比点四 => 三比点四` and `三分之点四 => 三分之点四`.
- Required fix: Non-blocking follow-up: add Python-generated fixture rows for `三比点四` and `三分之点四` if this unit receives another fixture-hardening pass.

## Resolved Blockers Checked

- Malformed decimal and leading-dot rejected spans are fixed. Fixtures now cover trailing-dot no-ops at `fixtures/asr_chinese_itn_core.jsonl:38` through `fixtures/asr_chinese_itn_core.jsonl:40` and leading-dot no-ops at `fixtures/asr_chinese_itn_core.jsonl:62` through `fixtures/asr_chinese_itn_core.jsonl:66`. Rust rejects empty decimal segments in both pure-number and value-number predicates at `rust/crates/v2m-core/src/asr_chinese_itn.rs:651` through `rust/crates/v2m-core/src/asr_chinese_itn.rs:679`.
- The suffix-retry scanner blocker is fixed. Rust now attempts the full candidate once and preserves it on conversion failure at `rust/crates/v2m-core/src/asr_chinese_itn.rs:124` through `rust/crates/v2m-core/src/asr_chinese_itn.rs:133`, matching Python's single `pattern.sub(replace, original)` replacement flow at `../inference/qwen3asr_dml/chinese_itn.py:508` through `../inference/qwen3asr_dml/chinese_itn.py:511`.
- The byte-based idiom lookback blocker is fixed for the reviewed algorithm shape. Rust computes the two-character lookback by character positions at `rust/crates/v2m-core/src/asr_chinese_itn.rs:225` through `rust/crates/v2m-core/src/asr_chinese_itn.rs:236`, consumes immediate idiom-adjacent numeric spans as no-op at `rust/crates/v2m-core/src/asr_chinese_itn.rs:91` through `rust/crates/v2m-core/src/asr_chinese_itn.rs:100` and `rust/crates/v2m-core/src/asr_chinese_itn.rs:238` through `rust/crates/v2m-core/src/asr_chinese_itn.rs:256`, and fixtures cover no-op and ASCII-separated cases at `fixtures/asr_chinese_itn_core.jsonl:57` through `fixtures/asr_chinese_itn_core.jsonl:61`.
- The range/unit and large-place two-digit branch blockers are fixed. Fixtures cover `万`/`亿`/`千克`/`千米`/`千米每小时` edges at `fixtures/asr_chinese_itn_core.jsonl:42` through `fixtures/asr_chinese_itn_core.jsonl:48` and the `[万千百]+two-digit` branch at `fixtures/asr_chinese_itn_core.jsonl:49` through `fixtures/asr_chinese_itn_core.jsonl:56`. Rust implements the range predicates at `rust/crates/v2m-core/src/asr_chinese_itn.rs:849` through `rust/crates/v2m-core/src/asr_chinese_itn.rs:930` and conversion at `rust/crates/v2m-core/src/asr_chinese_itn.rs:945` through `rust/crates/v2m-core/src/asr_chinese_itn.rs:1087`.

## Checks

- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml asr_chinese_itn_core`: passed; 1 targeted fixture test passed, 120 filtered.
- `uv run python rewrite-in-rust/bootstrap/check_asr_chinese_itn_core.py`: passed; `asr_chinese_itn_core fixtures ok: 66 cases`.
- `cargo clippy --manifest-path rewrite-in-rust/rust/Cargo.toml --all-targets --all-features -- -D warnings`: passed.
- `uv run python - <<'PY' ... PY`: sampled previous blocker inputs; Python preserves malformed decimal/time/operator spans and converts the fixed range/idiom cases as represented in the current fixtures.

## Residual Risk

This rerun stayed within the `data_algorithm_reviewer` role for `asr_chinese_itn_core`. The unit boundary remains confirmed by `manifest.yaml:1725` through `manifest.yaml:1751`, with split/rollback evidence in `dependencies/asr_text_postprocess_contract.yaml:116` through `dependencies/asr_text_postprocess_contract.yaml:118`, `bootstrap/asr_text_postprocess_contract.md:29` through `bootstrap/asr_text_postprocess_contract.md:33`, and `records/0102-bootstrap-asr-text-postprocess-contract.md:20` through `records/0102-bootstrap-asr-text-postprocess-contract.md:34`. The Rust fixture test includes the 66-case JSONL file at `rust/crates/v2m-core/src/asr_chinese_itn.rs:1125` through `rust/crates/v2m-core/src/asr_chinese_itn.rs:1140`.

The remaining risk is fixture completeness around rare Python regex span boundaries, not a known algorithm blocker. The exact ratio/fraction leading-dot examples from rerun2 should be added if the team wants every sampled edge recorded as a golden case.

## Promotion Note

This role does not block promotion. Previous high-severity data/algorithm blockers are fixed; the remaining low-severity item is fixture hardening only.
