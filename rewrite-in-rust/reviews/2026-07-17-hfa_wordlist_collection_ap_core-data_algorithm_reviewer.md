# hfa_wordlist_collection_ap_core - data_algorithm_reviewer

Date: 2026-07-17
Decision: fail

Unit: `hfa_wordlist_collection_ap_core`
Role: `data_algorithm_reviewer`

## Findings

### 1. Python sorting is not reproduced for longer NaN layouts and regresses to O(n^2)

- Severity: high
- Location: `rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:589`
- Issue: `sort_by_start` is a stable adjacent insertion sort. It is not an
  emulation of CPython 3.12's `list.sort(key=lambda w: w.start)` when the key
  relation is not a total order, and its O(n^2) time complexity regresses from
  Python Timsort's O(n log n) worst-case comparison complexity. The confirmed
  interface explicitly retains raw NaN starts and Python-style sorting, so this
  is an in-scope parity defect rather than an unsupported-input concern.
- Evidence: legacy `add_AP` appends and then invokes `list.sort` on both the
  no-overlap and overlap paths (`inference/HubertFA/tools/align_word.py:193`,
  `inference/HubertFA/tools/align_word.py:213`). Rust instead swaps adjacent
  entries only when `right.start < left.start`
  (`rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:597`). The existing
  fixture covers only the layout `[NaN, 1.0, 10.0]`
  (`rewrite-in-rust/fixtures/hfa_wordlist_collection_ap_core.jsonl:24`), for
  which both algorithms happen to retain the same order.
- Counterexample: under the project-pinned Python 3.12.13, construct `Word`
  objects `c=(0,1)`, `a=(2,3)`, `n=(4,5)`, and `b=(6,7)`; then mutate the public
  starts to `a.start=-2`, `n.start=NaN`, and `b.start=-1`. This is the same raw
  post-construction mutation already used to admit NaN starts in the fixture
  seam. The exact probe was:

  ```python
  from inference.HubertFA.tools.align_word import Word, WordList

  words = [
      Word(0.0, 1.0, "c", True),
      Word(2.0, 3.0, "a", True),
      Word(4.0, 5.0, "n", True),
      Word(6.0, 7.0, "b", True),
  ]
  words[1].start = -2.0
  words[2].start = float("nan")
  words[3].start = -1.0
  collection = WordList(words)
  collection.add_AP(Word(10.0, 11.0, "AP", True))
  print([(word.text, word.start) for word in collection])
  ```

  Python produced `[a(-2), b(-1), c(0), n(NaN), AP(10)]`. Applying the current
  Rust insertion routine to the identical key sequence
  `[0, -2, NaN, -1, 10]` produces `[a(-2), c(0), n(NaN), b(-1), AP(10)]`.
  Consequently flattened phoneme order and every later order-sensitive
  finalizer operation can diverge.
- A stable Rust `sort_by` using `left < right`, reverse `<`, and `Equal` for the
  unordered case is not a fix. A direct Rust probe produced the same incorrect
  `[a, c, n, b, AP]` order. That comparator also treats NaN as equal to every
  finite value while finite values remain unequal to each other, so its
  equivalence relation is non-transitive and does not satisfy Rust sorting's
  total-order contract.
- Required fix: replace the insertion routine with a tested CPython-3.12-compatible
  decorated-key sorting routine for the retained IEEE domain, without supplying
  Rust sort with a non-total comparator. Preserve O(n log n) whole-list behavior
  and add the exact five-entry case above plus equal finite keys, NaN in multiple
  positions, infinities, and repeated calls to the shared Python/Rust fixtures.
  Narrowing the API to finite starts would also remove the ambiguity, but would
  contradict the currently confirmed raw/NaN policy and therefore requires a
  manifest/record decision rather than an implementation-only change.

### 2. Public aliased handles expose RefCell borrow panics

- Severity: medium
- Location: `rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:251`,
  `rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:376`,
  `rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:395`
- Issue: `WordHandle` publicly exposes `Rc<RefCell<Word>>`, and both caller-owned
  source handles and handles returned through `WordList::entries()` can be
  borrowed independently of `WordList`. Collection methods then use panicking
  `borrow()`/`borrow_mut()` calls. Holding a valid external mutable borrow across
  `append`, `overlapping_words`, `add_ap`, sorting, projection, or prefix cleanup
  aborts the operation with a Rust panic rather than a result or persistent
  diagnostic. Method-local borrow lifetimes prevent internal list-mutation
  conflicts, but do not protect against these public external aliases.
