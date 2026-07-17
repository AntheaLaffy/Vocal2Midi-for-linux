# hfa_phoneme_mora_g2p_core - behavior_reviewer rerun

Date: 2026-07-17
Decision: pass

Unit: `hfa_phoneme_mora_g2p_core`
Role: `behavior_reviewer` (manifest `stage_behavior_reviewer` evidence)

## Findings

No behavior parity findings in the current post-fix implementation.

The prior Unicode-version blocker is closed. `python_15_lowercase` at
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_g2p.rs:377` preserves the 55 scalar
values that gained lowercase mappings after Python 3.12's Unicode 15 tables,
while applying Rust lowercase to the surrounding chunks. The isolated set at
lines 391 through 404 exactly covers the four previously observed groups: two
single ranges plus U+10D50..U+10D65 and U+16EA0..U+16EB8. The mora parser now
uses that compatibility function before split/fallback decisions at line 191.

The repaired public behavior matches independently, not only through an internal
helper test. The four-category probe that previously failed now returns identical
Python/Rust phonemes, words, prefixes, and indexes for U+1C89, U+A7CB, U+10D50,
and U+16EA0. A complete public differential called the real legacy
`JapanesePhonemeMoraG2P` and Rust `japanese_phoneme_mora_g2p` once for every
valid Unicode scalar. All 1,112,064 output rows matched byte for byte, including
the 55 repaired identities, with SHA-256
`f52021c01e0c91232687223f9a3a434349fe06e786ed2939f2831e19daaf0ff3`.

Chunk boundaries do not regress contextual lowercase. A second public
differential combined each of the 55 isolated scalars with 17 contexts around
Greek sigma, cased letters, combining marks, dotted I, and sharp S, then added
14 baseline mora/fallback strings. All 949 cases matched byte for byte with
SHA-256 `231fe821c893a62f595aa473d993e0e0efca8f650f85baa767426566464536fa`.
The durable fixture independently covers Greek final sigma at
`rewrite-in-rust/fixtures/hfa_phoneme_mora_g2p_core.jsonl:21` and validates all
Unicode 15 scalar lowercase mappings at line 22 with count 1,112,064 and MD5
`463756413147af7de3cf822b56a336b1`. The Python checker computes that digest from
Python 3.12 `str.lower()` at
`rewrite-in-rust/bootstrap/check_hfa_phoneme_mora_g2p_core.py:52`; Rust computes
it from the production compatibility function at `hfa_g2p.rs:463`.

The original Base and raw-phoneme contracts remain exact. Rust still validates
empty, boundary, and consecutive `SP` states before prefixing, preserves exact
`IndexError`/empty-message `AssertionError`, and leaves words/indexes untouched
at `rewrite-in-rust/rust/crates/v2m-core/src/hfa_g2p.rs:67`. `phoneme_g2p` still
uses Python-compatible outer strip, literal ASCII-space splitting, exact
uppercase `SP` filtering, retained empty tokens, ordered interleaving, and
nullable/empty/string prefixes at line 102. Fixture lines 1 through 9 and 23
through 28 cover these return, error, ordering, and mapping behaviors.

Japanese mora grouping also remains unchanged outside the compatibility fix.
Control filtering, `N`/`cl`, vowel normalization, joined spellings, the legacy
separate consonant/vowel shape, multi-vowel/palatal fallbacks, original-phone
versus lowercase-word output, `SP` placement, and word indexes remain ordered at
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_g2p.rs:134`. Fixture lines 10
through 21 cover those branches, including all four repaired unknown-word
identity categories. The prior independent 12-case string probe still matches
byte for byte with SHA-256
`505b351c306067503565d86854b7fc50f957d79dc6e667ac93477eeec28eeb23`;
it covers contextual lower, U+001C..U+001F strip, literal spaces, language
prefixing, controls, special vowels, joined/fallback forms, order, and indexes.
Both runtimes also retain stable repeated calls.

