# hfa_wordlist_finalize_core - data_algorithm_reviewer rerun

Date: 2026-07-17
Decision: pass

Unit: `hfa_wordlist_finalize_core`
Role: `data_algorithm_reviewer`

## Findings

No data-structure, numeric-parity, aliasing, or algorithm-complexity findings
remain. Both findings from the initial failed gate are closed by the current
implementation and fixture evidence below.

## Closed Findings

### Whole-source and whole-Word clone regression

The former `self.entries.clone()` calls are gone from all three finalizers.
`fill_small_gaps_inner` records only the entry count and obtains one current
`WordHandle` at a time (`rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:576-604`).
An invalid entry at index zero therefore reaches `word_at` immediately; no tail
entry and no `Invalid(String)` payload is cloned. Successful fill remains O(n)
time and O(1) auxiliary space, with only a constant number of `Rc` handle
clones and scalar values alive at once.

`add_sp_inner` similarly reads source entries index-wise (`:644-681`). Its one
O(n) `candidates` vector is the required counterpart of legacy `words_res`:
the method must retain accepted source handles and generated SP handles until
the replacement can be committed. It does not clone the source vector or any
invalid string tail, does not mutate the source collection before `:692`, and
reads the trailing start from the original last source handle at `:681-689`.
Validated append still scans candidates for overlap, so the legacy O(n^2)
worst-case construction time and O(n) replacement storage are preserved rather
than worsened.

`check` now uses two index-wise passes with constant auxiliary storage
(`:705-796`). The first pass clones one safe handle and validates the borrowed
Word/phonemes inside one short `read` closure; it does not call `snapshot()` or
retain a Word, text, or phoneme vector. The second pass retains only two handles
and their scalar end/start values. It clones two texts only after detecting the
single cross-Word mismatch needed for the diagnostic. Thus valid check is
O(n + p) time and O(1) auxiliary space, while invalid-first check performs one
index lookup and one diagnostic allocation before returning.

The 10,000-entry regression at `:2568-2613` passes. Its valid half scans 10,000
full-span Words through fill and both check passes; its invalid-first half keeps
10,000 distinct owned tail strings and verifies exact short-circuit results for
fill, add-SP, and check without altering the 10,001-entry source. The focused
test completed in 0.02 seconds in the debug test profile. Static inspection is
the decisive allocation proof: the test is a long-input functional regression,
not a heap-allocation counter.

### Exact interior equality coverage

Fixture `fill_interior_touch_equal_above` now uses exactly representable binary
values (`rewrite-in-rust/fixtures/hfa_wordlist_finalize_core.jsonl:7`):
`2.125 - 2.0 == 0.125` for equality, and `3.25 - 3.0 == 0.25` for the separate
above-threshold branch. An independent Python 3.12.13 probe wrapped legacy
`Word.move_end`; this fixture produced exactly one call,
`('at-zero', 2.0, 2.125, 0.125)`. The equality predecessor end became `2.125`,
while the above-threshold predecessor remained `3.0`. This proves both sides
of the inclusive `gap <= gap_length` branch rather than relying on the case
name or rounded decimal display.

The case count remains 53, and both the legacy checker and Rust table consume
the regenerated golden successfully.

## Confirmed Data And Algorithm Properties

- The canonical representation remains one ordered heterogeneous
  `Vec<WordListEntry>` and one persistent `Vec<String>` log; there is no
  parallel finalizer collection, interval store, identity map, or diagnostic
  state (`hfa_word.rs:430-435`).
- `word_at` clones only a `WordHandle` for a `Word` variant and returns the
  structured attribute error directly for an invalid variant (`:799-810`).
  Source aliases and repeated list positions therefore retain one canonical
  `Rc<RefCell<Word>>` identity without deep-copying the Word.
- Fill still executes negative-start, leading, trailing, then interior logic in
  legacy order. Handle mutations remain visible through every repeated alias,
  and errors after an earlier mutation preserve that partial state.
