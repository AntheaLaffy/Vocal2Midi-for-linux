# asr_chinese_itn_core - behavior_reviewer

Date: 2026-07-18
Decision: pass-with-followups

## Findings

- Severity: low
- Location: fixtures/asr_chinese_itn_core.jsonl:49
- Issue: The previous behavior blockers are fixed, but the expanded golden set still does not include the bare `百` form `三百二三` that was named in the first rerun finding.
- Evidence: The 0105 fixture cluster covers the missing `[万千百]+two-digit` branch for `万` and `千` at `fixtures/asr_chinese_itn_core.jsonl:49` through `fixtures/asr_chinese_itn_core.jsonl:55`, and covers `三百二三十人` at `fixtures/asr_chinese_itn_core.jsonl:12`, but `rg -n "三百二三" fixtures/asr_chinese_itn_core.jsonl` only finds the `...十人` form. Python's range detector includes `[一二三四五六七八九十]+[万千百][一二三四五六七八九]{2}` at `../inference/qwen3asr_dml/chinese_itn.py:155`; a Python probe returned `三百二三 => 320~330`. Rust source inspection shows the bare `百` form is now accepted by `is_large_place_two_digit_range` at `rust/crates/v2m-core/src/asr_chinese_itn.rs:897` through `rust/crates/v2m-core/src/asr_chinese_itn.rs:930`, and converted by `convert_range_pattern_2` at `rust/crates/v2m-core/src/asr_chinese_itn.rs:1012` through `rust/crates/v2m-core/src/asr_chinese_itn.rs:1084`.
- Required fix: Add a Python-generated golden fixture for `三百二三 -> 320~330` in the next fixture cleanup. This is a coverage follow-up, not a behavior blocker for this rerun.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_asr_chinese_itn_core.py`: passed, `asr_chinese_itn_core fixtures ok: 60 cases`.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml asr_chinese_itn_core`: passed, 1 `asr_chinese_itn_core_fixture_parity` test, 120 filtered.
- `uv run python - <<'PY' ... chinese_to_num(...) ... PY`: confirmed current Python outputs for the previous blockers: `三万二三 => 32000~33000`, `三万二三人 => 32000~33000人`, `一万二三 => 12000~13000`, `二十万三四 => 23000~24000`, `十三万二三 => 102000~103000`, `三千二三 => 3200~3300`, `三百二三 => 320~330`, `三万二三千米 => 32000~33000千米`, malformed decimals/time remain unchanged, and idiom-adjacent cases match the new fixture expectations.
- Source review: malformed decimal no-ops are fixture-gated at `fixtures/asr_chinese_itn_core.jsonl:38` through `fixtures/asr_chinese_itn_core.jsonl:41`; Rust rejects empty decimal segments at `rust/crates/v2m-core/src/asr_chinese_itn.rs:650` through `rust/crates/v2m-core/src/asr_chinese_itn.rs:668` and `rust/crates/v2m-core/src/asr_chinese_itn.rs:676` through `rust/crates/v2m-core/src/asr_chinese_itn.rs:688`.
- Source review: range unit blockers for `万`/`亿`/`千米`/`千克` are fixture-gated at `fixtures/asr_chinese_itn_core.jsonl:42` through `fixtures/asr_chinese_itn_core.jsonl:48`; Rust preserves Python's range-first treatment by skipping numeric-place suffix stripping at `rust/crates/v2m-core/src/asr_chinese_itn.rs:932` through `rust/crates/v2m-core/src/asr_chinese_itn.rs:942` and `rust/crates/v2m-core/src/asr_chinese_itn.rs:944` through `rust/crates/v2m-core/src/asr_chinese_itn.rs:956`.
- Source review: the first rerun's missing large-place branch is fixture-gated at `fixtures/asr_chinese_itn_core.jsonl:49` through `fixtures/asr_chinese_itn_core.jsonl:55`; Rust now includes the branch at `rust/crates/v2m-core/src/asr_chinese_itn.rs:897` through `rust/crates/v2m-core/src/asr_chinese_itn.rs:930` and preserves the `十三万二三` leading-`十` quirk at `rust/crates/v2m-core/src/asr_chinese_itn.rs:1062` through `rust/crates/v2m-core/src/asr_chinese_itn.rs:1078`.
- Source review: idiom-adjacent behavior is fixture-gated at `fixtures/asr_chinese_itn_core.jsonl:56` through `fixtures/asr_chinese_itn_core.jsonl:60`; Rust consumes immediate numeric spans after an idiom at `rust/crates/v2m-core/src/asr_chinese_itn.rs:91` through `rust/crates/v2m-core/src/asr_chinese_itn.rs:101` and computes two-character lookback by character positions at `rust/crates/v2m-core/src/asr_chinese_itn.rs:225` through `rust/crates/v2m-core/src/asr_chinese_itn.rs:236`.

## Residual Risk

This rerun stayed behavior-only and reviewed the selected `chinese_to_num` seam. The Rust module is still a hand-written scanner rather than a Python regex engine, so fixture coverage remains the main guard against future drift. The only concrete gap seen in this rerun is the missing bare `三百二三` fixture, despite source inspection indicating Rust handles it.

## Promotion Note

This behavior role does not block promotion. The coordinator may treat the behavior blockers from the initial review and first rerun as fixed, subject to the separate required data/algorithm review and the low-severity fixture follow-up above.