The expanded shared gate consumes the same 28-row file. The legacy checker
imports and calls the real Python classes at
`rewrite-in-rust/bootstrap/check_hfa_phoneme_mora_g2p_core.py:52`, while the Rust
test embeds that JSONL and compares exact results at
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_g2p.rs:417` and line 484. Both pass
the new public identity, contextual, and full-scalar cases as well as the
original 25 cases.

Scope, production ownership, and rollback remain intact. Static search found
only the independent Rust module and its tests calling the Rust G2P functions;
`infer_base.py` and `hfa_api.py` still route to the legacy Python classes. No
DictionaryG2P, file/warning, dataset, YAML/config, export, model, bridge, or
frontend behavior was added to this unit. The manifest remains
`status: reimplemented` and `current_owner: legacy` at
`rewrite-in-rust/manifest.yaml:1411`; rollback remains leaving `BaseG2P`,
`PhonemeG2P`, and `JapanesePhonemeMoraG2P` as runtime owners.

Writer/reviewer separation is preserved. This rerun did not modify production
code, tests, fixtures, checker, dependency/bootstrap artifacts, records, or the
manifest. It adds only this report; all independent probes were isolated under
`/tmp`. The original failed behavior report remains as historical finding and
fix evidence.

## Checks

- `UV_CACHE_DIR=/tmp/uv-cache uv run python --version`: passed; Python 3.12.13.
- `UV_CACHE_DIR=/tmp/uv-cache uv run python rewrite-in-rust/bootstrap/check_hfa_phoneme_mora_g2p_core.py`: passed; validated all 28 fixtures and repeated calls, including Unicode version `15.0.0`, 1,112,064 scalars, and MD5 `463756413147af7de3cf822b56a336b1`.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_phoneme_mora_g2p_core -- --nocapture`: passed 3 focused tests: exact expanded fixture table, 10,000-token path, and repeated calls.
- Independent four-category public identity differential: passed byte for byte; SHA-256 `db0cd6abc14c5cd875478c59774c752ca1fdac19fb2a5c3c777bee86fa1ab283`.
- Independent full-scalar public differential: passed all 1,112,064 Python/Rust output rows byte for byte; each output file was 45 MB with SHA-256 `f52021c01e0c91232687223f9a3a434349fe06e786ed2939f2831e19daaf0ff3`.
- Independent 949-case contextual public differential: passed all 55 isolated scalars x 17 contexts plus 14 baselines byte for byte; SHA-256 `231fe821c893a62f595aa473d993e0e0efca8f650f85baa767426566464536fa`.
- Independent 12-case strip/split/prefix/fallback public differential: passed byte for byte; SHA-256 `505b351c306067503565d86854b7fc50f957d79dc6e667ac93477eeec28eeb23`.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml`: passed 108 `v2m-core` tests, 5 bridge tests, and doc tests.
- `cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check`: passed.
- `cargo clippy --manifest-path rewrite-in-rust/rust/Cargo.toml --all-targets --all-features -- -D warnings`: passed.
- `RUSTDOCFLAGS='-D warnings' cargo doc --manifest-path rewrite-in-rust/rust/Cargo.toml --no-deps`: passed.
- `git diff --check`: passed before this report was written.
- Static scope/routing search: passed; no production Rust G2P route or out-of-scope implementation was found.

## Residual Risk

The compatibility function is intentionally pinned to Python 3.12/Unicode 15.
The durable full-scalar digest will fail if a future Rust Unicode table adds
another mapping, but maintainers must then extend the isolation set rather than
regenerate the expected digest against the newer behavior. The independent
public proof exhausts every single scalar and selected multi-character contexts,
not every possible Unicode string. Rust UTF-8 also cannot represent lone Python
surrogates; the documented seam already limits inputs to UTF-8.

Dictionary warnings/files, dataset/lab IO, future bridge payload validation and
Python-facing error mapping, model execution, config/export behavior, production
routing, and owner-switch rollback remain outside this unit.

## Promotion Note

The prior behavior blocker is closed, and this rerun passes the manifest
`stage_behavior_reviewer` requirement. This role does not block coordinator
state update, but the coordinator must still evaluate current passing evidence
for every separately required review, including the post-fix data/algorithm
rerun, before marking the unit verified. This report does not update the
manifest or approve production routing.
