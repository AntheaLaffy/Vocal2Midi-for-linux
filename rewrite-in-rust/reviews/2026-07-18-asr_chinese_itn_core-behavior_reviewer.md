# asr_chinese_itn_core - behavior_reviewer

Date: 2026-07-18
Decision: fail

## Findings

- Severity: high
- Location: rust/crates/v2m-core/src/asr_chinese_itn.rs:593
- Issue: Rust converts incomplete decimal spans that Python leaves unchanged.
- Evidence: Python `pure_num` requires digits after each `点` at `../inference/qwen3asr_dml/chinese_itn.py:229`, and `value_num` also requires digits after the decimal point at `../inference/qwen3asr_dml/chinese_itn.py:232`, so `replace` falls through to the original span at `../inference/qwen3asr_dml/chinese_itn.py:482`. Confirmed with `uv run python - <<'PY' ... chinese_to_num('三点') ... PY`: Python returns `三点`, `一点`, and `零点` unchanged. Rust `is_pure_num` accepts `点` without proving a following digit at `rust/crates/v2m-core/src/asr_chinese_itn.rs:607`, and `convert_pure_num` maps it to `.` at `rust/crates/v2m-core/src/asr_chinese_itn.rs:444`, so the public Rust path would convert `三点` to `3.` and `一点` to `1.`.
- Required fix: Make Rust pure-decimal span validation match Python's regex contract, including no-op behavior for trailing or leading decimal-point spans, then add fixture cases such as `三点`, `一点`, `零点`, `我三点到`, and `点三`.

- Severity: high
- Location: rust/crates/v2m-core/src/asr_chinese_itn.rs:781
- Issue: Rust strips some unit suffixes before range detection, which loses Python range semantics for bare `万`/`亿` ranges and mapped units that contain `千`.
- Evidence: Python range pattern 1 treats `[十百千万亿]` as part of the range expression at `../inference/qwen3asr_dml/chinese_itn.py:85`, and `convert_range_expression` deliberately does not strip numeric units `万`, `亿`, `千`, `百`, or `十` at `../inference/qwen3asr_dml/chinese_itn.py:167`. Confirmed with `uv run python - <<'PY' ... PY`: Python returns `三四万 => 3~4万`, `三四亿 => 3~4亿`, `三四千克 => 3~4kg`, and `三四千米 => 3~4千米`. Rust includes `万`, `亿`, `千克`, and `千米` in its unit table at `rust/crates/v2m-core/src/asr_chinese_itn.rs:7`, calls `strip_unit` before range detection at `rust/crates/v2m-core/src/asr_chinese_itn.rs:782`, and then falls through to pure-number conversion at `rust/crates/v2m-core/src/asr_chinese_itn.rs:159`; this path produces digit strings such as `34万`, `34亿`, `34kg`, and `34千米` instead of Python's range outputs.
- Required fix: Preserve Python's range-first treatment for numeric-place units and mapped units whose leading `千` is part of `_range_pattern_1`, then add fixtures for `三四万`, `二三亿`, `三四千克`, `三四千米`, and `三四千米每小时`.

- Severity: medium
- Location: fixtures/asr_chinese_itn_core.jsonl:1
- Issue: Fixture evidence is too narrow for the claimed public behavior categories.
- Evidence: The manifest claims coverage for range expressions and conversion-error/no-op behavior at `manifest.yaml:1733`, and the dependency record asks for range, unit, and exception-swallowing no-op cases at `dependencies/asr_text_postprocess_contract.yaml:38`. The 37 fixtures exercise common dates, decimals with trailing digits, range cases with `十`/`百`, idioms, fuzzy no-ops, and several units at `fixtures/asr_chinese_itn_core.jsonl:1`, but they do not include incomplete decimal no-ops or pattern-1 ranges ending in `万`/`亿`/mapped `千*` units, which are where the Rust implementation drifts.
- Required fix: Regenerate or extend the Python golden fixture file with the uncovered span-selection and no-op cases, and keep the Rust fixture test as the gate.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_asr_chinese_itn_core.py`: passed, 37 fixture cases matched current Python.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml asr_chinese_itn_core`: passed, 1 Rust fixture-parity test.
- `uv run python - <<'PY' ... chinese_to_num(...) ... PY`: confirmed Python outputs for uncovered probes: `三点`, `一点`, and `零点` remain unchanged; `三四万`, `三四亿`, `三四千克`, and `三四千米` convert to ranges.

## Residual Risk

This review stayed behavior-only and did not inspect unrelated ASR child units. Because the Rust implementation is hand-written scanner logic rather than the Python regex engine, additional uncovered span-selection drift may remain until fixtures include a broader generated matrix around decimal boundaries, numeric units, mapped units, ASCII heads/suffixes, and malformed-but-matched spans.

## Promotion Note

This role blocks promotion. Keep `inference.qwen3asr_dml.chinese_itn.chinese_to_num` as runtime owner under the recorded rollback route until the drift is fixed and the fixture evidence is expanded.