- Evidence: this standalone probe against the built `v2m_core` library exits 101
  with `RefCell already mutably borrowed` at `hfa_word.rs:397`:

  ```rust
  let word = Rc::new(RefCell::new(
      Word::new(0.0, 1.0, "x", true).unwrap(),
  ));
  let _guard = word.borrow_mut();
  let mut words = WordList::new();
  words.append(Rc::clone(&word));
  ```

  The same panicking calls occur during overlap scans, AP subtraction and sort,
  projections, and prefix mutation (`rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:377`,
  `rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:461`,
  `rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:496`,
  `rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:540`,
  `rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:578`,
  `rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:605`).
- Required fix: either encapsulate the handle so callers cannot retain raw
  `Ref`/`RefMut` guards across collection calls, or use fallible borrow access
  and define a non-panicking error/diagnostic contract for borrow conflicts.
  Because several current methods are infallible, this may require a deliberate
  public API adjustment. Add a regression test exercising a retained source
  alias and an alias cloned from `entries()`.

## Checked Scope

- The canonical storage shape otherwise fits the selected compatibility seam:
  one `WordList` owns ordered heterogeneous `WordListEntry` values and one log,
  valid entries retain shared Word identity, and raw constructor/extend paths are
  distinct from validated append (`rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:263`,
  `rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:329`). Empty/no-overlap AP
  paths retain the source handle; overlap residuals allocate new full-span Words.
- Interval validation and subtraction preserve Python evaluation order and raw
  IEEE comparison behavior. Raw interval validation precedes remove interval
  validation; the local `python_max`/`python_min` helpers preserve first-argument
  selection for equal, signed-zero, and unordered comparisons
  (`inference/HubertFA/tools/align_word.py:157`,
  `rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:425`).
- AP residual derivation retains legacy order, inclusive `duration >= min_dur`
  filtering, the public `0.1` default, branch-local ignoring of `min_dur`, full-span
  one-phoneme reconstruction, append-before-sort partial mutation, and caught
  error logging (`inference/HubertFA/tools/align_word.py:181`,
  `rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:460`). The subtraction
  pass has the same O(n^2) worst case as legacy because each processed Word can
  grow the residual set by at most one; the additional O(n^2) sort is the new
  regression identified above.
- Flattened projections and language-prefix clearing preserve list/phoneme order,
  last-component selection, and mutation of earlier valid entries before a later
  invalid entry fails. Repeated AP calls and log accumulation are stateful as in
  Python.
- The planned finalizer can extend the same private entry and log storage without
  a parallel collection representation. Its `add_SP` port must still prove the
  temporary-list shared-log behavior at `inference/HubertFA/tools/align_word.py:237`;
  the current owned `Vec<String>` is not itself shareable between two `WordList`
  values, although the behavior can be implemented against the original list or
  by a carefully restored temporary owner.

## Checks

- `env UV_CACHE_DIR=/tmp/uv-cache-hfa-data-review uv run python rewrite-in-rust/bootstrap/check_hfa_wordlist_collection_ap_core.py`: passed all 28 current fixture rows.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_wordlist_collection`: passed; 1 selected Rust test passed.
- Project Python 3.12.13 exact legacy Word probe above: reproduced
  `[a, b, c, n, AP]` from input `[c, a, n, b, AP]` with starts
  `[0, -2, NaN, -1, 10]`.
- Read-only Python/Rust algorithm probe: current insertion and the proposed
  unordered-as-Equal stable `sort_by` both produced `[a, c, n, b, AP]` for that
  layout.
- Standalone linked-library borrow probe: reproduced exit 101 and
  `RefCell already mutably borrowed` at `WordList::append`.

## Residual Risk

`WordListEntry::Invalid` currently models only string-valued invalid entries.
That matches the executable seam and is explicitly recorded as a bridge decision
still to make, but arbitrary Python non-Word values could require richer type/error
metadata later. No model inference or production routing was exercised.

CPython's order for unordered keys depends on its concrete list-sort algorithm,
not a mathematical ordering relation. The project currently pins Python 3.12;
any future interpreter-range change must rerun the NaN ordering corpus rather
than assuming another CPython version makes the same comparison schedule.

## Promotion Note

This role blocks verification and promotion. The sorting mismatch must be fixed
and added to the shared parity table, and the public alias borrow-panic surface
must be removed or explicitly redesigned before rerunning the independent
data/algorithm gate. The existing passing fixture commands do not cover either
finding.
