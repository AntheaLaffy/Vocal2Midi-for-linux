# hfa_wordlist_collection_ap_core - data_algorithm_reviewer rerun

Date: 2026-07-17
Decision: pass

Unit: `hfa_wordlist_collection_ap_core`
Role: `data_algorithm_reviewer`

## Findings

No data-structure, numeric-parity, algorithm-complexity, or aliasing findings
remain from the prior failed gate. The two prior blockers are covered by the
post-fix implementation and rerun evidence below.

## Checks

- `UV_CACHE_DIR=/tmp/uv-cache-hfa-data-rerun uv run python --version`: passed;
  the project environment is Python 3.12.13.
- `UV_CACHE_DIR=/tmp/uv-cache-hfa-data-rerun uv run python rewrite-in-rust/bootstrap/check_hfa_wordlist_collection_ap_core.py`:
  passed all 45 shared cases. The fixture includes the exact five-entry NaN
  counterexample at
  `rewrite-in-rust/fixtures/hfa_wordlist_collection_ap_core.jsonl:30`, the
  three-entry counterexample at line 29, equal-finite and NaN/Infinity matrices
  at lines 34-40, repeated AP sorting at line 41, and 65/127/257-entry
  CPython-generated corpora at lines 43-45.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_wordlist_collection -- --nocapture`:
  passed the collection fixture-table test. The Rust harness dispatches both
  `sort_key_corpus` and `unicode_printability_digest` cases at
  `rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:1938-1971`.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml word_handles_preserve_aliases_without_exposing_borrow_guards -- --nocapture`:
  passed the focused source/entry alias test at
  `rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:1981-1999`.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml`: passed all 102
  `v2m_core` tests, all 5 `v2m_quant_bridge` tests, and doc tests.
- `cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check`:
  passed.
- `cargo clippy --manifest-path rewrite-in-rust/rust/Cargo.toml --all-targets --all-features -- -D warnings`:
  passed with no warnings.
- `git diff --check`: passed before this report was written.

### Sorting differential and complexity

The exact legacy operation is `WordList.add_AP`, which appends and calls
`list.sort(key=lambda w: w.start)` on both relevant branches in
`inference/HubertFA/tools/align_word.py:193-195` and `:213`. The Rust path
decorates entries and runs `python_list_sort` at
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:676-697`.

The counterexample is distinguishing: the removed adjacent-insertion routine
would produce `[a, c, n, b, AP]` for the case-30 key sequence
`[0, -2, NaN, -1, 10]`, while CPython and the repaired implementation produce
`[a, b, c, n, AP]`. The matrix, repeated-call, and 65/127/257 corpora similarly
exercise NaN barriers, stable equal keys, powersort collapse, both merge
directions, and galloping beyond the old short-list path.

The previous adjacent-insertion implementation is gone. The replacement
retains raw directional `<` comparisons rather than inventing a non-total
`f64` comparator: natural-run discovery and stable binary insertion are at
`:731-811`; minrun and powersort pending-run collapse are at `:813-856`;
pre-merge gallop trimming and `merge_lo`/`merge_hi` selection are at
`:858-897`; and both gallop directions plus merge loops are at `:903-1256`.
The minrun cap and merge schedule give worst-case O(n log n) sorting; the
temporary left/right vectors make peak auxiliary storage O(n), while the
legacy AP interval scan remains O(n^2) independently of sorting.

An independent public sort differential wrapped the current private sort in a
temporary `/tmp` probe and generated deterministic Python 3.12.13 key lists at
lengths 2-19 and 31, 32, 33, 63, 64, 65, 66, 95, 127, 128, 129, 191, 255,
257, and 300. It compared Python `sorted(range(n), key=lambda i: keys[i])`
with Rust output for 396 lists / 24,300 entries containing repeated finite
keys, NaN, positive/negative infinity, and signed zero. All ordered labels
matched. The exact five-entry legacy probe independently produced
`[('a', -2.0), ('b', -1.0), ('c', 0.0), ('n', nan), ('AP', 10.0)]`, matching
fixture case `sort_data_reviewer_five_entry_counterexample`.

Timing the same probe on random finite keys gave `n=1000: 0.005425s`,
`n=4000: 0.013108s`, `n=16000: 0.051240s`, and `n=64000: 0.216667s`;
the approximately 4x-per-4x scaling at larger sizes is consistent with the
claimed n log n schedule and not quadratic insertion behavior. The durable
writer-selected 65/127/257 corpora remain the primary parity evidence.

### Unicode and handle safety

The independent Python digest over every Unicode scalar produced
`unicodedata 15.0.0`, 1,112,064 scalars, 963,066 non-printable values, and
MD5 `f9d22b381b01c21d615f1d6436fec3d3`, matching fixture case
`unicode_15_printability_full_scalar_digest` and the Rust digest test. The
generated table contains 712 ranges at
`rewrite-in-rust/rust/crates/v2m-core/src/python_15_nonprintable.rs:1-720`;
an independent source count also reports 712 entries. `contains` uses binary
search at `:722-731`, and Rust's scalar iteration skips
surrogates exactly as the Python checker does. Escape-width selection is at
`hfa_word.rs:1291-1323` and covers the explicit control/BMP/non-BMP samples.

`WordHandle` keeps `Rc<RefCell<Word>>` private and exposes owned snapshots,
identity checks, and selected fallible mutations at
`hfa_word.rs:251-309`; internal `read`/`write` use `try_borrow` and
`try_borrow_mut` at `:310-324`. No public caller can retain a `Ref` or `RefMut`
guard across collection operations, and the source alias plus handle cloned
from `entries()` test passes. The collection remains one canonical ordered
heterogeneous `WordListEntry` vector and one log (`hfa_word.rs:409-414`), so
raw invalid entries, alias-preserving AP branches, reconstructed residuals,
and prefix partial mutation are not represented by parallel stores.

## Residual Risk

The sorting compatibility promise is deliberately pinned to CPython 3.12.13's
concrete comparison schedule. A Python interpreter upgrade must regenerate and
rerun the NaN corpus; mathematical total-order reasoning is not a substitute.
The Unicode proof is pinned to Unicode 15.0 and Rust scalar values; Python lone
surrogates and arbitrary non-string invalid objects remain outside this seam.
`WordHandle` is intentionally single-threaded (`Rc`) and its safe mutation
facade covers the selected bridge fields, not arbitrary Python duck typing.
Final gap repair/SP/check behavior, decoder/model execution, and production
routing remain outside this unit. AP's legacy interval subtraction is still
quadratic in the number of existing words; this rerun only closes the new sort
complexity regression.

## Promotion Note

This independent data/algorithm gate passes and no longer blocks coordinator
state update. The coordinator may combine this result with the behavior,
dependency/bootstrap, and error-tracing reviews before marking the unit
verified. This report does not change the manifest or approve production owner
promotion; rollback remains the Python `WordList` implementation described in
record 0069 and the bootstrap record.
