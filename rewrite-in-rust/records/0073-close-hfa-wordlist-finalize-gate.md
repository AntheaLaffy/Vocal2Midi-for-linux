# 0073 - Close HFA WordList Finalization Gate

Date: 2026-07-17

## Context

Records 0066, 0068, and 0070 established the ordered HubertFA Word lifecycle
and verified its canonical model plus collection/AP prerequisites. Record 0071
fixed a 53-case Python 3.12.13 finalization gate, and record 0072 implemented
`fill_small_gaps`, `add_sp`, and `check` on the same Rust `WordList` state.

The first data/algorithm review found two blockers: all finalizer paths cloned
the whole heterogeneous source list and `check` retained complete Word
snapshots, while the nominal interior equality fixture used a non-equal binary
float result. The first dependency review also found two stale bootstrap
declarations. Those reports remain as failure/follow-up history.

The current passing evidence is:

- dependency/bootstrap rerun:
  `rewrite-in-rust/reviews/2026-07-17-hfa_wordlist_finalize_core-dependency_bootstrap_reviewer-rerun.md`
- behavior rerun:
  `rewrite-in-rust/reviews/2026-07-17-hfa_wordlist_finalize_core-behavior_reviewer-rerun.md`
- data/algorithm rerun:
  `rewrite-in-rust/reviews/2026-07-17-hfa_wordlist_finalize_core-data_algorithm_reviewer-rerun.md`
- error/tracing rerun:
  `rewrite-in-rust/reviews/2026-07-17-hfa_wordlist_finalize_core-error_tracing_reviewer-rerun.md`

All four required roles returned `pass` with no findings.

## Decision

Accept `hfa_wordlist_finalize_core` as verified for the current legacy-owned,
no-bridge Rust library seam.

The verified unit preserves:

- leading, trailing, and interior small-gap repair in exact evaluation order,
  including strict/equal thresholds, partial mutation, warnings, aliases, and
  special floats;
- leading/interior/trailing SP insertion through the canonical validated append
  policy, source/generated identity, implicit empty/overlap discard,
  original-last trailing behavior, shared diagnostic ordering, and replacement
  only after successful candidate construction;
- final invariant checking with exact first-failure order, return values,
  messages, and repeated warning accumulation;
- structured semantic `IndexError` projection while keeping internal
  `BorrowError` out of legacy compatibility logs;
- index-wise short borrows, O(1) auxiliary storage for gap repair/check, and no
  eager cloning of invalid string tails or full Word payloads;
- caller-compatible clear/extend behavior over the same entries and log without
  migrating `hfa_api.py` short-word repair.

The equality golden now uses exactly representable `0.125` and independently
keeps a `0.25` above-threshold gap. No `discard_empty`, `get_text`, fault
injection, bridge, or production route was introduced.

## Verification

```bash
uv run python rewrite-in-rust/bootstrap/check_hfa_wordlist_finalize_core.py
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_wordlist_finalize
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_wordlist_collection
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_word
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml
cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check
cargo clippy --manifest-path rewrite-in-rust/rust/Cargo.toml --all-targets --all-features -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --manifest-path rewrite-in-rust/rust/Cargo.toml --no-deps
git diff --check
```

The shared Python/Rust finalizer table passes all 53 cases. The corrected gate
also passes a 10,000-entry short-circuit/scaling regression, the verified
45-case collection prerequisite, and the full workspace with 105 `v2m-core`
tests plus 5 bridge tests.

## Residual Risk

The independent library still models string-valued invalid entries and f64
numeric fields rather than arbitrary Python objects and dynamic int/float
formatting. `WordHandle` remains deliberately single-threaded. A production
bridge must define Python object conversion, warning/error and user-text log
presentation, identity ownership across `inference/API/hfa_api.py` mutation and
clear/extend repair, caller routing, and rollback before ownership changes.

Decoder aggregation, audio/model/export IO, and short-word repair remain
Python-owned.

## Reversal

Rollback remains keeping Python `WordList.fill_small_gaps`, `add_SP`, and
`check` as runtime owners. No production caller imports the Rust finalizer.
