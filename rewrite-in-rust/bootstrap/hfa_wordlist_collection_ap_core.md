# hfa_wordlist_collection_ap_core Bootstrap

## Boundary

`hfa_wordlist_collection_ap_core` covers the decoder and pre-aggregation
`WordList` behavior in `inference/HubertFA/tools/align_word.py`:

- one canonical ordered heterogeneous collection with a persistent in-memory log;
- raw list construction and `extend` behavior that can retain non-Word,
  empty-phoneme, overlapping, or otherwise invalid pre-existing entries;
- append rejection for words without phonemes or overlapping intervals;
- overlap queries that ignore non-Word elements already present in the list;
- raw/remove interval validation and subtraction;
- AP subtraction across existing words, minimum-duration filtering, generated
  full-span phonemes, caught-error logging, and final start sorting;
- flattened phoneme-text and word-interval projections;
- language-prefix removal from stored phoneme text;
- log string, clear-log, and diagnostic ordering behavior.

Because Python subclasses `list`, storage is intentionally broader than the
validated `append(Word)` method. `overlapping_words` skips non-Word entries,
while projections and later final checks observe them. This behavior is part of
the confirmed seam and cannot be represented by silently narrowing the Rust
collection to `Vec<Word>`.

It explicitly excludes `fill_small_gaps`, `add_SP`, and `check`; those are the
post-aggregation `hfa_wordlist_finalize_core` unit.

## Dependency Expansion

The only direct source dependencies are Python list/string/sort behavior and the
`Phoneme`/`Word` types from `hfa_word_model_core`. CPython v3.12.13
[`Objects/listobject.c`](https://github.com/python/cpython/blob/v3.12.13/Objects/listobject.c)
and [`Objects/listsort.txt`](https://github.com/python/cpython/blob/v3.12.13/Objects/listsort.txt)
are the algorithm references for exact float-key comparison scheduling. The
caller's lifecycle confirms the boundary:

- `AlignmentDecoder.decode` constructs a `WordList`, appends decoded Words, and
  consumes its `phonemes` and `intervals` projections;
- `HubertFA.infer` adds non-lexical AP Words and clears language prefixes before
  duplicate-pass selection and multi-pass boundary aggregation.

NumPy computes decoder and aggregate inputs but is not needed to preserve these
collection policies. librosa/audio IO, TextGrid/export, ONNX Runtime, and model
sessions remain legacy-owned.

## Seam

- kind: independent Rust library
- crate/module: extend `v2m-core::hfa_word`
- prerequisite: verified `hfa_word_model_core`
- canonical types introduced here: `WordList` plus an entry representation that
  can retain both prerequisite Word values and pre-existing invalid/non-Word
  values used by compatibility fixtures
- canonical diagnostic storage introduced here: ordered WordList log entries
- runtime owner: legacy Python
- bridge dependencies: none

`hfa_wordlist_finalize_core` must extend this same heterogeneous WordList-entry
and log storage. It must not duplicate collection types, discard invalid entries,
or convert between parallel representations.

The Rust representation stores valid entries in `WordHandle`, whose
`Rc<RefCell<Word>>` interior is private. This is a single-threaded compatibility
invariant for Python object identity: raw seed, extend, and AP empty/no-overlap
branches retain the original handle, while AP overlap residuals allocate new
handles. Public callers receive owned snapshots and safe fallible mutation
methods; they cannot retain borrow guards across collection operations. No
unsafe code or cross-thread sharing is introduced.

## Fixture Harness

Shared Python/Rust fixture data and the legacy checker are:

```text
rewrite-in-rust/fixtures/hfa_wordlist_collection_ap_core.jsonl
rewrite-in-rust/bootstrap/check_hfa_wordlist_collection_ap_core.py
```

The 45-case fixture table encodes valid Words through the shared
`hfa_word_model_core` shape and invalid entries through explicit
`{"kind":"invalid","value":...}` entries. It covers:

- raw constructor/extend bypass of append validation and retained invalid state;
- append success, touching boundaries, insertion order, empty-phoneme and
  overlap rejection, and exact dataclass repr logs including quote, backslash,
  control/format/separator characters, Unicode escape-width boundaries, and
  preserved printable non-ASCII;
- overlap queries with touching/non-touching boundaries and retained non-Word
  elements that are ignored by overlap scanning;
- interval subtraction for no, partial, split, full, touching, invalid, NaN,
  negative infinity, and positive infinity inputs, including validation order;
- AP empty/no-overlap paths that ignore `min_dur` and preserve the original
  Word object and phoneme layout;
- AP overlap paths that subtract multiple Words with Python loop evaluation
  order, rebuild residuals as new full-span one-phoneme Words, and apply
  the default `0.1` plus explicit below/equal/negative/NaN/infinite duration
  thresholds;
- CPython 3.12.13 sort behavior across equal finite keys, eight NaN/infinity
  matrices, repeated AP calls, and 65-/127-/257-entry merge/gallop corpora;
- a full Unicode 15.0 printability digest over all 1,112,064 scalar values plus
  exact `\\x`, `\\u`, and `\\U` repr samples;
- source alias vs reconstructed fragment identity, safe mutation from source and
  entry-obtained handles, and partial mutation when a post-append sort fails;
- flattened phoneme/interval projections;
- prefix clearing using no prefix, one prefix, multiple `/` components, and a
  trailing slash, including retained mutation before a later invalid entry;
- exact uncaught projection/prefix `AttributeError` surfaces for invalid entries;
- accumulated log rendering, ordering, clearing, and resumption.

Expected commands:

```bash
uv run python rewrite-in-rust/bootstrap/check_hfa_wordlist_collection_ap_core.py
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_wordlist_collection
```

## Repeated-Call Behavior

Append and AP addition mutate order, aliases, and log state. Empty/no-overlap AP
insertion retains the caller's original Word object; overlap residuals are new
Words. The no-overlap path appends before sorting, so a caught sort failure can
leave the new Word stored. Sorting follows the CPython 3.12.13 comparison
schedule even for incomparable NaN keys. Prefix cleanup mutates stored Phoneme
text. Log entries accumulate until explicitly cleared.

## Rollback

Keep Python `WordList` as the decoder and pre-aggregation runtime owner. No
decoder, aggregation, API, GUI/Web/CLI, export, or model path imports Rust in
this unit.
