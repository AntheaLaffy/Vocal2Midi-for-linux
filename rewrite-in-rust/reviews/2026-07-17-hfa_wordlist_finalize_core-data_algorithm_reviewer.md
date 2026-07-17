# hfa_wordlist_finalize_core - data_algorithm_reviewer

Date: 2026-07-17
Decision: fail

Unit: `hfa_wordlist_finalize_core`
Role: `data_algorithm_reviewer`

## Findings

### High - whole-entry and whole-Word clones regress auxiliary space and error short-circuit complexity

All three finalization operations clone the complete heterogeneous entry vector
before inspecting its first item
(`rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:578`, `:647`, and
`:708`). The implementation comment at `:576-577` says this clones only
handles, but `WordListEntry` also contains `Invalid(String)` (`:337-341`), so
every raw invalid string is deep-cloned. This is part of the selected
compatibility surface, not an unreachable representation.

The resulting regressions are:

- Legacy invalid-first `fill_small_gaps`, `add_SP`, and `check` inspect index
  zero and stop in O(1) time and O(1) auxiliary space. Rust first spends
  O(n + total invalid-string bytes) time and space cloning the entire tail.
- Legacy successful `fill_small_gaps` scans in O(n) time with O(1) auxiliary
  space. Rust retains O(n) cloned entries/handles for the duration of the scan.
- Legacy `check` scans all Words/phonemes in O(n + p) time with O(1)
  auxiliary space. Rust first clones all entries, then `snapshot()` deep-clones
  each Word, text, and phoneme vector and retains every snapshot in `words`
  (`:708-717`, `:776`, `:779-792`). Peak auxiliary space is therefore
  O(n + p + text payload), in addition to avoidable clone work.

An independent Python 3.12.13 `tracemalloc` probe started measurement only
after constructing the input. For valid lists of 1, 1,000, and 10,000 Words,
legacy peak allocations remained 88/172/148 bytes for `fill_small_gaps` and
208/236/236 bytes for `check`. For invalid-first lists of the same sizes,
peaks stayed fixed at 598 bytes (`fill_small_gaps`), 973 bytes (`add_SP`), and
325 bytes (`check`). The Rust allocation growth follows directly from the
owned `Vec`/`String`/`Word` clones above and is asymptotically different.

Required fix:

- Do not clone the whole `entries` vector. Split short borrows of
  `self.entries` and `self.log`, access by index, and clone only the one
  `WordHandle` needed for the current operation. Do not clone `Invalid(String)`
  values merely to discover an error.
- `fill_small_gaps` must keep O(n) time and O(1) auxiliary storage while
  preserving the existing leading, trailing, then interior mutation order and
  handle alias visibility.
- `add_sp` may retain its necessary O(n) candidate vector and legacy O(n^2)
  validated-append overlap scan, but it must read source entries index-wise.
  Candidate warnings must still reach the canonical log immediately, source
  entries must remain unchanged until commit, and trailing start must still
  come from the original last source handle.
- `check` should validate one short-lived snapshot at a time, then perform a
  second index-wise pass for cross-Word adjacency (or an equivalent constant
  storage design). It must not retain all Word snapshots. Auxiliary storage
  should be O(1) apart from the current snapshot and one diagnostic string.
- Add regression evidence with long valid and invalid-first inputs so a future
  whole-vector or whole-payload clone is detected. Rerun this data/algorithm
  gate after the fix.

### Medium - the claimed interior equality fixture does not reach equality

Fixture `fill_interior_touch_equal_above` labels the boundary between `2.0`
and `2.1` as the equality case
(`rewrite-in-rust/fixtures/hfa_wordlist_finalize_core.jsonl:7`), but Python
3.12.13 evaluates `2.1 - 2.0` as `0.10000000000000009`. It therefore exercises
the above-threshold false branch, not exact `gap == gap_length`. A per-case
`sys.settrace` probe confirmed that legacy `align_word.py:231` was not reached
by this fixture; the only calls reaching that mutation line were the smaller
finite gap in `fill_order_negative_start_trailing_interior_alias` and the
positive-infinity threshold case.

The implementation's raw comparison at `hfa_word.rs:604-606` is correct by
inspection, but the durable 53-case evidence does not prove the inclusive
equality promised by the manifest/bootstrap records. Add an exactly
representable case, for example an end/start difference and `gap_length` of
`0.125`, regenerate the Python golden, and keep the explicit above-threshold
case separately.

