# 0076 - Close HFA Phoneme Mora G2P Gate

Date: 2026-07-17

## Context

Record 0075 implemented `hfa_phoneme_mora_g2p_core` behind the independent
Rust library seam. Required independent reviews covered dependency/bootstrap,
behavior parity, and data/algorithm properties.

## Initial Finding

The first behavior and data/algorithm reviews failed with one Medium blocker:
Rust 1.95 used Unicode 17 lowercase mappings for 55 scalars that Python 3.12's
Unicode 15 tables leave unchanged. The mismatch changed unknown Japanese mora
word text while leaving phones and indexes intact.

The writer fixed the blocker with a Python-15 compatibility layer that preserves
the exact 55-member set while delegating surrounding chunks to contextual Rust
lowercase. The shared table grew from 25 to 28 rows and now includes public
representatives for all four mapping groups, Greek final sigma, and an MD5 over
all 1,112,064 valid scalar lowercase mappings.

## Review Evidence

All current reruns passed with no findings:

- dependency/bootstrap confirmed the six-unit recut, pure local seam, exact
  55-member reference, no new crate/bridge, legacy ownership, and rollback;
- behavior compared all 1,112,064 real public scalar outputs plus 949 contextual
  cases byte for byte across Python and Rust;
- data/algorithm proved the guard equals the complete Unicode 17/15 difference
  set, matched the full digest, ran 1,495 boundary/mixed cases, and rechecked
  tables, whitespace, complexity, and the 10,000-token regression.

The historical failed reports remain beside the passing reruns as audit
evidence.

## Decision

Mark `hfa_phoneme_mora_g2p_core` as `verified`. Keep `current_owner: legacy`.
No production Python import, inference path, API route, CLI route, or bridge was
changed.

## Verification

```bash
uv run python rewrite-in-rust/bootstrap/check_hfa_phoneme_mora_g2p_core.py
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_phoneme_mora_g2p_core
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml
cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check
cargo clippy --manifest-path rewrite-in-rust/rust/Cargo.toml --all-targets --all-features -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --manifest-path rewrite-in-rust/rust/Cargo.toml --no-deps
uv run python scripts/audit_vendored_sources.py
```

## Reversal

Rollback remains leaving `BaseG2P`, `PhonemeG2P`, and
`JapanesePhonemeMoraG2P` as production owners. Promotion must separately define
payload validation, Python-facing error mapping, caller routing, and rollback.