- Add-SP retains source identities, creates a fresh handle for each generated
  Word, uses the shared validated append/overlap helper, writes candidate
  warnings directly to the canonical outer log, and commits candidates only
  after reachable construction succeeds. Discarded overlap/empty behavior and
  original-last-end trailing behavior are unchanged.
- Check still validates every Word-internal invariant before the separate
  cross-Word adjacency pass. The first failure appends one warning and returns,
  preserving the legacy short-circuit order.
- Raw IEEE comparisons remain unchanged: unordered NaN intervals fail the
  strict checks, infinities follow ordinary comparisons, signed zero compares
  equal at edges, and Python-compatible float rendering retains `-0.0`.
- Private `try_borrow`/`try_borrow_mut` access cannot panic on a borrow
  conflict. The integrity test verifies `BorrowError`, no semantic-log
  pollution, and retained entries for fill, add-SP, and check (`:2543-2565`).

## Checks

- `UV_CACHE_DIR=/tmp/uv-cache-hfa-finalize-data-rerun uv run python --version`:
  Python 3.12.13.
- `UV_CACHE_DIR=/tmp/uv-cache-hfa-finalize-data-rerun uv run python rewrite-in-rust/bootstrap/check_hfa_wordlist_finalize_core.py`:
  passed all 53 finalization cases.
- Independent exact-threshold probe: passed; equality was exactly `0.125`,
  above was exactly `0.25`, and only equality invoked legacy `move_end`.
- `UV_CACHE_DIR=/tmp/uv-cache-hfa-finalize-data-rerun uv run python rewrite-in-rust/bootstrap/check_hfa_wordlist_collection_ap_core.py`:
  passed all 45 prerequisite collection/AP cases.
- `UV_CACHE_DIR=/tmp/uv-cache-hfa-finalize-data-rerun uv run python rewrite-in-rust/bootstrap/check_hfa_word_model_core.py`:
  passed all 14 prerequisite model cases.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_wordlist_finalize_scales_and_short_circuits_invalid_first -- --nocapture`:
  passed the 10,000-entry regression.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_wordlist_finalize -- --nocapture`:
  passed all 3 focused finalizer tests.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_wordlist_collection -- --nocapture`:
  passed the prerequisite collection test.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_word -- --nocapture`:
  passed all 7 selected HFA tests.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml`: passed all 105
  `v2m_core` tests, all 5 `v2m_quant_bridge` tests, and doc tests.
- `cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check`:
  passed.
- `cargo clippy --manifest-path rewrite-in-rust/rust/Cargo.toml --all-targets --all-features -- -D warnings`:
  passed with no warnings.
- `RUSTDOCFLAGS='-D warnings' cargo doc --manifest-path rewrite-in-rust/rust/Cargo.toml --no-deps`:
  passed.
- `git diff --check`: passed before this report was written.

## Residual Risk

The 10,000-entry test proves long valid scans and exact invalid-first behavior,
while the O(1) auxiliary-space conclusion still depends on source inspection;
it does not instrument allocator traffic. Future finalizer changes should keep
the explicit absence of whole-entry clones and retained Word snapshots under
review.

The fixture table remains finite and uses a `1e-12` comparison tolerance for
ordinary Rust numeric outputs. It does not exhaust every finite `f64` bit
pattern. Arbitrary Python invalid objects, production bridge identity mapping,
diagnostic presentation, and `hfa_api.py`'s alias-sensitive short-word repair
`clear()`/`extend()` workflow remain outside this library-owned unit. Python is
still the runtime owner.

## Promotion Note

This independent data/algorithm rerun passes and no longer blocks coordinator
verification of `hfa_wordlist_finalize_core`. The coordinator may combine it
with the required dependency/bootstrap, behavior, and error/tracing decisions
before updating unit state. This report does not modify the manifest or approve
a production owner switch; rollback remains the legacy Python finalizer.
