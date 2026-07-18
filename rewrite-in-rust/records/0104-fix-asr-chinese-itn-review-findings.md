# 0104 - Fix ASR Chinese ITN Review Findings

Date: 2026-07-18

## Unit

`asr_chinese_itn_core`

## Review Findings Addressed

The initial behavior and data/algorithm reviews failed the unit on uncovered
Python/Rust drift:

- malformed decimal spans such as `三点`, `一点`, and `一二点`
- rejected longer spans such as `十点二十`
- range expressions with `万`, `亿`, `千克`, `千米`, and `千米每小时`
- Rust scanner suffix retries that did not match Python's one-shot regex
  replacement behavior
- over-broad candidate suffix consumption such as `五秒钟`

## Fixture Expansion

`rewrite-in-rust/fixtures/asr_chinese_itn_core.jsonl` now has 48 Python 3.12
golden cases. The 11 new cases cover malformed decimal/time no-ops and the
range/unit edges raised by review.

The Python fixture checker passes against the current legacy implementation:

```bash
uv run python rewrite-in-rust/bootstrap/check_asr_chinese_itn_core.py
```

## Rust Changes

The Rust scanner now treats a candidate span as a one-shot replacement attempt:
when the full candidate cannot be converted, it preserves the original span
instead of retrying shorter suffixes.

Candidate boundary selection now separates the numeric body from one allowed
unit suffix. This preserves Python behavior for cases like `三个人` and
`五秒钟`, while still allowing `千克`, `千米`, and `千米每小时` to participate
in range/unit conversion.

Decimal predicates were tightened so trailing-dot or malformed decimal spans are
left unchanged, matching the Python regex contract.

## State

The unit remains `reimplemented` until the required behavior and data/algorithm
review reruns pass.
