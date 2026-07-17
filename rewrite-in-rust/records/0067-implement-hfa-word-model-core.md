# 0067 - Implement HFA Word Model Core

Date: 2026-07-17

## Context

Record 0066 split the HubertFA word lifecycle and confirmed
`hfa_word_model_core` as the prerequisite unit that owns canonical `Phoneme`
and `Word` types plus their local mutations. `WordList`, decoder math,
aggregation, audio/export/model IO, and production routing remain outside this
unit.

## Decision

Implement `v2m-core::hfa_word` as an independent Rust library module and keep
Python as the runtime owner.

The implementation preserves:

- Python `max(0.0, start)` clamping, including NaN, negative infinity, and
  negative zero;
- strict `start < end` validation and Python-compatible float formatting in
  exact constructor error messages;
- optional full-span initial phonemes and Word duration;
- contained add and contiguous append/end-growth behavior;
- finite, NaN, infinity, and negative-zero boundary comparison behavior;
- Python's empty-phoneme `move_start` short-circuit order: negative,
  negative-infinity, and NaN inputs warn before indexing, while nonnegative
  inputs and all empty-phoneme `move_end` calls project the list access as a
  structured `IndexError: list index out of range` rather than panicking;
- exact warning order and either `WARNING: ...` log-list delivery or a
  `UserWarning` diagnostic for caller presentation.

Special floats in the shared JSONL use `$float` markers so both Python and Rust
can consume strict JSON while preserving NaN, infinities, and negative zero.

## Verification

```bash
uv run python rewrite-in-rust/bootstrap/check_hfa_word_model_core.py
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_word
```

## Residual Risk

Independent dependency/bootstrap, stage behavior, and data/algorithm reviews
remain required. Later WordList units must extend these canonical types rather
than defining parallel interval models. Production promotion still needs a
Python-facing payload, error/warning mapping, owner switch, and rollback record.

## Reversal

Keep `Phoneme` and `Word` in `inference.HubertFA.tools.align_word` as runtime
owners. No production caller imports this Rust module.
