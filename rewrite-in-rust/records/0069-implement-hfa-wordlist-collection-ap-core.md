# 0069 - Implement HFA WordList Collection and AP Core

Date: 2026-07-17

## Context

Record 0066 split the HubertFA Word lifecycle into three ordered units, and
record 0068 verified the canonical Rust `Phoneme` and `Word` prerequisite. The
next confirmed unit owns decoder/pre-aggregation `WordList` collection and AP
behavior without taking ownership of final gap/SP/check policy.

## Decision

Extend `v2m-core::hfa_word` with one canonical heterogeneous `WordList` and
`WordListEntry` representation.

Valid Word entries use a public `WordHandle` wrapper with private
`Rc<RefCell<Word>>` storage to preserve Python list/object alias behavior in
this single-threaded library seam. The wrapper exposes owned snapshots,
identity comparison, and fallible concrete mutations, but no `Ref`, `RefMut`,
`Rc`, or `RefCell`. Raw construction and extend retain the supplied handle and
string-valued invalid entries. AP empty/no-overlap insertion retains the source
handle; overlap residuals are new full-span Word handles.

Start sorting is a safe Rust adaptation of CPython v3.12.13
[`Objects/listobject.c`](https://github.com/python/cpython/blob/v3.12.13/Objects/listobject.c)
and [`Objects/listsort.txt`](https://github.com/python/cpython/blob/v3.12.13/Objects/listsort.txt).
It preserves the raw `<` comparison schedule for natural runs, stable binary
insertion, powersort collapse, pre-merge trimming, `merge_lo`/`merge_hi`, and
galloping. This is required for Python's concrete result when `f64` keys include
NaN; a total comparator or a generic stable merge produces different orders.

Exact Python repr printability uses a generated 712-range table from the
project's Python 3.12.13 / Unicode 15.0.0 runtime. The fixture checker verifies
all 1,112,064 Unicode scalar values through a shared MD5 bitstream digest in
addition to explicit `\\x`, `\\u`, and `\\U` boundary cases.

The implementation preserves:

- raw seed/extend bypass and validated append as distinct entry paths;
- exact dataclass repr warning text, escaping, order, clear, and resume;
- overlap scanning that ignores invalid entries and treats touching as separate;
- interval validation order and subtraction over finite values, NaN, and
  infinities;
- AP branch-local minimum-duration behavior, loop evaluation order, source
  aliasing, residual reconstruction, the public `0.1` default, and repeated
  state;
- CPython 3.12.13 list-sort results for equal finite keys, multiple NaN
  placements, infinities, repeated AP calls, and long merge/gallop corpora;
- append-before-sort partial state and caught invalid-entry sort errors;
- flattened projections, slash-prefix mutation including partial mutation before
  a later invalid entry, and exact invalid-entry AttributeError projections.

`fill_small_gaps`, `add_SP`, `check`, decoder/model execution, and production
routing remain legacy-owned.

## Verification

```bash
uv run python rewrite-in-rust/bootstrap/check_hfa_wordlist_collection_ap_core.py
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_wordlist_collection
```

The shared table contains 45 cases, including 65-, 127-, and 257-entry
CPython-generated sort corpora.

## Residual Risk

Independent dependency/bootstrap, stage behavior, data/algorithm, and error
tracing reviews remain required. The compatibility invalid-entry variant is
currently string-valued because that is the executable fixture and legacy error
surface; a production bridge must define any broader Python object payload.
Finalization must extend this exact WordList/entry/log representation.

## Reversal

Keep Python `WordList` as the decoder and pre-aggregation runtime owner. No
production caller imports the Rust implementation.
