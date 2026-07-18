# asr_chinese_itn_core - data_algorithm_reviewer rerun

Date: 2026-07-18
Decision: fail

## Findings

- Severity: high
- Location: rust/crates/v2m-core/src/asr_chinese_itn.rs:133
- Issue: The remaining idiom-overlap algorithm still uses a fixed UTF-8 byte lookback, `start.saturating_sub(6)`, for Python's two-character regex-group lookback. This can preserve numeric spans after an idiom when Python converts them, especially when ASCII characters separate the idiom from the later Chinese number.
- Evidence: Python computes `l_pos, r_pos` from regex group 2 string indices and checks idiom starts in `range(l_pos - 2, r_pos)` at `../inference/qwen3asr_dml/chinese_itn.py:417` and `../inference/qwen3asr_dml/chinese_itn.py:428`; `pattern.sub(replace, original)` applies that per matched span at `../inference/qwen3asr_dml/chinese_itn.py:508`. Rust passes a byte window into `idiom_overlaps` at `rust/crates/v2m-core/src/asr_chinese_itn.rs:132` and compares byte ranges at `rust/crates/v2m-core/src/asr_chinese_itn.rs:349`. Rust also permits a one-letter ASCII head before a numeric span at `rust/crates/v2m-core/src/asr_chinese_itn.rs:216`, then preserves the whole one-shot candidate when `idiom_overlaps` returns true at `rust/crates/v2m-core/src/asr_chinese_itn.rs:123`. The Python reference command returned `一点一滴abc三 => 一点一滴abc3`, `十有八九abc三 => 十有八九abc3`, `a 一点一滴abc三 => a 一点一滴abc3`, and `一点一滴c三 => 一点一滴c3`; source inspection shows the Rust byte window still overlaps the preceding idiom for the trailing `c三`/`abc三` candidate. Existing fixtures cover standalone idioms at `fixtures/asr_chinese_itn_core.jsonl:20`, `fixtures/asr_chinese_itn_core.jsonl:31`, and `fixtures/asr_chinese_itn_core.jsonl:32`, and ASCII head behavior at `fixtures/asr_chinese_itn_core.jsonl:26`, but not the combined idiom-plus-ASCII-adjacent-number shape.
- Required fix: Replace the fixed byte lookback with Python-equivalent character-index accounting for group 2's start/end, or add Python-generated fixtures that deliberately re-scope this adjacency behavior and adjust the Rust implementation to match those fixtures.

## Resolved Findings Checked

- Malformed decimal no-ops from the initial review are now fixture-backed at `fixtures/asr_chinese_itn_core.jsonl:38`, `fixtures/asr_chinese_itn_core.jsonl:39`, and `fixtures/asr_chinese_itn_core.jsonl:40`; Rust rejects empty decimal segments in `is_pure_num` and `is_value_num` at `rust/crates/v2m-core/src/asr_chinese_itn.rs:607` and `rust/crates/v2m-core/src/asr_chinese_itn.rs:628`.
- The suffix-retry scanner blocker is fixed for the reviewed span classes: Rust now attempts only the full candidate at `rust/crates/v2m-core/src/asr_chinese_itn.rs:117` and preserves it on `None` at `rust/crates/v2m-core/src/asr_chinese_itn.rs:123`.
- Range/unit drift for `万`, `亿`, `千克`, `千米`, and `千米每小时` is now fixture-backed at `fixtures/asr_chinese_itn_core.jsonl:42` through `fixtures/asr_chinese_itn_core.jsonl:48`, with range conversion skipping numeric units but stripping mapped non-numeric suffixes at `rust/crates/v2m-core/src/asr_chinese_itn.rs:853`.

## Checks

- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml asr_chinese_itn_core`: passed, 1 targeted fixture test passed against 48 JSONL cases.
- `uv run python rewrite-in-rust/bootstrap/check_asr_chinese_itn_core.py`: passed, `asr_chinese_itn_core fixtures ok: 48 cases`.
- `cargo clippy --manifest-path rewrite-in-rust/rust/Cargo.toml --all-targets --all-features -- -D warnings`: passed.
- `uv run python - <<'PY' ... PY`: sampled Python idiom/ASCII adjacency behavior; Python converts the trailing number in `一点一滴abc三`, `十有八九abc三`, `a 一点一滴abc三`, and `一点一滴c三`.

## Residual Risk

The unit boundary remains confirmed by `manifest.yaml:1725` through `manifest.yaml:1747`, and writer/reviewer separation is preserved in this read-only rerun. The fixture suite is much stronger than the initial 37-case set, but it still does not cover mixed idiom/ASCII adjacency where byte and character windows diverge.

## Promotion Note

This role blocks promotion. The initial high-severity decimal, one-shot scanner, and range/unit findings are fixed, but the idiom-overlap algorithm still has a data/algorithm parity blocker.
