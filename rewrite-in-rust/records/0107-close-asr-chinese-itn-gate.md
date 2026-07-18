# 0107 - Close ASR Chinese ITN Gate

Date: 2026-07-18

## Unit

`asr_chinese_itn_core`

## Decision

Mark `asr_chinese_itn_core` as `verified`.

Python remains the runtime owner. This gate verifies the independent Rust
library seam and fixture parity only.

## Review Evidence

Required reviews:

- `reviews/2026-07-18-asr_chinese_itn_core-behavior_reviewer-rerun2.md`
  ended `pass-with-followups`. The prior behavior blockers were fixed. Its low
  follow-up requested a bare `三百二三 -> 320~330` fixture, which was added in
  record 0106.
- `reviews/2026-07-18-asr_chinese_itn_core-data_algorithm_reviewer-rerun3.md`
  ended `pass-with-followups`. The prior data/algorithm blockers were fixed.
  Its low follow-up requested malformed ratio/fraction leading-dot fixtures,
  which this record closes with `三比点四` and `三分之点四`.

Earlier failing reports remain useful evidence for what was fixed:

- malformed trailing/leading decimal no-ops
- one-shot rejected-span behavior
- range+unit behavior for `万`, `亿`, `千克`, `千米`, and `千米每小时`
- large-place plus two-digit range branch
- idiom-adjacent no-op and ASCII-adjacent conversion behavior

## Fixture Evidence

`fixtures/asr_chinese_itn_core.jsonl` now has 68 Python 3.12 golden cases.

## Verification

Required final commands:

```bash
uv run python rewrite-in-rust/bootstrap/check_asr_chinese_itn_core.py
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml asr_chinese_itn_core
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml
cargo clippy --manifest-path rewrite-in-rust/rust/Cargo.toml --all-targets --all-features -- -D warnings
uv run python -m py_compile inference/qwen3asr_dml/chinese_itn.py
uv run python scripts/audit_vendored_sources.py
```

## Rollback

Keep `inference.qwen3asr_dml.chinese_itn.chinese_to_num` as runtime owner.
