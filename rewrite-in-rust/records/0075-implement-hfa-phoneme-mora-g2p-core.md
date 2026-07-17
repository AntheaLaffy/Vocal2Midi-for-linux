# 0075 - Implement HFA Phoneme Mora G2P Core

Date: 2026-07-17

## Context

Record 0074 split the provisional HFA G2P/config/export inventory into six
ordered units. The first confirmed writer route is
`hfa_phoneme_mora_g2p_core`, covering only `BaseG2P.__call__`, `PhonemeG2P`,
and `JapanesePhonemeMoraG2P` from `inference/HubertFA/tools/g2p.py`.

## Implementation

Added a 28-case Python 3.12 JSONL table and real-class checker. The fixture gate
preserves:

- leading/trailing/consecutive `SP` validation and exact exception projection;
- nullable, empty, and non-empty language prefixes;
- Python `str.strip()` whitespace, including U+001C..U+001F;
- literal ASCII-space splitting and empty raw-phoneme tokens;
- Japanese `SP`/`AP`/`EP`, `N`, `cl`, `I`, and `U` handling;
- joined mora spellings, the legacy separate consonant/vowel shape, multi-vowel
  and palatal fallbacks, unknown-token casing, output order, and index maps;
- stable repeated calls.

Initial independent behavior and data/algorithm reviews found that the active
Rust toolchain used Unicode 17 lowercase mappings while Python 3.12 uses Unicode
15. The difference changed unknown mora words for 55 valid scalars. The writer
added a Python-15 lowercase compatibility layer, public cases for all four newer
mapping groups, a contextual Greek-final-sigma case, and an MD5 fixture over all
1,112,064 valid scalar mappings. The historical failed reports remain as audit
evidence and require clean reviewer reruns.

Added `v2m-core::hfa_g2p` with `HfaG2pOutput`, structured `HfaG2pError`, the
shared base contract, raw phoneme conversion, and Japanese mora conversion. A
10,000-token focused test guards the linear token path. No Python production
import or caller route changed.

## State

The manifest unit is `reimplemented`. Python remains the runtime owner until
independent dependency/bootstrap, behavior, and data/algorithm reviews pass and
a later promotion record defines payload/error mapping and rollback.

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

Rollback is leaving `BaseG2P`, `PhonemeG2P`, and
`JapanesePhonemeMoraG2P` as the only production owners. The independent Rust
module can be removed without changing application behavior.
