# asr_chinese_itn_core - behavior_reviewer

Date: 2026-07-18
Decision: fail

## Findings

- Severity: high
- Location: rust/crates/v2m-core/src/asr_chinese_itn.rs:805
- Issue: Rust still misses one Python range-expression branch, so some public `chinese_to_num` range outputs remain incompatible.
- Evidence: The manifest requires preserving range behavior at `manifest.yaml:1733`, and the dependency record requires JSONL coverage for range patterns at `dependencies/asr_text_postprocess_contract.yaml:38`. Python recognizes `[一二三四五六七八九十]+[万千百][一二三四五六七八九]{2}{optional_unit}` as a range expression at `../inference/qwen3asr_dml/chinese_itn.py:155`, then dispatches range conversion before numeric-value conversion at `../inference/qwen3asr_dml/chinese_itn.py:437`. A Python reference probe returned `三万二三 => 32000~33000`, `三万二三人 => 32000~33000人`, `一万二三 => 12000~13000`, `二十万三四 => 23000~24000`, and `三百二三 => 320~330`. Rust only enters `convert_range_expression` when `is_range_expression` is true at `rust/crates/v2m-core/src/asr_chinese_itn.rs:142`; its predicate covers the two-leading-digit unit form, bare `十` plus two digits, and `[digit][百千][two digits]十` at `rust/crates/v2m-core/src/asr_chinese_itn.rs:810`, `rust/crates/v2m-core/src/asr_chinese_itn.rs:823`, and `rust/crates/v2m-core/src/asr_chinese_itn.rs:836`, but it has no equivalent for Python's line-155 branch. Those inputs therefore fall through toward the numeric-value path at `rust/crates/v2m-core/src/asr_chinese_itn.rs:150`, whose accepted characters and conversion do not produce range delimiters at `rust/crates/v2m-core/src/asr_chinese_itn.rs:628` and `rust/crates/v2m-core/src/asr_chinese_itn.rs:471`. `rg -n "三万二三|三百二三\"|三千二三\"|二十万三四|一万二三" fixtures/asr_chinese_itn_core.jsonl` found no coverage for this branch.
- Required fix: Add Python-generated fixtures for the missing range branch, including bare and unit-suffixed cases such as `三万二三`, `三万二三人`, `一万二三`, `二十万三四`, `三百二三`, and `三千二三`, then update Rust range detection so those candidates enter the existing `convert_range_pattern_2` path and match Python outputs.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_asr_chinese_itn_core.py`: passed, 48 fixture cases matched current Python.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml asr_chinese_itn_core`: passed, 1 Rust fixture-parity test.
- `uv run python - <<'PY' ... chinese_to_num(...) ... PY`: confirmed Python outputs for the uncovered range branch: `三万二三 => 32000~33000`, `三万二三人 => 32000~33000人`, `三千二三十人 => 3200~3300人`, `三百二三十人 => 320~330人`, `十三万二三 => 102000~103000`, `二十万三四 => 23000~24000`, `三千二三 => 3200~3300`, `三百二三 => 320~330`, `一万二三 => 12000~13000`, and `三万二三千米 => 32000~33000千米`.
- Source/diff review: the previous malformed-decimal blockers are covered by `fixtures/asr_chinese_itn_core.jsonl:38`, `fixtures/asr_chinese_itn_core.jsonl:39`, `fixtures/asr_chinese_itn_core.jsonl:40`, and `fixtures/asr_chinese_itn_core.jsonl:41`, and Rust now rejects empty decimal segments at `rust/crates/v2m-core/src/asr_chinese_itn.rs:607` and `rust/crates/v2m-core/src/asr_chinese_itn.rs:628`.
- Source/diff review: the previous `万`/`亿`/`千米`/`千克` range-unit examples are covered by `fixtures/asr_chinese_itn_core.jsonl:42` through `fixtures/asr_chinese_itn_core.jsonl:48`, and the Rust range converter no longer strips numeric-place units before pattern matching at `rust/crates/v2m-core/src/asr_chinese_itn.rs:853`.
- Source/diff review: the previous suffix-retry scanner behavior was removed; Rust now makes one replacement attempt for the full candidate at `rust/crates/v2m-core/src/asr_chinese_itn.rs:117`, with candidate suffix selection constrained at `rust/crates/v2m-core/src/asr_chinese_itn.rs:263`.

## Residual Risk

This rerun stayed behavior-only and focused on the selected `chinese_to_num` seam. The specific blockers from the first behavior review are fixed, but the fixture set still does not cover every Python `is_range_expression` branch, and the handwritten Rust predicate is still narrower than the Python regex.

## Promotion Note

This role blocks promotion. Keep `inference.qwen3asr_dml.chinese_itn.chinese_to_num` as the runtime owner under the recorded rollback route until the missing range branch is fixed and fixture-gated.
