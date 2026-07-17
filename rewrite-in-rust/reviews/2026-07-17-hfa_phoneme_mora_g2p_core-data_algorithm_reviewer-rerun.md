# hfa_phoneme_mora_g2p_core - data_algorithm_reviewer rerun

Date: 2026-07-17
Decision: pass

Unit: `hfa_phoneme_mora_g2p_core`
Role: `data_algorithm_reviewer`

## Findings

No data/algorithm findings. The prior Medium Unicode-version blocker is closed.

The compatibility guard in
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_g2p.rs:391` contains exactly the
55 scalars whose current Rust 1.95 / Unicode 17 single-scalar lowercase mapping
differs from Python 3.12 / Unicode 15. An independent full-scalar comparison
found no missing or extra guard entry. The full 1,112,064-scalar compatibility
digest also matches the Python-generated fixture: MD5
`463756413147af7de3cf822b56a336b1`.

The chunked lowering at
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_g2p.rs:377` preserves the guarded
scalars and applies contextual lowering to each surrounding Unicode-15 chunk.
All guarded scalars were unassigned in Unicode 15, so they were neither Cased
nor Case_Ignorable and correctly terminate contextual final-sigma runs. An
independent differential exercised all 55 scalars in nine boundary shapes with
Greek sigma, cased characters, and combining marks (495 cases), plus 1,000
deterministic mixed cases; all 1,495 public results matched Python. The shared
fixtures additionally bind all four guarded ranges and Greek final sigma.

## Confirmed Properties

The algorithm tables remain exact. A structured Python AST/Rust source audit
found the same 33 Python `_CONSONANTS` values in Rust `MORA_ONSETS`, the same 41
`_JOIN_MAP` entries in `joined_mora`, and the same 33 pure-consonant
short-circuits. Consequently the nominal separate-token join branch remains
unreachable on both sides, preserving legacy behavior. Longest-onset priority
and the phone/word/index construction order are unchanged.

Python strip and literal-space behavior remains exact. Rust
`char::is_whitespace()` plus U+001C..U+001F matches all 29 Python 3.12
`str.isspace()` scalars. Raw mode retains empty ASCII-space fields, while mora
mode strips and filters them as before.

Runtime remains linear in input bytes with fixed table factors. Lowercase
chunking visits each input scalar once; onset lookup is bounded by the fixed
33-entry table; output storage is O(input/output bytes). Guard boundaries can
create one short-lived lowercase allocation per chunk, so allocation count is
O(g) for `g` guarded scalars, but total copied data and runtime remain linear.
The focused 10,000-token regression still produces 10,000 words, 30,001 phones,
and a same-length index vector.

Generated word indexes cast `usize` to `isize`. For the owned `Vec<String>`
representation reviewed here, a physically allocatable vector cannot reach an
index beyond `isize::MAX`; validation of arbitrary Python integers supplied by
future custom `BaseG2P` bridge implementations remains outside this unit.

Writer/reviewer separation is intact. This reviewer helped write the earlier
dependency recut but did not write the Rust production implementation, its
tests, the fixture, or the checker. Independent probes remained under `/tmp`;
this rerun adds only this report.

## Checks

- `UV_CACHE_DIR=/tmp/uv-cache-hfa-g2p-data uv run python --version`: passed;
  Python 3.12.13 with Unicode 15.0.0.
- `uv run python rewrite-in-rust/bootstrap/check_hfa_phoneme_mora_g2p_core.py`:
  passed all 28 legacy fixture rows.
- Fixture structure audit: passed; 28 unique IDs, all required expectations
  present, comprising 6 base-contract, 12 mora, 9 phoneme, and 1 lowercase
  scalar-digest case.
- Full Unicode scalar audit: passed; the 55-entry compatibility guard exactly
  equals the Rust-17/Python-15 lowercase difference set, and the 1,112,064
  scalar digest matched.
- Guard-boundary public differential: passed 495 exhaustive boundary cases and
  1,000 deterministic mixed cases with zero mismatches.
- Structured table/branch/whitespace audit: passed; 33 onsets, 41 join-map
  entries, 33 pure-consonant short-circuits, and 29 whitespace scalars matched.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_phoneme_mora_g2p_core -- --nocapture`:
  passed 3 focused tests, including fixture/digest, repeated-call, and
  10,000-token coverage.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml`: passed 108
  `v2m-core` tests, 5 bridge tests, and doc tests.
- `cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check`:
  passed.
- `cargo clippy --manifest-path rewrite-in-rust/rust/Cargo.toml --all-targets --all-features -- -D warnings`:
  passed.
- `git diff --check`: passed before this report was written.
- Cargo dependency and production-routing audit: passed; there is no
  crate/lockfile change or production Python route to `hfa_g2p`.

## Residual Risk

The compatibility behavior is intentionally pinned to Python 3.12's Unicode
15 tables. Future Rust Unicode-table changes are covered by the complete guard
set and full-scalar digest, but a deliberate change of the legacy Python
version would require regenerating both. The O(g) short-lived chunk allocations
were bounded by inspection and exercised by differential tests, not measured
with an allocator profiler. Arbitrary custom-bridge Python integers still need
validation at the bridge boundary; built-in generated indexes are
allocation-bounded.

## Promotion Note

This data/algorithm gate passes and no longer blocks coordinator verification
of `hfa_phoneme_mora_g2p_core`. Other review roles remain independent. Python
remains runtime owner; this report does not change the manifest or approve
production promotion by itself.
