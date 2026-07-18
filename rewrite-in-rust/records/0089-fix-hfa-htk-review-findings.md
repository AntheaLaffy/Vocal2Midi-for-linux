# 0089 - Fix HFA HTK Review Findings

Date: 2026-07-18

## Context

The first behavior and data/algorithm reviews for
`hfa_htk_label_export_core` failed the record 0088 implementation. Both reviews
found that finite HTK timestamps beyond `i128` range were saturated by Rust,
while Python `int(float(time) * 10000000)` emits arbitrary-precision integer
text. The behavior review also found that empty-name `wav_path` inputs should
raise `ValueError` before directory or file side effects.

The unit remains `reimplemented`; this record documents writer follow-up work
before review reruns.

## Fixes

HTK time rendering now converts the scaled `f64` from its IEEE-754 bits into a
decimal integer string. Finite values are truncated toward zero without using a
fixed-width integer bound, matching Python's `int(float)` behavior for the
fixture-bound public surface. NaN and infinity still return Python-compatible
`ValueError` and `OverflowError` messages.

Path planning now mirrors `Path.with_suffix(".lab").name` for empty-name paths.
If `wav_path` has no file name, the planner returns a Python-compatible
`ValueError` and the partial side-effect plan accumulated before the failure.

## Fixtures

`rewrite-in-rust/fixtures/hfa_htk_label_export_core.jsonl` was expanded from 12
to 14 Python-generated rows. Added coverage locks:

- huge positive and negative finite HTK timestamps that render beyond `i128`;
- empty `wav_path` name failure before writes.

## Verification

Focused evidence:

```bash
uv run python rewrite-in-rust/bootstrap/check_hfa_htk_label_export_core.py
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_htk_label_export_core
```

The Python checker validates all 14 cases, and the Rust fixture parity test
matches the expanded projection.

## Review State

The initial review reports remain durable audit evidence:

- `rewrite-in-rust/reviews/2026-07-18-hfa_htk_label_export_core-behavior_reviewer.md`
- `rewrite-in-rust/reviews/2026-07-18-hfa_htk_label_export_core-data_algorithm_reviewer.md`

Behavior and data/algorithm reruns are required before the coordinator may mark
the unit `verified`. The existing dependency/bootstrap and error/tracing passes
remain valid unless a rerun identifies that these fixes changed their scope.

## Reversal

Rollback remains keeping Python `Exporter.save_htk` as runtime owner. No
production caller route changed.
