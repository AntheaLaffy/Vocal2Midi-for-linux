# hfa_wordlist_finalize_core Bootstrap

## Boundary

`hfa_wordlist_finalize_core` covers the post-aggregation methods in
`inference/HubertFA/tools/align_word.py`:

- `WordList.fill_small_gaps`: leading, trailing, and interior repair in its
  exact execution order, strict threshold comparisons, edge-phoneme mutation,
  warning delivery, broad outer error capture, and partial state after an
  earlier mutation;
- `WordList.add_SP`: leading/interior/trailing SP construction, validated
  append filtering, shared-log replacement, identity retention, caught inner
  constructor errors, final `check()` invocation, and broad outer error capture;
- `WordList.check`: return value, empty-list behavior, exact first-failure
  order/message, special-float comparisons, and repeated log accumulation.

It explicitly excludes decoder construction, AP insertion, projections, prefix
clearing, multi-pass aggregation, audio/model execution, and export. There is
no `discard_empty` or `get_text` API in the repository or migration history;
empty entries are only discarded implicitly by `add_SP`'s existing validated
`append` call.

## Dependency Expansion And Callers

The direct algorithm dependencies remain the canonical `Phoneme`/`Word` model
from `hfa_word_model_core`, the heterogeneous `WordListEntry` storage and
ordered log from `hfa_wordlist_collection_ap_core`, and Python list/interval/
diagnostic control flow. `infer_base.py` invokes finalization only after
multi-pass aggregate boundaries are reconstructed:

```text
result_word.fill_small_gaps(wav_length)
result_word.add_SP(wav_length)
warning_log = result_word.log()
```

`inference/API/hfa_api.py` then consumes the same prediction objects and runs
short-word boundary mutation plus `words.clear()`/`words.extend(kept_words)`
in its repair helper. That caller is a source/reference and alias-sensitive
ownership risk for future promotion, but it does not add a finalizer
dependency or a second collection type. NumPy, librosa, ONNX Runtime, model
sessions, export, and Python warning presentation remain legacy-owned.

## Seam And Canonical State

- kind: independent Rust library
- crate/module: extend `v2m-core::hfa_word`
- prerequisites: verified `hfa_word_model_core` and
  `hfa_wordlist_collection_ap_core`
- canonical state: reuse the existing heterogeneous `WordListEntry` values,
  `WordHandle` identity, and one ordered diagnostic buffer; do not discard raw
  invalid entries or create parallel interval/log storage
- runtime owner: legacy Python
- bridge dependencies: none

The fixture records identity labels for source Words and generated `newN` SP
Words. Repeated source IDs prove that one Word can occupy multiple list
positions and that a mutation is visible through both positions. The Rust
implementation preserves this alias behavior with the canonical safe handle
facade and does not expose a public `Rc`/`RefCell` or a second ownership model.

## Fixture Harness

The 53-case Python 3.12.13 golden table and silent checker are:

```text
rewrite-in-rust/fixtures/hfa_wordlist_finalize_core.jsonl
rewrite-in-rust/bootstrap/check_hfa_wordlist_finalize_core.py
```

The checker consumes the same `$float` markers used by the model and collection
fixtures (`nan`, `+inf`, `-inf`, and `-0.0`) and compares returns, full entry
snapshots, identity labels, and exact accumulated logs. `--generate` emits a
fresh table from the pinned legacy runtime; normal execution is silent on
success and raises an assertion with case/field context on mismatch.

### `fill_small_gaps` coverage

- empty-list exact `ERROR in fill_small_gaps`;
- negative first start set directly to integer zero without changing the first
  phoneme, followed by trailing then interior repair;
- leading strict `0 < start < gap_length < dur`, including start and duration
  equality counterexamples;
- trailing `end >= wav_length - gap_length`, including equality and below;
- interior `0 < gap <= gap_length`, including touching, equal, above, and
  overlap cases;
- invalid middle after trailing mutation, empty-phoneme partial mutation, and
  exact outer errors; helper warnings continue into later steps;
- finite, NaN, positive/negative infinity gap lengths, wav lengths, and raw
  endpoints; repeated calls with a larger wav length;
- repeated source WordHandle positions proving mutation/condition reads through
  the same alias.

### `add_SP` coverage

- empty-list exact outer error;
- leading/interior/trailing insertion, no-gap/touching boundaries, custom text,
  replacement ordering, source identity retention, and generated SP identity;
- implicit validated-append discard of empty and overlapping Words, including
  original-last-end trailing behavior and subsequent check warning order;
- invalid first/middle/last partial behavior; natural raw negative-endpoint
  fixtures for the interior and trailing `Word(...)` `ValueError` branches;
- pre-existing log preservation before temp append warnings, final check
  warnings, and outer errors;
- NaN and negative-infinity wav suppression, finite-to-`+inf` wav insertion, same-wav repetition,
  larger-wav non-idempotence, repeated-position aliasing, and clear/extend after
  finalization;
- the leading `Word(0, first.start, ...)` constructor failure is mathematically
  unreachable for accepted numeric inputs because its branch requires
  `first.start > 0`; no shared checker hook, parity case, or Rust production API
  introduces fault injection. Likewise, replace-then-outer-error from a
  monkeypatched `check()` is documented as outside the accepted seam.

### `check` coverage

- empty and valid `True` results;
- exact first-failure order for invalid entry, word interval, empty phonemes,
  first edge, last edge, phoneme interval, phoneme adjacency, and word
  adjacency;
- crossing defects proving an earlier word-internal failure wins over a later
  word gap, plus a phone interval defect whose edges/adjacency are otherwise
  valid;
- NaN invalid interval, finite-to-positive-infinity valid interval, and repeat
  calls that append duplicate warnings.

## Repeated Calls And Rollback

Both methods are stateful. `fill_small_gaps` can extend an already-repaired end
on a later larger wav length. `add_SP` replaces list contents, shares the
original log buffer, is stable for a repeated same wav, and can append a new
trailing SP for a larger wav. `check()` appends one warning per invocation.

Keep Python `WordList.fill_small_gaps`, `add_SP`, and `check` as runtime owners.
No decoder, aggregation, API, GUI/Web/CLI, export, or model path imports Rust
for this unit. A future promotion record must define Python payload conversion,
warning/error presentation, alias ownership across `hfa_api.py`'s clear/extend
repair, routing, and rollback.

## Verification

```bash
uv run python rewrite-in-rust/bootstrap/check_hfa_wordlist_finalize_core.py
```