## Checks And Confirmed Properties

- `UV_CACHE_DIR=/tmp/uv-cache-hfa-finalize-data uv run python --version`:
  Python 3.12.13.
- `uv run python rewrite-in-rust/bootstrap/check_hfa_wordlist_finalize_core.py`:
  passed all 53 legacy golden cases.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_wordlist_finalize -- --nocapture`:
  passed both focused tests, including the 53-case Rust table and structured
  borrow/index integrity test.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_wordlist_collection -- --nocapture`:
  passed the 45-case prerequisite collection gate.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_word -- --nocapture`:
  passed all 6 selected HFA tests.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml`: passed all 104
  `v2m_core` tests, all 5 `v2m_quant_bridge` tests, and doc tests.
- `cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check`:
  passed.
- `cargo clippy --manifest-path rewrite-in-rust/rust/Cargo.toml --all-targets --all-features -- -D warnings`:
  passed with no warnings.
- `git diff --check`: passed before this report was written.

The representation itself remains canonical: the Rust source contains one
`WordList` with one `Vec<WordListEntry>` and one `Vec<String>` log
(`hfa_word.rs:430-435`), and no parallel finalizer interval or diagnostic
store. `WordHandle` keeps `Rc<RefCell<Word>>` private and exposes only owned
snapshots and fallible mutations (`:251-324`). Public callers cannot retain a
borrow guard across a collection call; internal access uses `try_borrow` /
`try_borrow_mut`, and the focused conflict test returns `BorrowError` without
panic or semantic-log pollution (`:2540-2561`).

Apart from the clone regression, source/generated identity and mutation order
are correct. Entry clones retain the same `WordHandle`; `fill_small_gaps`
mutates those handles in leading/trailing/interior order; repeated positions
therefore observe the same Word. `add_sp` retains source handles, creates one
fresh handle per accepted generated Word, applies the same validated append
and overlap policy to candidates, writes append warnings to the one outer log,
commits only at `:694`, and uses the original source last handle at `:683-690`.
This matches legacy partial-state and discarded-overlap behavior.

Raw IEEE comparison behavior also matches. `fill` and `add_sp` use ordinary
`f64` comparisons, while `check` treats unordered `partial_cmp` as an invalid
interval and uses IEEE `!=` for edges/adjacency (`:719-790`), matching Python
for NaN and infinities. The table reaches NaN and both infinities. It does not
contain a durable `-0.0` finalizer case, so an independent legacy probe covered
negative-zero start through `fill`, `add_SP`, and `check`, plus a trailing
constructor error: negative zero was retained, compared equal to positive
zero, and rendered as `-0.0`. Rust's strict/equality comparisons and tested
`python_float_string(-0.0)` (`:2565-2569`) produce the same results.

A `sys.settrace` run over all 53 legacy cases confirmed reachable finalizer
control flow is effective: negative-start assignment, helper invocation,
trailing/interior repairs, leading/interior/trailing SP construction,
interior/trailing constructor errors, outer caught errors, replacement/check,
and every `check` stage were executed. The leading SP constructor exception at
legacy `align_word.py:243-244` had zero hits, consistent with the documented
mathematical exclusion: the constructor is called only after `first.start > 0`.
Short-circuit fixtures also prove an earlier Word-internal failure wins over
later Word adjacency and that invalid/empty candidate effects survive in the
shared log.

## Residual Risk

The fixture table uses a `1e-12` tolerance for ordinary Rust numeric outputs,
so it is branch/parity evidence rather than an exhaustive bit-for-bit proof of
all finite arithmetic. The independent signed-zero probe is not yet durable in
the 53-case table. Arbitrary Python invalid objects, bridge-level object alias
mapping, diagnostics crossing the bridge, and the `hfa_api.py` short-word
repair `clear()`/`extend()` workflow remain future promotion concerns. The
current `clear_entries` plus `raw_extend` operations preserve supplied Rust
handles and retain the log (`hfa_word.rs:459-468`), but no production bridge
defines how the same Python object identities survive conversion.

## Promotion Note

This data/algorithm gate fails and blocks coordinator verification of
`hfa_wordlist_finalize_core`. Fix the whole-source/whole-snapshot clone
regression, add an exact interior-equality fixture, rerun the shared and full
checks, and obtain a fresh independent data/algorithm review. This report does
not change the manifest or approve a production owner switch; Python remains
the runtime owner and rollback route described by records 0071 and 0072.
