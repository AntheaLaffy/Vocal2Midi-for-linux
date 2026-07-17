# hfa_phoneme_mora_g2p_core - behavior_reviewer

Date: 2026-07-17
Decision: fail

Unit: `hfa_phoneme_mora_g2p_core`
Role: `behavior_reviewer` (manifest `stage_behavior_reviewer` evidence)

## Findings

### Medium - Rust lowercasing changes valid UTF-8 mora words that Python 3.12 preserves

- Location: `rewrite-in-rust/rust/crates/v2m-core/src/hfa_g2p.rs:191`
- Issue: `parse_mora_groups` uses the current Rust toolchain's
  `str::to_lowercase()`, while the compatibility runtime is Python 3.12.13 with
  Unicode 15.0. Rust 1.95 has newer Unicode casing tables. For code points added
  casing after Unicode 15, legacy `t.lower()` leaves the unknown token unchanged
  but Rust changes the mora word. The public contract accepts UTF-8 text and
  preserves unknown-token casing behavior; it does not restrict input to ASCII
  HubertFA vocabulary.
- Evidence: an exhaustive scalar-table differential found 1,433 Python lowercase
  mappings, 1,488 Rust mappings, and 55 mismatches, all newer Rust mappings absent
  from Python 3.12. A real public-API differential reproduced four families:
  U+1C89 -> U+1C8A, U+A7CB -> U+0264, U+10D50 -> U+10D70, and U+16EA0 -> U+16EBB
  in Rust `HfaG2pOutput.words`; Python retains each original code point. For
  U+1C89 with language `ja`, both sides return phonemes
  `["SP", "ja/\u{1c89}", "SP"]` and indexes `[-1, 0, -1]`, but Python returns
  words `["\u{1c89}"]` while Rust returns `["\u{1c8a}"]`. This follows the
  observable fallback at legacy `inference/HubertFA/tools/g2p.py:103` and line
  122 versus Rust lines 191 and 212. None of the 25 shared fixtures reaches a
  Unicode-version-differing code point.
- Required fix: use a Python 3.12/Unicode 15-compatible lowercase implementation
  for mora word normalization instead of toolchain-dependent casing. Add durable
  public fixtures for representative newer-casing code points, including exact
  phonemes, words, indexes, and language prefixing; ideally pin a full Unicode 15
  lowercase digest so a Rust upgrade cannot silently change the contract. Rerun
  the legacy checker, focused/full Rust tests, and this behavior gate.

No other behavior parity findings were found.

The shared Base contract otherwise matches Python evaluation order and output
shape. Legacy obtains `_g2p`, indexes the first/last phonemes, rejects invalid
boundaries or consecutive `SP`, then prefixes each non-`SP` phone at
`inference/HubertFA/tools/g2p.py:20`. Rust preserves empty-list `IndexError`,
empty-message `AssertionError`, assertion-before-prefix order, nullable/empty/
string language behavior, word/index retention, and ordered arrays at
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_g2p.rs:67`. Fixture lines 20 through
25 cover each error and valid projection exactly.

Raw `PhonemeG2P` behavior matches for the reviewed domain. Both sides apply
Python-compatible outer `strip`, split only on literal ASCII space, remove only
exact uppercase `SP`, retain consecutive-space empty tokens, interleave one `SP`
after every word, and generate indexes in input order
(`inference/HubertFA/tools/g2p.py:37`; Rust `hfa_g2p.rs:102`). Fixture lines 1
through 9 cover empty input, only `SP`, None/empty/non-empty language, repeated
spaces, embedded tab/newline, U+001C strip, and lowercase `sp`. Independent
enumeration confirmed Rust's explicit Python whitespace predicate matches all 29
Python 3.12 `str.isspace()` scalar values, including U+001C through U+001F.

Japanese mora control flow and index construction match apart from the finding.
The Rust order mirrors legacy `SP`/`AP`/`EP` filtering, `N`/`cl`, normalized
vowels, joined spellings, separated consonant/vowel legacy shape, multi-vowel
and palatal fallbacks, original-phone versus lowercase-word behavior, one
trailing `SP` per group, and monotonically assigned word indexes
(`inference/HubertFA/tools/g2p.py:82`; Rust `hfa_g2p.rs:134`). Fixture lines 10
through 19 and an independent combined public probe cover those branches.
Multi-character Unicode lower checks, including Greek final sigma, also matched;
the mismatch is specifically the Unicode table version, not contextual casing.
Repeated calls are stable in both the legacy checker and the focused Rust test.

The 25-case gate is genuinely cross-runtime: the checker imports the real legacy
classes and executes every fixture at
`rewrite-in-rust/bootstrap/check_hfa_phoneme_mora_g2p_core.py:50`, while Rust
embeds the same JSONL and compares exact JSON at
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_g2p.rs:386` and line 432. Its
passing result establishes the covered ASCII/control cases, but cannot override
the independently reproduced public input mismatch above.

