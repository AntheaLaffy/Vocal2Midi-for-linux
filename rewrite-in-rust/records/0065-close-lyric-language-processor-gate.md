# 0065 - Close Lyric Language Processor Gate

Date: 2026-07-17

## Context

`lyric_language_processor_contract` was confirmed by record 0063 and
implemented by record 0064 as a fixture-bound Rust library unit for the
deterministic processor behavior in
`inference/LyricFA/tools/language_processors.py`.

The unit composes the already verified `zh_g2p_dictionary_core` and
`ja_g2p_fallback_core` helper seams. Bundled Chinese dictionary file IO,
OpenJTalk frontend execution, lyric matching file/state behavior, model
execution, GUI/Web/CLI routing, and production bridge wiring remain
legacy-owned.

The current post-fix review evidence is:

- dependency/bootstrap review:
  `rewrite-in-rust/reviews/2026-07-17-lyric_language_processor_contract-dependency_bootstrap_reviewer-rerun2.md`
- behavior review:
  `rewrite-in-rust/reviews/2026-07-17-lyric_language_processor_contract-behavior_reviewer-rerun2.md`
- data/algorithm review:
  `rewrite-in-rust/reviews/2026-07-17-lyric_language_processor_contract-data_algorithm_reviewer-rerun2.md`

All three reviews returned `pass` with no findings. They confirmed that the
earlier Chinese bracket-regex and Python `re` whitespace findings are closed,
the Japanese reference `LyricData` path is fixture-backed, and no production
Rust routing or dependency expansion was introduced.

## Decision

Accept `lyric_language_processor_contract` as verified for the current
legacy-owned, no-bridge Rust library seam.

The verified unit preserves:

- supported-language ordering and case-insensitive processor selection;
- exact unsupported-language message text;
- Python `re` whitespace behavior, including U+001C through U+001F;
- English filtering, lowercase splitting, and identity phonetics;
- legacy Chinese bracket-adjacent regex cleaning;
- Chinese splitting and phonetics over injected Mandarin maps;
- the legacy Chinese empty-input phonetic shape;
- Japanese fallback cleaning, mora splitting, and romaji conversion;
- Japanese reference lyric construction and `LyricData` output shape.

## Verification

```bash
uv run python rewrite-in-rust/bootstrap/check_lyric_language_processor_contract.py
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml lyric_language
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml
cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check
cargo clippy --manifest-path rewrite-in-rust/rust/Cargo.toml --all-targets --all-features -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --manifest-path rewrite-in-rust/rust/Cargo.toml --no-deps
uv run pytest tests/test_web_api.py
git diff --check
```

The data/algorithm reviewer also compared Python `re` and the Rust predicate
over every Unicode scalar value. Both selected the same 29 whitespace code
points for the current project toolchains.

## Residual Risk

This verification remains fixture-bound. Production promotion must still
define Chinese dictionary asset packaging and malformed-payload validation,
OpenJTalk ownership or explicit exclusion, Python-facing error and logging
behavior, and rollback. A Python, Rust, or Unicode database upgrade should
rerun the exhaustive whitespace membership probe.

## Reversal

Rollback remains keeping `ProcessorFactory`, `LanguageProcessor`, and
`LyricData` in `inference.LyricFA.tools.language_processors` as runtime owners.
No production bridge was introduced.
