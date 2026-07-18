# 0106 - Fix ASR Chinese ITN Leading Dot

Date: 2026-07-18

## Unit

`asr_chinese_itn_core`

## Review Finding Addressed

The second data/algorithm rerun failed on leading malformed decimal spans such
as `点三`. Python's regex consumes the full span and preserves it unchanged,
while Rust previously skipped `点` as a non-starting character and converted the
following digit.

The behavior rerun2 also left a low-severity follow-up to fixture the bare
`三百二三 -> 320~330` form.

## Fixture Expansion

`rewrite-in-rust/fixtures/asr_chinese_itn_core.jsonl` now has 66 Python 3.12
golden cases. The 6 new cases cover:

- bare `三百二三`
- `点三`
- `我点三`
- `点三个人`
- `点三abc`
- `百分之点三`

## Rust Change

`点` is now allowed to start a one-shot candidate. The existing malformed
decimal predicates reject that candidate, so the full rejected span is preserved
instead of partially converting the following digit.

## State

The unit remains `reimplemented` until the required review rerun passes.
