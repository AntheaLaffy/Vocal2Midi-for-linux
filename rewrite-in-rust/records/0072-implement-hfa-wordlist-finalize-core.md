# 0072 - Implement HFA WordList Finalization Core

Date: 2026-07-17

## Context

Record 0071 confirmed the fixture-bound post-aggregation finalization seam over
the verified canonical `Phoneme`, `Word`, `WordHandle`, heterogeneous
`WordListEntry`, and persistent `WordList` log state. The 53-case Python 3.12.13
golden table fixes the ordered behavior of gap repair, SP insertion, final
checks, partial mutation, object identity, special floats, and diagnostics.

## Decision

Extend `v2m-core::hfa_word::WordList` without adding another entry, identity, or
log representation.

The implementation adds:

- `fill_small_gaps` plus the legacy `0.1` default, preserving leading,
  trailing, then interior evaluation order, strict thresholds, handle aliases,
  partial mutation, helper warnings, caught semantic errors, and IEEE values;
- `add_sp` plus the legacy `SP` default, building candidate entries through
  the same validated append policy while writing warnings to the outer log,
  replacing entries only after construction succeeds, reading the trailing
  endpoint from the original source, and ignoring the final check boolean;
- `check`, preserving exact first-failure order, messages, Python float
  formatting, repeated warning accumulation, and invalid-entry handling;
- structured `IndexError: list index out of range` mapping for local Word
  mutation failures and `clear_entries` for the caller-compatible clear/extend
  workflow without clearing diagnostics.

Validated append and overlap behavior now use private shared helpers so both
the verified collection unit and finalization candidates exercise the same
policy. `fill_small_gaps` and `add_sp` scan source entries by index, clone only
the current safe `WordHandle`, and release each collection borrow before Word
access, mutation, or diagnostic writes. They never clone invalid string tails;
`add_sp` retains only its required candidate vector and does not replace source
entries before successful construction. Repeated aliases still mutate and read
the same canonical identity.

`check` uses two index-wise passes with O(1) auxiliary storage. The first pass
validates each Word and its phonemes inside one short private read closure and
allocates a diagnostic only on failure. The second pass reads adjacent end/start
values first and clones the two Word texts only when it finds a mismatch. Borrow
conflicts remain structured `BorrowError` results and are never converted into
legacy semantic log lines.

The Rust test harness consumes the existing 53-case JSONL directly. It parses
source and reused identities, generated identities, pre-existing logs, special
floats, full Word/phoneme snapshots, repeated calls, and clear/extend actions.
No fault injection, discarded-entry API, text projection, bridge, or production
route is introduced.

The initial data/algorithm review also found that the nominal `0.1` interior
equality fixture actually produced `0.10000000000000009`. That fixture now uses
exact binary values: a `0.125` threshold and equality gap, plus a separate
`0.25` above-threshold gap. Its golden result was regenerated and checked
against the Python 3.12.13 implementation without changing the 53-case count.

## Verification

```bash
uv run python rewrite-in-rust/bootstrap/check_hfa_wordlist_finalize_core.py
uv run python --version
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_wordlist_finalize
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_wordlist_collection
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_word
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml
cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check
cargo clippy --manifest-path rewrite-in-rust/rust/Cargo.toml --all-targets --all-features -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --manifest-path rewrite-in-rust/rust/Cargo.toml --no-deps
git diff --check
```

The shared Python checker passes all 53 cases under Python 3.12.13. The Rust
finalizer consumes and passes the same 53 cases, the existing collection gate
passes all 45 cases, the HFA filter passes 7 tests, and the full workspace
passes 105 `v2m-core` tests plus 5 bridge tests. The additional Rust-only
integrity test covers direct `BorrowError` propagation and structured
`IndexError` mapping without adding a production fault-injection surface. A
10,000-entry regression covers valid fill/check scans and invalid-first
short-circuit behavior for fill, SP insertion, and check.

## Residual Risk

Independent dependency/bootstrap, stage behavior, data/algorithm, and
error/tracing reviews remain required. A future production bridge must define
arbitrary Python invalid-object conversion, exact diagnostic presentation,
Python-to-Rust identity and alias ownership across `hfa_api.py` repair,
payloads, routing, and rollback. The current Rust facade is deliberately
single-threaded, and Python remains the runtime owner.

## Reversal

Keep `inference.HubertFA.tools.align_word.WordList.fill_small_gaps`, `add_SP`,
and `check` as runtime owners. Removing the uncalled Rust finalizer restores the
pre-unit state; no Python caller, model, export, GUI, Web, or CLI path changed.
