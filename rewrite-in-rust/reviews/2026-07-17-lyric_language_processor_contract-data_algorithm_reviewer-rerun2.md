# lyric_language_processor_contract - data_algorithm_reviewer rerun2

Date: 2026-07-17
Decision: pass

Unit: `lyric_language_processor_contract`
Role: `data_algorithm_reviewer`

## Findings

No findings.

The prior Python-regex whitespace blocker is fixed across every clean-text data
path. Legacy base cleaning filters with the processor's regex character class
and compacts Python `re` `\s` at
`inference/LyricFA/tools/language_processors.py:15`; English includes `\s` in
its allowed class at `inference/LyricFA/tools/language_processors.py:43`, and
Japanese independently performs whitespace-only compaction at
`inference/LyricFA/tools/language_processors.py:62`. Rust now defines
`is_python_regex_whitespace` as Rust Unicode whitespace plus U+001C..U+001F at
`rewrite-in-rust/rust/crates/v2m-core/src/lyric_language.rs:346`. English uses
that predicate while filtering at
`rewrite-in-rust/rust/crates/v2m-core/src/lyric_language.rs:219`, while Chinese
and Japanese both reach it through `collapse_whitespace` at
`rewrite-in-rust/rust/crates/v2m-core/src/lyric_language.rs:193`,
`rewrite-in-rust/rust/crates/v2m-core/src/lyric_language.rs:258`, and
`rewrite-in-rust/rust/crates/v2m-core/src/lyric_language.rs:307`.

The fixture gate directly locks all four formerly missing separators in each
processor path: English at
`rewrite-in-rust/fixtures/lyric_language_processor_contract.jsonl:3`, inherited
base/Chinese at
`rewrite-in-rust/fixtures/lyric_language_processor_contract.jsonl:6`, and
Japanese at
`rewrite-in-rust/fixtures/lyric_language_processor_contract.jsonl:11`. The
Python checker executes `clean_text`, split, and phonetic conversion for every
`processor_flow` row at
`rewrite-in-rust/bootstrap/check_lyric_language_processor_contract.py:76` and
iterates the full fixture file at
`rewrite-in-rust/bootstrap/check_lyric_language_processor_contract.py:169`.
The Rust fixture test executes the same processor flow at
`rewrite-in-rust/rust/crates/v2m-core/src/lyric_language.rs:415` and dispatches
every `processor_flow` row through it at
`rewrite-in-rust/rust/crates/v2m-core/src/lyric_language.rs:489`.

The fix does not introduce an observed data or algorithm regression. An
exhaustive probe over all Unicode scalar values produced the same 29-code-point
whitespace set for current project Python `re` and the Rust predicate, including
U+001C..U+001F and excluding no Python whitespace. Targeted probes against the
actual Python and Rust processors matched for each of the four separators in
English, Chinese, and Japanese. The probes also retained the six prior Chinese
bracket-regex expectations (`abc]中`, `1]中`, `[Verse]中`, `[]中`, `ab]]中`, and
`a]b[c^d-e\\f中  文`). The bracket algorithm remains isolated in
`clean_chinese_legacy_regex` at
`rewrite-in-rust/rust/crates/v2m-core/src/lyric_language.rs:324`, and its fixture
remains at `rewrite-in-rust/fixtures/lyric_language_processor_contract.jsonl:7`.

The processor-local algorithms remain linear in input characters. Whitespace
compaction is a single pass, English adds a single filtering pass, and Chinese
uses one character collection plus one forward scan. Chinese and Japanese G2P
internals remain delegated to their already verified helper seams rather than
being duplicated in this unit at
`rewrite-in-rust/rust/crates/v2m-core/src/lyric_language.rs:197` and
`rewrite-in-rust/rust/crates/v2m-core/src/lyric_language.rs:262`. The full
`v2m-core` suite passed all 98 tests, providing broader regression evidence
beyond the targeted fixture.

Writer/reviewer separation is preserved. This report covers exactly
`lyric_language_processor_contract` as `data_algorithm_reviewer`; I reviewed
current state and did not edit production code, fixtures, bootstrap,
dependencies, manifest, or records.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_lyric_language_processor_contract.py`: passed with no output.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml lyric_language`: passed; 1 selected fixture-table test passed and 97 `v2m-core` tests were filtered out.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml -p v2m-core`: passed; 98 unit tests and 0 doc tests failed.
- Exhaustive Python `re` `\s` versus Rust predicate probe over U+0000..U+10FFFF: passed; both returned the same 29 scalar values.
- Targeted actual-processor probes for U+001C..U+001F across English, Chinese, and Japanese: passed on both Python and Rust.
- Targeted Chinese legacy bracket-regex probes for six prior edge cases: passed on both Python and Rust.
- `git diff --check`: passed.

## Residual Risk

This review does not reopen Chinese dictionary conversion or Japanese fallback
tokenizer/table internals; it relies on their closed helper gates. It also does
not prove bundled Chinese dictionary asset loading, OpenJTalk frontend behavior,
production bridge wiring, Python-facing runtime error mapping, or GUI/Web/CLI
routing, all of which remain outside this unit. Unicode whitespace equivalence
is proven against the current project Python and Rust toolchains; a future
Unicode database change should rerun the exhaustive probe.

## Promotion Note

This data/algorithm role no longer blocks coordinator state update. The
coordinator may record this role as passing; whole-unit verification still
depends on the other required review evidence in the manifest.
