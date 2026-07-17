# hfa_word_model_core - behavior_reviewer rerun

Date: 2026-07-17
Decision: pass

Unit: `hfa_word_model_core`
Role: `behavior_reviewer` (manifest `stage_behavior_reviewer` evidence)

## Findings

No behavior parity findings.

The prior empty-phoneme `move_start` blocker is closed. Legacy Python evaluates
`0 <= new_start < self.phonemes[0].end` at
`inference/HubertFA/tools/align_word.py:92`, so negative, negative-infinity, and
NaN starts select the warning branch before list access. Rust now performs that
short-circuit before `first_mut()` at
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:206`. The shared fixture
proves exact warning order and unchanged Word state with a caller log list at
`rewrite-in-rust/fixtures/hfa_word_model_core.jsonl:9` and with the no-log
`UserWarning` projection at
`rewrite-in-rust/fixtures/hfa_word_model_core.jsonl:14`.

Empty-phoneme error ordering remains correct for the inputs that reach list
access. Negative zero, zero, positive infinity, and empty `move_end` project to
`IndexError: list index out of range` at
`rewrite-in-rust/fixtures/hfa_word_model_core.jsonl:10`, matching Python and the
structured Rust `HfaWordMutationError` at
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:32`.

Constructor behavior matches the selected compatibility surface. Both sides
clamp starts like Python `max(0.0, start)`, apply strict `start < end`, preserve
exact `ValueError` text, optionally initialize one full-span phoneme, and expose
the same duration. The fixture covers reversed intervals, negative zero, NaN on
each side, both infinities, and ordinary values at
`rewrite-in-rust/fixtures/hfa_word_model_core.jsonl:1` through line 4; Rust
constructor and float-formatting logic is at
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:79`,
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:113`, and
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:265`.

Word mutation behavior also matches. Contained addition, out-of-bound and
mutated zero-length rejection, contiguous append/end growth, first-append left
alignment, synchronized Word/edge-phoneme moves, special-float comparisons, and
exact log ordering are fixture-backed at
`rewrite-in-rust/fixtures/hfa_word_model_core.jsonl:5` through line 8. No-log
add, append, and boundary failures preserve the `UserWarning` category, message,
ordering, and unchanged state at
`rewrite-in-rust/fixtures/hfa_word_model_core.jsonl:11` through line 14.

Rollback and ownership remain intact. The manifest keeps the unit
`reimplemented` and `current_owner: legacy` at
`rewrite-in-rust/manifest.yaml:1302`. Production decoder, inference, and API
callers still import Python `Phoneme`, `Word`, and `WordList`; the Rust module is
only exported inside the independent workspace at
`rewrite-in-rust/rust/crates/v2m-core/src/lib.rs:17`. No production Rust route
was found.

Writer/reviewer separation is preserved. I wrote dependency/bootstrap artifacts
for this unit but did not write its Rust implementation or tests. This rerun
reviews current post-fix behavior only and does not modify production code,
fixtures, bootstrap/dependency artifacts, manifest state, or migration records.

## Checks

- `UV_CACHE_DIR=/tmp/uv-cache uv run python rewrite-in-rust/bootstrap/check_hfa_word_model_core.py`: passed.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_word`: passed; 2 selected `v2m-core` tests passed, 98 were filtered out, and 0 bridge tests were selected.
- `cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check`: passed.
- `git diff --check`: passed before this report was written.
- Static routing search: production callers remain on the Python HFA word model; no production Rust routing was found.

## Residual Risk

The proof remains fixture-bound to string text and `f64` interval values. It
does not cover arbitrary Python duck-typed inputs, warning-filter configuration,
NumPy scalar formatting outside encoded values, WordList policies, decoder
math, multi-pass aggregation, audio/export/model IO, or a future Python/Rust
bridge. Those remain outside this behavior role or belong to later units.

## Promotion Note

This behavior role no longer blocks coordinator state update. The coordinator
may record the manifest `stage_behavior_reviewer` requirement as passed using
this rerun, but must still evaluate the separately required dependency/bootstrap
and data/algorithm review evidence before marking the unit verified. This report
does not update the manifest.
