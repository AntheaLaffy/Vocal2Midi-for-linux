# 0103 - Implement ASR Chinese ITN Core

Date: 2026-07-18

## Unit

`asr_chinese_itn_core`

## Change

Added an independent Rust implementation of the fixture-backed public behavior
for `inference/qwen3asr_dml/chinese_itn.py::chinese_to_num`.

The implementation lives in `rewrite-in-rust/rust/crates/v2m-core/` and is not
connected to production Python callers. Python remains the runtime owner.

## Fixture Evidence

`rewrite-in-rust/fixtures/asr_chinese_itn_core.jsonl` contains 37 golden cases
generated from the legacy Python implementation. The cases cover:

- date and time normalization
- pure digits and decimal digits
- strict single-`一` no-op behavior
- ordinary numeric values
- range expressions
- percent, fraction, and ratio conversion
- idiom and fuzzy-expression no-ops
- unit mapping for grams, kilograms, speed, people, blocks, and seconds
- ASCII head/suffix preservation
- consecutive-value quirks
- sentence-embedded numeric spans

`rewrite-in-rust/bootstrap/check_asr_chinese_itn_core.py` verifies that the
fixture outputs still match the current Python implementation.

## Implementation Notes

The Rust module does not attempt to expose a Python-compatible regex engine.
Instead it ports the observable `chinese_to_num` span-selection and conversion
behavior needed by the fixture categories. This keeps the Rust seam narrow and
rollbackable while preserving the public helper output under review.

No `ndarray`, audio, model, ONNX Runtime, PyO3, subprocess bridge, HTTP route,
or production router was introduced for this unit.

## Manifest State

`asr_chinese_itn_core` is now `reimplemented`. It still requires independent
stage behavior and data/algorithm review before it can be marked `verified`.

## Rollback

Keep `inference.qwen3asr_dml.chinese_itn.chinese_to_num` as the runtime owner.
