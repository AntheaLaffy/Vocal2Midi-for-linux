# 0068 - Close HFA Word Model Gate

Date: 2026-07-17

## Context

Record 0066 split the provisional HubertFA word lifecycle into three ordered
units. Record 0067 implemented the prerequisite `hfa_word_model_core` as the
canonical Rust `Phoneme` and `Word` model without production routing.

The unit keeps `WordList`, decoder math, multi-pass aggregation, audio and
export IO, ONNX/model execution, GUI/Web/CLI routing, and production bridge
wiring legacy-owned.

The current post-fix review evidence is:

- dependency/bootstrap review:
  `rewrite-in-rust/reviews/2026-07-17-hfa_word_model_core-dependency_bootstrap_reviewer-rerun.md`
- behavior review:
  `rewrite-in-rust/reviews/2026-07-17-hfa_word_model_core-behavior_reviewer-rerun.md`
- data/algorithm review:
  `rewrite-in-rust/reviews/2026-07-17-hfa_word_model_core-data_algorithm_reviewer.md`

All three reviews returned `pass` with no findings. The behavior rerun closed
the earlier empty-phoneme `move_start` short-circuit mismatch, and the
dependency rerun confirmed that the documented test filter runs both unit
tests.

## Decision

Accept `hfa_word_model_core` as verified for the current legacy-owned,
no-bridge Rust library seam.

The verified unit preserves:

- Python `max(0.0, start)` behavior for finite values, NaN, infinities, and
  negative zero;
- strict interval validation and exact constructor error text;
- optional full-span initial phonemes and Word duration;
- contained phoneme addition and contiguous append/end growth;
- synchronized Word and first/last phoneme boundary mutations;
- Python chained-comparison short-circuit behavior;
- empty-phoneme `IndexError` outcomes where legacy Python indexes;
- exact ordered log-list and no-log `UserWarning` projections.

## Verification

```bash
uv run python rewrite-in-rust/bootstrap/check_hfa_word_model_core.py
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_word
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml
cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check
cargo clippy --manifest-path rewrite-in-rust/rust/Cargo.toml --all-targets --all-features -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --manifest-path rewrite-in-rust/rust/Cargo.toml --no-deps
git diff --check
```

The full Rust regression passed 100 `v2m-core` tests and 5 bridge tests.

## Residual Risk

Python stores the exact mutable Phoneme object supplied by a caller, while the
Rust API takes owned values. Current decoder, infer, and API callers do not rely
on cross-owner alias identity, so this does not block the independent library
seam. A production bridge must explicitly define object identity/aliasing,
warning and error payloads, caller routing, and rollback before ownership can
change.

The later `hfa_wordlist_collection_ap_core` and
`hfa_wordlist_finalize_core` units must extend these canonical Rust types rather
than introduce parallel interval representations.

## Reversal

Rollback remains keeping `Phoneme` and `Word` in
`inference.HubertFA.tools.align_word` as runtime owners. No production caller
imports the Rust module.
