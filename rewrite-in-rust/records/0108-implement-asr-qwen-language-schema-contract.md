# 0108 - Implement ASR Qwen Language Schema Contract

Date: 2026-07-18

## Unit

`asr_qwen_language_schema_contract`

## Change

Added an independent Rust implementation for the fixture-backed language and
schema contracts from:

- `inference/qwen3asr_dml/utils.py::SUPPORTED_LANGUAGES`
- `inference/qwen3asr_dml/utils.py::normalize_language_name`
- `inference/qwen3asr_dml/utils.py::validate_language`
- `inference/qwen3asr_dml/schema.py` dataclasses and `MsgType`

The Rust module is `rewrite-in-rust/rust/crates/v2m-core/src/asr_qwen_language_schema.rs`.
It is not wired into production Python callers.

## Fixture Evidence

`rewrite-in-rust/fixtures/asr_qwen_language_schema_contract.jsonl` contains 57
Python 3.12 golden cases covering:

- `normalize_language_name` `None`, empty, whitespace, trim/case, Unicode, and
  integer-display behavior
- exact `SUPPORTED_LANGUAGES` order and every accepted language
- unsupported-language `ValueError` messages including Python list repr
- `MsgType` enum auto order, names, values, `str`, and `repr`
- `StreamingMessage`, `DecodeResult`, `ASREngineConfig`, and
  `TranscribeResult` default/custom snapshots and reprs
- `DecodeResult.stable_tokens` default_factory list independence
- required-constructor `TypeError` messages for dataclasses with mandatory
  fields

`rewrite-in-rust/bootstrap/check_asr_qwen_language_schema_contract.py` verifies
the fixture file against the current Python implementation.

## Dependency Boundary

This unit does not use NumPy, SciPy, pydub, ONNX Runtime, model sessions,
`ndarray`, audio loading, PyO3, subprocess bridges, HTTP, or production routing.
The broader `utils.py` imports audio dependencies, but this child unit only owns
language validation and schema DTO behavior recorded by the split bootstrap.

## Manifest State

`asr_qwen_language_schema_contract` is now `reimplemented`. It requires
independent behavior and error/tracing reviews before it can be marked
`verified`.

## Rollback

Keep `inference.qwen3asr_dml.utils` and `inference.qwen3asr_dml.schema` as
runtime owners.
