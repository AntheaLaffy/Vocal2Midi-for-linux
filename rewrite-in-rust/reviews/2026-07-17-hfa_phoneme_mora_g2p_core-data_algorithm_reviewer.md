# hfa_phoneme_mora_g2p_core - data_algorithm_reviewer

Date: 2026-07-17
Decision: fail

Unit: `hfa_phoneme_mora_g2p_core`
Role: `data_algorithm_reviewer`

## Findings

### Medium - Rust Unicode 17 lowercasing diverges from pinned Python 3.12 Unicode 15

- Location: `rewrite-in-rust/rust/crates/v2m-core/src/hfa_g2p.rs:191`
- Issue: The Rust implementation uses the current toolchain's
  `str::to_lowercase()` for every mora token. Rust 1.95 reports Unicode 17.0,
  while project Python 3.12.13 reports `unicodedata 15.0.0`. Code points that
  were unassigned in Unicode 15 but gained case mappings in Unicode 16/17 are
  preserved by legacy `t.lower()` and changed by Rust.
- Evidence: A full-scalar independent audit found 55 single-code-point lower
  mappings that differ. Public-API probes reproduced the defect:
  U+1C89 remains U+1C89 in Python but Rust emits word U+1C8A; U+A7CB remains
  U+A7CB in Python but Rust emits U+0264; U+10D50 remains U+10D50 in Python but
  Rust emits U+10D70. In all three cases both implementations retain the
  original token in `phonemes`, so the Rust result becomes internally different
  from the legacy word/phone pairing. A 949-case table/combination differential
  produced 20 mismatching inputs, all containing these newer cased code points.
  The current fixture only binds ASCII unknown-case behavior at
  `rewrite-in-rust/fixtures/hfa_phoneme_mora_g2p_core.jsonl:17`.
- Impact: The manifest promises unknown/case-token behavior, and the public
  legacy converter accepts arbitrary Unicode strings. Results can therefore
  change merely by compiling with a newer Rust Unicode table even though the
  compatibility Python remains pinned.
- Required fix: Implement Python 3.12 / Unicode 15 lower semantics for this
  seam, or explicitly narrow and validate the accepted token alphabet before
  lowercasing. Do not fix only the three examples: bind at least representative
  Unicode 16/17 additions and preferably a full Unicode-15 lowercase digest or
  generated compatibility table so later Rust toolchain upgrades cannot repeat
  the drift. Preserve context-sensitive mappings such as Greek final sigma,
  which current Rust and Python matched in independent probes. Regenerate the
  shared fixture from Python and rerun behavior plus data/algorithm gates.

No other data-structure, table, branch-priority, index-shape, or complexity
finding was identified.

## Confirmed Properties

The local algorithm tables otherwise match exactly. An AST/source audit found
the same 33 Python `_CONSONANTS` values in Rust `MORA_ONSETS` and the same 41
`_JOIN_MAP` key/value pairs in `joined_mora`. Rust lists all two-code-point
onsets before one-code-point onsets at
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_g2p.rs:12`, preserving longest
onset precedence. Same-length ordering cannot affect prefix selection because
distinct equal-length strings cannot both prefix the same position.

The nominal separate-token join branch is unreachable on both sides. Legacy
calls `_split_mora_token` before the `JOIN_MAP` branch at
`inference/HubertFA/tools/g2p.py:103`; every one of the 33 consonants returns a
non-`None` phone vector there, so `k a` remains two words. Rust has the same
short-circuit at `rewrite-in-rust/rust/crates/v2m-core/src/hfa_g2p.rs:191`
before its join branch at line 202. Fixture line 14 binds this legacy fact, and
the independent corpus matched every onset, precomposed onset/vowel form, and
separated consonant/vowel form outside the Unicode-version finding.

Python strip and literal-space semantics are preserved. A full-scalar audit
confirmed Rust `char::is_whitespace()` plus the explicit U+001C..U+001F range at
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_g2p.rs:373` equals all 29 Python
3.12 `str.isspace()` scalars. Both raw and mora paths split only ASCII space;
raw mode retains empty fields while mora mode strips and filters them. Fixture
lines 1, 6 through 8, 10, 18, and 19 cover the distinguishing shapes.

