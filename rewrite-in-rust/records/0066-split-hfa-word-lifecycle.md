# 0066 - Split HubertFA Word Lifecycle

Date: 2026-07-17

## Context

After `lyric_language_processor_contract` was verified, the next manifest unit
was the planned, provisional `hfa_word_interval_core` over
`inference/HubertFA/tools/align_word.py`.

That file contains three different ownership phases behind shared Python types:

1. `Phoneme` and `Word` own local interval validation and mutation;
2. `WordList` constructs decoder/pre-aggregation collections and applies AP and
   language-prefix policies;
3. the same WordList is finalized after multi-pass aggregation through gap
   repair, SP insertion, and global checks.

Keeping all three in one review unit would make local value invariants depend on
collection algorithms and would hide the caller lifecycle that determines when
each mutation is valid.

## Discovery

`align_word.py` imports only Python standard-library `warnings` and
`dataclasses`. Its algorithms are list, interval, sorting, string, and diagnostic
control flow.

The callers establish an ordered lifecycle:

- `inference/HubertFA/tools/decoder.py` constructs Phonemes, Words, and a
  WordList from NumPy decoder output, then consumes flattened projections;
- `inference/HubertFA/tools/infer_base.py` adds non-lexical AP intervals and
  clears language prefixes before duplicate-pass selection;
- after multi-pass aggregate boundaries are reconstructed, `infer_base.py`
  calls `fill_small_gaps`, then `add_SP`, then reads the accumulated log;
- `inference/API/hfa_api.py` also calls `Word.move_end` for short-word repair,
  but ONNX model loading and runtime inference remain outside this seam.

Project dependencies include NumPy, librosa, TextGrid, pydub, scipy, and torch,
and their Python/native sources are covered by the vendored source inventory.
They are caller dependencies, not dependencies of `align_word.py`; importing or
replacing them would broaden this fixture-bound work into decoder, audio,
export, or model execution.

## Decision

Replace `hfa_word_interval_core` with three confirmed, ordered units:

1. `hfa_word_model_core` owns the canonical Rust `Phoneme` and `Word` types,
   constructor invariants, duration, local add/append behavior, boundary moves,
   and warning/log-list outcomes.
2. `hfa_wordlist_collection_ap_core` depends on the first unit, introduces the
   one canonical heterogeneous Rust WordList-entry/log representation, and owns
   raw invalid entries, append/overlap, interval subtraction, AP insertion,
   projections, prefix cleanup, and log lifecycle before aggregation. It must
   retain compatibility states created through raw list construction/extend and
   cannot silently narrow storage to `Vec<Word>`.
3. `hfa_wordlist_finalize_core` depends on both prior units and extends that same
   WordList with post-aggregation gap repair, SP insertion, caught-error logging,
   and final check invariants.

All three use the default independent Rust library seam in a shared future
`v2m-core::hfa_word` module. Shared types impose dependency order; they do not
justify merging independently fixture-testable lifecycle phases. Later units
must extend canonical types rather than duplicate or translate them.

Do not add PyO3, subprocess, HTTP, CLI, runtime-router, NumPy, librosa,
TextGrid, ONNX Runtime, model-runtime, GUI, or Web dependencies to these units.

## Bootstrap Evidence

Dependency and seam records:

```text
rewrite-in-rust/dependencies/hfa_word_model_core.yaml
rewrite-in-rust/dependencies/hfa_wordlist_collection_ap_core.yaml
rewrite-in-rust/dependencies/hfa_wordlist_finalize_core.yaml
rewrite-in-rust/bootstrap/hfa_word_model_core.md
rewrite-in-rust/bootstrap/hfa_wordlist_collection_ap_core.md
rewrite-in-rust/bootstrap/hfa_wordlist_finalize_core.md
```

The first two units have executable legacy fixture gates:

```text
rewrite-in-rust/fixtures/hfa_word_model_core.jsonl
rewrite-in-rust/bootstrap/check_hfa_word_model_core.py
rewrite-in-rust/fixtures/hfa_wordlist_collection_ap_core.jsonl
rewrite-in-rust/bootstrap/check_hfa_wordlist_collection_ap_core.py
```

Run it with:

```bash
uv run python rewrite-in-rust/bootstrap/check_hfa_word_model_core.py
uv run python rewrite-in-rust/bootstrap/check_hfa_wordlist_collection_ap_core.py
```

The collection/AP gate records exact repr diagnostics, invalid-entry retention,
special-float interval/threshold/sort behavior, AP alias and reconstruction
rules, partial mutation on caught errors, projections, and prefix cleanup. The
finalization bootstrap record defines its separate fixture schema and check
command; that fixture must be added before finalization writer work begins.

## Kept Legacy

The re-cut does not migrate NumPy decoder math, librosa audio IO, multi-pass
aggregation, TextGrid/label export, ONNX Runtime sessions, HubertFA model
execution, API/GUI/Web/CLI routing, Python warning presentation, or production
bridge wiring.

## Reversal

Rollback is keeping `Phoneme`, `Word`, and `WordList` in
`inference.HubertFA.tools.align_word` as runtime owners. The manifest split can
be merged or re-cut again before implementation if fixture discovery disproves
the lifecycle boundaries; no production caller was changed.
