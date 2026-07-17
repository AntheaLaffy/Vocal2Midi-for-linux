# 0070 - Close HFA WordList Collection and AP Gate

Date: 2026-07-17

## Context

Record 0066 split the HubertFA Word lifecycle into three ordered units, record
0068 verified the canonical Rust `Phoneme` and `Word` prerequisite, and record
0069 implemented decoder/pre-aggregation collection and AP behavior over one
heterogeneous `WordList` representation.

The first behavior and data/algorithm reviews found three blockers: the
insertion sort did not reproduce CPython 3.12 when NaN keys changed comparison
order, exact Word repr did not escape all Python-nonprintable Unicode, and the
public `Rc<RefCell<Word>>` alias permitted caller-triggered borrow panics. Those
reports remain as failure history.

The post-fix passing evidence is:

- dependency/bootstrap review:
  `rewrite-in-rust/reviews/2026-07-17-hfa_wordlist_collection_ap_core-dependency_bootstrap_reviewer.md`
- behavior rerun:
  `rewrite-in-rust/reviews/2026-07-17-hfa_wordlist_collection_ap_core-behavior_reviewer-rerun.md`
- data/algorithm rerun:
  `rewrite-in-rust/reviews/2026-07-17-hfa_wordlist_collection_ap_core-data_algorithm_reviewer-rerun.md`
- error/tracing review:
  `rewrite-in-rust/reviews/2026-07-17-hfa_wordlist_collection_ap_core-error_tracing_reviewer.md`

All four required roles returned `pass` with no findings.

## Decision

Accept `hfa_wordlist_collection_ap_core` as verified for the current
legacy-owned, no-bridge Rust library seam.

The verified unit preserves:

- canonical ordered heterogeneous entries and persistent diagnostic state;
- raw construction/extend bypass versus validated append behavior;
- overlap, interval subtraction, AP filtering/reconstruction, original versus
  fragment alias identity, partial mutation, and exact error/log order;
- CPython 3.12.13 stable list-sort comparison scheduling for finite, equal,
  NaN, and infinite starts with O(n log n) worst-case sorting;
- Python 3.12.13 / Unicode 15 repr printability and `\\x`, `\\u`, and `\\U`
  escape widths across all Unicode scalar values;
- flattened projections and ordered language-prefix mutation;
- shared Word identity through a safe `WordHandle` whose private interior does
  not expose borrow guards or a public panic path.

`fill_small_gaps`, `add_SP`, and `check` remain the planned
`hfa_wordlist_finalize_core` unit. Decoder math, multi-pass aggregation,
audio/model/export behavior, production routing, and Python presentation remain
legacy-owned.

## Verification

```bash
uv run python rewrite-in-rust/bootstrap/check_hfa_wordlist_collection_ap_core.py
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_wordlist_collection
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml
cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check
cargo clippy --manifest-path rewrite-in-rust/rust/Cargo.toml --all-targets --all-features -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --manifest-path rewrite-in-rust/rust/Cargo.toml --no-deps
git diff --check
```

The shared checker and Rust fixture test pass all 45 cases. Independent
reviewers also matched CPython sort results over additional mixed IEEE corpora,
confirmed n-log-n scaling, and verified the full Unicode scalar digest. The
full Rust regression passed 102 `v2m-core` tests and 5 bridge tests.

## Residual Risk

The sort contract is pinned to CPython 3.12.13 and the repr table to Unicode
15.0; interpreter or Unicode upgrades must regenerate and rerun their corpora.
Rust `String` cannot represent lone Python surrogates, invalid entries currently
model the string-valued compatibility surface, and `WordHandle` is deliberately
single-threaded. A production bridge must define broader Python object
conversion, aliasing, warning/error payloads, user-text log handling, caller
routing, and rollback before ownership changes.

The finalizer must extend this exact `WordListEntry`, Word identity, and log
state rather than introduce parallel storage.

## Reversal

Rollback remains keeping Python `WordList` construction, AP insertion,
projections, and prefix cleanup as runtime owners. No production caller imports
the Rust implementation.