Output construction preserves parallel structure. Each accepted word appends
its phones with the same word index and then one `SP`/`-1`; the base contract
prefixes only non-SP phones without mutating words or indexes. The 25-row table
and 949-case differential found no phone/order/index mismatch outside the word
lowercase values above. Generated word indexes cast `usize` to `isize` at
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_g2p.rs:112` and line 147. For the
current owned `Vec<String>` representation, a physically allocatable vector
cannot reach an index beyond `isize::MAX`; cross-language arbitrary-integer
payload validation remains a future bridge concern.

Complexity is linear in input bytes with a fixed table factor. Tokenization and
output construction allocate O(input/output bytes). `split_mora_token` checks a
constant 33-onset table and performs only bounded passes over the current token;
there is no input-sized nested scan. The focused 10,000-token regression at
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_g2p.rs:455` produced 10,000 words,
30,001 phones, and a matching index vector. It is a useful shape/regression
guard, though static inspection rather than that single size proves the
asymptotic bound.

Writer/reviewer separation is intact. This reviewer helped write the prior
dependency recut but did not write `hfa_g2p.rs`, its production tests, or the
fixture/checker. Independent probes were isolated under `/tmp`; this review
adds only this report.

## Checks

- `UV_CACHE_DIR=/tmp/uv-cache-hfa-g2p-data uv run python --version`: passed;
  Python 3.12.13 with Unicode 15.0.0.
- `uv run python rewrite-in-rust/bootstrap/check_hfa_phoneme_mora_g2p_core.py`:
  passed all 25 legacy fixture rows.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_phoneme_mora_g2p_core -- --nocapture`:
  passed 3 focused tests, including the 10,000-token regression.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml`: passed 108
  `v2m-core` tests, 5 bridge tests, and doc tests.
- `cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check`:
  passed.
- `cargo clippy --manifest-path rewrite-in-rust/rust/Cargo.toml --all-targets --all-features -- -D warnings`:
  passed.
- Structured table/branch audit: passed; 33 onset/consonant values, 41 join-map
  entries, 33 pure-consonant short-circuits, and 29 whitespace scalars matched.
- Independent public table/combination differential: 949 cases; all ASCII HFA
  table, onset-priority, split, phone, and index behavior matched, with 20 cases
  failing only when they included the Unicode-version counterexamples.
- Full Unicode scalar lowercase audit: failed parity for 55 code points; Rust
  reports Unicode 17.0.0 and Python reports Unicode 15.0.0.
- Contextual Unicode probes for `ΟΣ`, `ΟΣΑ`, `İ`, `ẞ`, and `A\u0307Σ`: passed;
  current Rust and Python lower outputs matched.
- `git diff --check`: passed before this report was written.
- Cargo dependency and production-routing audit: passed; no crate/lockfile
  change or production Python route to `hfa_g2p` was found.

## Residual Risk

After fixing the blocker, arbitrary Unicode string compatibility will still be
tied to Python 3.12's Unicode 15 tables and should be guarded durably. The
public Rust index representation is platform-sized `isize`, while Python custom
`BaseG2P` subclasses can theoretically return arbitrary integers; the built-in
converter-generated indexes reviewed here are allocation-bounded and safe.
Memory behavior was inspected and exercised at 10,000 tokens but not measured
with an allocator profiler.

## Promotion Note

This data/algorithm gate fails and blocks coordinator verification of
`hfa_phoneme_mora_g2p_core`. Fix and fixture-bind Python 3.12 / Unicode 15 lower
semantics, rerun focused/full checks, and obtain a fresh independent
data/algorithm review. Python remains runtime owner; this report does not update
the manifest or approve production promotion.
