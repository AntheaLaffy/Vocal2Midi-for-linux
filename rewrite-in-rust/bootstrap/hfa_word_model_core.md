# hfa_word_model_core Bootstrap

## Boundary

`hfa_word_model_core` covers the canonical `Phoneme` and `Word` interval model
from `inference/HubertFA/tools/align_word.py`:

- clamp negative starts to `0.0` before validation;
- require `start < end` with exact `ValueError` text;
- optionally initialize a Word with one full-span phoneme;
- expose Word duration;
- add contained phonemes without changing the Word interval;
- append contiguous phonemes while growing `Word.end`;
- move Word start/end together with the first/last phoneme boundary;
- preserve current warning text and log-list delivery behavior.

The unit does not cover `WordList`, AP/SP policies, global collection checks,
decoder math, multi-pass aggregation, audio loading, export, or model execution.

## Dependency Expansion

`align_word.py` imports only `warnings` and `dataclasses`. Production callers
import the types from:

- `inference/HubertFA/tools/decoder.py`, which constructs Phoneme and Word values
  from NumPy decoder output;
- `inference/HubertFA/tools/infer_base.py`, which reconstructs aggregate Words;
- `inference/API/hfa_api.py`, which calls `Word.move_end` during short-word
  repair before continuing through the Python pipeline.

NumPy, librosa, TextGrid/export dependencies, ONNX Runtime, and HubertFA model
execution are caller capabilities, not dependencies of this type seam. The
vendored source inventory contains their sources, but no package-level parity is
needed for this hand-written standard-library replacement.

## Seam

- kind: independent Rust library
- crate: `rewrite-in-rust/rust/crates/v2m-core`
- expected module: `hfa_word`
- canonical types introduced here: `Phoneme`, `Word`
- runtime owner: legacy Python
- bridge dependencies: none

Later HFA WordList units must reuse these types. They must not define parallel
interval models or change constructor/mutation semantics.

## Fixture Harness

Shared Python/Rust fixture data lives at:

```text
rewrite-in-rust/fixtures/hfa_word_model_core.jsonl
```

The current Python side is checked by:

```bash
uv run python rewrite-in-rust/bootstrap/check_hfa_word_model_core.py
```

The fixture table covers constructor success/failure shapes, exact error text,
negative-start clamping, optional initial phonemes, duration, contained and
rejected additions, contiguous append/end growth, rejected gaps, mutated
zero-length inputs, boundary moves, and warnings with or without a caller-owned
log list. Constructor and boundary cases use JSON-safe special-float markers to
cover reversed intervals, negative zero, NaN, and both infinities. Boundary
`move_start` without phonemes preserves Python's chained-comparison order:
negative, negative-infinity, and NaN inputs deliver the normal boundary warning
without indexing, while nonnegative inputs reach the list access and preserve
the legacy `IndexError: list index out of range` surface through a structured
Rust mutation error rather than a panic. Empty-phoneme `move_end` always reaches
the list access.

The Rust writer should consume the same JSONL table and expose a targeted test:

```bash
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_word
```

The compatibility fixture represents warning delivery as either a caller-owned
list of exact strings or captured warning records. A future bridge may choose a
structured Rust diagnostic type internally, but fixture projection must preserve
the current message and ordering before promotion explicitly changes policy.

## Repeated-Call Behavior

Construction is deterministic. Word mutations are stateful and order-sensitive:
append updates `Word.end`, and boundary moves mutate the first or last stored
phoneme. The Rust API must preserve that ownership relationship without sharing
mutable aliases outside an explicit borrow.

## Rollback

Keep `Phoneme` and `Word` in
`inference.HubertFA.tools.align_word` as runtime owners. No decoder, infer,
API, GUI, Web, CLI, or model caller imports Rust in this unit.