Scope and rollback are intact. Rust adds only the independent `hfa_g2p` library
module; static search found no DictionaryG2P, file, warning, YAML, export, model,
or production caller implementation in it. Production `infer_base.py` still
imports and selects Python `PhonemeG2P`/`JapanesePhonemeMoraG2P` at lines 13 and
155 through 177, and `hfa_api.py` still routes through that Python owner at lines
127 through 134. No production `v2m_core::hfa_g2p` call exists. The manifest
remains `status: reimplemented` and `current_owner: legacy` at
`rewrite-in-rust/manifest.yaml:1411`, so rollback is leaving the three Python
classes as runtime owners.

Writer/reviewer separation is preserved. This review did not modify production
code, tests, fixtures, checker, dependency/bootstrap artifacts, records, or the
manifest. It adds only this report; independent probes were isolated under
`/tmp`.

## Checks

- `UV_CACHE_DIR=/tmp/uv-cache uv run python rewrite-in-rust/bootstrap/check_hfa_phoneme_mora_g2p_core.py`: passed; validated all 25 real legacy fixtures and the repeated-call check under Python 3.12.13.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_phoneme_mora_g2p_core -- --nocapture`: passed 3 focused tests: exact shared table, 10,000-token scale, and repeated calls.
- Python/Rust Unicode scalar lowercase audit: failed parity; Python 3.12/Unicode 15 had 1,433 changed scalars, Rust 1.95 had 1,488, with 55 Rust-only mappings.
- Independent real public Unicode differential: failed parity for U+1C89, U+A7CB, U+10D50, and U+16EA0; exact phonemes/indexes matched but mora words differed as described above.
- Python/Rust whitespace scalar audit: passed; both predicates selected the same 29 code points, including U+001C through U+001F.
- Independent 12-case public string differential: passed byte for byte with SHA-256 `505b351c306067503565d86854b7fc50f957d79dc6e667ac93477eeec28eeb23`; covered Greek contextual lowercase, U+001C..U+001F strip, literal spaces, empty language prefix, controls, vowels/specials, joined/fallback forms, unknown casing, order, and indexes.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml`: passed 108 `v2m-core` tests, 5 bridge tests, and doc tests.
- `cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check`: passed.
- `cargo clippy --manifest-path rewrite-in-rust/rust/Cargo.toml --all-targets --all-features -- -D warnings`: passed.
- `git diff --check`: passed before this report was written.
- Static scope/routing search: inspected; only the independent module/tests call the Rust public functions, and production callers remain on legacy Python.

## Residual Risk

After pinning lowercase behavior, Rust UTF-8 strings still cannot represent lone
Python surrogate code points; the documented seam already describes UTF-8 input,
so those are outside the current contract. The fixture table remains selective
rather than exhaustive over every mora onset/join pair, although source-table
inspection and the independent combined probe found no additional branch or
ordering difference. Dictionary warnings/files, dataset/lab IO, arbitrary future
bridge payload validation, Python-facing error mapping, model execution, config,
export, caller routing, and owner-switch rollback remain outside this unit.

## Promotion Note

This behavior finding blocks coordinator verification of
`hfa_phoneme_mora_g2p_core`. Keep the manifest `reimplemented` and Python as
runtime owner until Python 3.12-compatible lowercase behavior is implemented,
durably fixture-tested, and this behavior role passes on rerun. Separately
required dependency/bootstrap and data/algorithm reviews do not waive this
failure. This report does not update the manifest or approve production routing.
