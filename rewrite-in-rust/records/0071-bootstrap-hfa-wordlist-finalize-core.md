# 0071 - Bootstrap HFA WordList Finalization

Date: 2026-07-17

## Context

Record 0066 split the HubertFA Word lifecycle into model, collection/AP, and
post-aggregation finalization phases. Records 0068 and 0070 verified the first
two phases over canonical `Phoneme`/`Word` and heterogeneous `WordListEntry`,
`WordHandle`, and log state. `hfa_wordlist_finalize_core` remains planned and
legacy-owned; this record adds its executable legacy fixture gate without
writing Rust production code or changing runtime routing.

## Decision

Keep the default independent-library seam and extend the exact canonical
collection/log state from `hfa_wordlist_collection_ap_core`. Do not add a
`discard_empty` or `get_text` API: empty entries are discarded only as a side
effect of the existing validated `WordList.append` call inside `add_SP`.

The fixture/checker pair is:

```text
rewrite-in-rust/fixtures/hfa_wordlist_finalize_core.jsonl
rewrite-in-rust/bootstrap/check_hfa_wordlist_finalize_core.py
```

The table has 53 Python 3.12.13 golden cases. It asserts return values, full
entry snapshots, source/generated identity labels, and exact accumulated logs.
Special floats use the shared `$float` markers. `--generate` emits fresh
goldens from the pinned legacy runtime; normal checker execution is silent on
success.

## Coverage

`fill_small_gaps` covers empty outer errors; negative-start integer-zero
mutation without first-phoneme mutation; leading strict inequalities and both
equality boundaries; trailing equality/below; interior touching/equal/above/
overlap; operation order; invalid-middle partial mutation after trailing
repair; empty-phoneme errors; helper warnings; NaN and positive/negative
infinity gap, wav, and raw endpoint values; repeated larger-wav calls; and the
same source Word at two list positions.

`add_SP` covers empty, leading/interior/trailing/custom/no-gap/touching paths;
source versus generated identity; implicit empty/overlap discard and original
last-end trailing semantics; invalid first/middle/last partial state; natural
raw negative-endpoint interior and trailing constructor `ValueError` branches;
shared pre-existing log ordering; NaN and positive/negative-infinity wav behavior;
same-wav/larger-wav repetition; repeated-position aliasing; and clear/extend
replacement. The leading generated `Word(0, first.start, ...)` constructor
failure is mathematically unreachable for accepted numeric inputs because its
branch requires `first.start > 0`; it is documented as an excluded control-flow
probe rather than a Rust production fault-injection requirement. A
replace-then-outer-error caused only by monkeypatching `check()` is likewise
excluded from the shared parity table.

`check` covers empty/valid results, every first-failure branch and exact order,
crossing defects where an earlier word-internal failure beats a later word
gap, a phone interval failure with otherwise valid edges/adjacency, NaN
invalid and finite-to-positive-infinity valid intervals, and repeated warning
logs.

The caller references include `inference/API/hfa_api.py`: after `infer` it
continues to mutate the same prediction objects and performs `clear()`/
`extend()` in short-word repair. This is an alias-sensitive promotion risk and
must be addressed in a future bridge/owner-switch record; it does not add a
second finalizer seam.

## Verification

```bash
uv run python rewrite-in-rust/bootstrap/check_hfa_wordlist_finalize_core.py
uv run python --version
git diff --check
```

The checker passes all 53 cases under project Python 3.12.13. Manifest status
stays `planned`; no Rust production code, bridge dependency, or runtime owner
change is part of this bootstrap.

## Reversal

Keep Python `WordList.fill_small_gaps`, `add_SP`, and `check` as runtime owners.
Promotion must define Python payload conversion, warning/error presentation,
`hfa_api.py` alias behavior, routing, and rollback before any owner switch.
