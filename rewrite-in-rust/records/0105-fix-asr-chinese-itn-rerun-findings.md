# 0105 - Fix ASR Chinese ITN Rerun Findings

Date: 2026-07-18

## Unit

`asr_chinese_itn_core`

## Rerun Findings Addressed

The first review reruns still failed on two uncovered surfaces:

- Python's range-expression branch for
  `[一二三四五六七八九十]+[万千百][一二三四五六七八九]{2}`
- idiom overlap implemented with a fixed UTF-8 byte lookback instead of
  Python's two-character string-index lookback

## Fixture Expansion

`rewrite-in-rust/fixtures/asr_chinese_itn_core.jsonl` now has 60 Python 3.12
golden cases. The 12 new cases cover:

- `三万二三`, `三万二三人`, `一万二三`, `二十万三四`,
  `十三万二三`, `三千二三`, and `三万二三千米`
- idiom-adjacent no-op behavior such as `正经八百三`
- idiom plus ASCII-adjacent conversion such as `一点一滴abc三`

## Rust Changes

The range predicate now includes the missing large-place plus two-digit branch
and reuses the existing range-pattern-2 converter. The converter also preserves
Python's leading-`十` quirk for cases like `十三万二三`, where Python uses the
first character of the base prefix.

The idiom handling now consumes immediate idiom-adjacent numeric spans as no-op,
matching cases like `正经八百三`, while ASCII-separated spans such as
`一点一滴abc三` can still convert. The generic overlap check now computes the
lookback from character positions, not a fixed byte count.

## State

The unit remains `reimplemented` until behavior and data/algorithm reruns pass.
