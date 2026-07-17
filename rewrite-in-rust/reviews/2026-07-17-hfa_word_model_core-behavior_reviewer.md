# hfa_word_model_core - behavior_reviewer

Date: 2026-07-17
Decision: fail

Unit: `hfa_word_model_core`
Role: `behavior_reviewer` (manifest `stage_behavior_reviewer` evidence)

## Findings

- Severity: medium
- Location: `rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:206`
- Issue: `Word::move_start` does not preserve Python's short-circuit evaluation
  when the Word has no phonemes. Legacy Python evaluates
  `0 <= new_start < self.phonemes[0].end` at
  `inference/HubertFA/tools/align_word.py:92`. For a negative, negative-infinity,
  or NaN `new_start`, the first comparison is false, so Python never indexes
  `self.phonemes[0]`; it emits or logs
  `<text>: start >= first_phone_end，无法调整word边界` and returns normally. Rust
  calls `self.phonemes.first_mut().ok_or(HfaWordMutationError)?` before checking
  `new_start`, so every empty-phoneme call returns the projected
  `IndexError: list index out of range`, including those short-circuit inputs.
- Evidence: a targeted uv-Python probe of `Word(0.0, 1.0, "empty")` showed
  `move_start(-1.0, logs)`, `move_start(-inf, logs)`, and
  `move_start(nan, logs)` all return normally and append the boundary warning;
  `-0.0`, `0.0`, and `0.25` reach the list access and raise `IndexError`. The
  current fixture covers only empty-phoneme moves at `0.0` and `1.0` at
  `rewrite-in-rust/fixtures/hfa_word_model_core.jsonl:9`, so both harnesses pass
  without exercising the short-circuit branch. The manifest explicitly includes
  special-float boundary mutation and empty-phoneme behavior at
  `rewrite-in-rust/manifest.yaml:1313` and
  `rewrite-in-rust/manifest.yaml:1319`.
- Required fix: Check the Python first comparison (`0.0 <= new_start`) before
  accessing the first phoneme. If it is false, deliver the normal boundary
  warning through the selected sink; otherwise preserve the current empty-list
  mutation error before comparing against `first.end`. Add a shared fixture case
  for an empty Word with negative, negative-infinity, and NaN `move_start`
  values, covering both log-list and warning projections, then rerun the Python
  checker and targeted Rust test.

## Scope Notes

Constructor behavior matches the covered compatibility surface. Legacy Python
clamps start through `max(0.0, start)` and validates strict `start < end` at
`inference/HubertFA/tools/align_word.py:11` and
`inference/HubertFA/tools/align_word.py:28`. Rust uses the matching clamp and
comparison at `rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:79` and
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:113`. Shared fixtures cover
reversed intervals, negative zero, NaN on each side, both infinities, exact
error text, optional initial phonemes, and duration at
`rewrite-in-rust/fixtures/hfa_word_model_core.jsonl:1` through line 4.

Contained addition, contiguous append/end growth, mutated zero-length inputs,
boundary synchronization, and exact log ordering match the fixture table.
Their Rust implementations are at
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:150`,
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:170`, and
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:200`; shared cases are at
`rewrite-in-rust/fixtures/hfa_word_model_core.jsonl:5` through line 8.

The structured no-log warning projection preserves the legacy `UserWarning`
category, message, mutation outcome, and operation order for covered add,
append, and boundary-warning paths at
`rewrite-in-rust/fixtures/hfa_word_model_core.jsonl:10` through line 12. Empty
`move_end` behavior is not affected by the finding because Python must evaluate
`self.phonemes[-1].start` in the first comparison and therefore always raises
`IndexError` for an empty Word, matching Rust.

Rollback remains intact. The manifest keeps `current_owner: legacy` at
`rewrite-in-rust/manifest.yaml:1306`; production callers still import Python
types from `inference.HubertFA.tools.align_word`, while Rust only exposes the
independent crate module at
`rewrite-in-rust/rust/crates/v2m-core/src/lib.rs:17`. No decoder, inference,
API, GUI, Web, or CLI runtime route imports the Rust seam.

Writer/reviewer separation is preserved. I wrote the dependency/bootstrap
artifacts for this unit but did not write the Rust implementation or unit tests.
This report reviews behavior parity only and does not modify production code,
fixtures, bootstrap/dependency artifacts, manifest state, or migration records.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_hfa_word_model_core.py`: passed.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_word`: passed; 2 selected `v2m-core` tests passed, 98 were filtered out, and 0 bridge tests were selected.
- `UV_CACHE_DIR=/tmp/uv-cache uv run python -c <empty Word move_start probe>`: exposed the negative/negative-infinity/NaN short-circuit mismatch described above.
- `git diff --check`: passed before this report was written.
- Static routing search: production callers remain on Python `Phoneme`, `Word`, and `WordList`; no production Rust routing was found.

## Residual Risk

The fixture boundary is typed as string text plus `f64` times. This review did
not attempt parity for arbitrary Python duck-typed objects, warning-filter
configuration, NumPy scalar formatting beyond values encoded by the fixture,
WordList behavior, decoder math, multi-pass aggregation, audio/export/model IO,
or a future Python/Rust bridge.

## Promotion Note

This behavior role blocks coordinator state update. Do not mark
`hfa_word_model_core` verified until the empty-phoneme `move_start`
short-circuit behavior is fixed, added to the shared fixture, and this role is
rerun against the post-fix state. Dependency/bootstrap and data/algorithm roles
remain separate reviews.
