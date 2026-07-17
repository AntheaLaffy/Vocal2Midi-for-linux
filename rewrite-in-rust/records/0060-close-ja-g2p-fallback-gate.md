# 0060 - Close Ja G2P Fallback Gate

Date: 2026-07-17

## Context

`ja_g2p_fallback_core` was confirmed by record 0059 as a fixture-bound unit for
deterministic Japanese G2P fallback behavior from
`inference/LyricFA/tools/JaG2p.py` when `pyopenjtalk` is absent.

The unit keeps `pyopenjtalk.run_frontend`, OpenJTalk dictionary/model ownership,
language processor orchestration, `LyricMatcher` and `lfa_api` orchestration,
ASR phoneme post-processing, model execution, GUI/Web/CLI routing, and
production bridge wiring legacy-owned.

The unit now has:

- dependency/bootstrap review:
  `rewrite-in-rust/reviews/2026-07-17-ja_g2p_fallback_core-dependency_bootstrap_reviewer.md`
- behavior review:
  `rewrite-in-rust/reviews/2026-07-17-ja_g2p_fallback_core-behavior_reviewer.md`
- data/algorithm review:
  `rewrite-in-rust/reviews/2026-07-17-ja_g2p_fallback_core-data_algorithm_reviewer.md`

All three reviews returned `pass` with no findings. The reviewers confirmed the
Linux fallback boundary, the forced `pyopenjtalk = None` checker, the
hand-written table replacement, and the absence of production Rust routing.

## Decision

Accept `ja_g2p_fallback_core` as verified for the current no-bridge,
pyopenjtalk-absent Rust library seam.

The verified Rust unit preserves:

- ASCII letter and special-letter classification;
- Python `str.isdigit()` digit membership plus `〇` numeric-like handling;
- kanji, kana, small-kana, long-mark, and Japanese symbol classification;
- whitespace normalization;
- mixed Latin, numeric-like, Japanese, and punctuation input splitting;
- Japanese segment splitting for contracted kana and long vowels;
- hiragana/katakana conversion ranges;
- `KATA_TO_ROMAJI` table behavior;
- long-vowel expansion, including `cl` and `n` long-vowel skipping;
- ASCII, full-width, and `〇` number mapping;
- unsupported Unicode digit fallback;
- Latin lowercase fallback, kanji fallback, and unknown-token fallback;
- `convert`, `convert_list`, `split_string_no_regex`, and
  `split_kana_no_regex` output shape.

## Verification

```bash
uv run python rewrite-in-rust/bootstrap/check_ja_g2p_fallback_core.py
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml ja_g2p
```

Broader checks also passed during coordinator review:

```bash
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml
cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check
cargo clippy --manifest-path rewrite-in-rust/rust/Cargo.toml --all-targets --all-features -- -D warnings
git diff --check
```

Reviewers also ran targeted audits proving the Rust hard-coded digit table
matches the current uv Python `str.isdigit()` membership.

## Residual Risk

The verified proof is fixture-bound and intentionally excludes
`pyopenjtalk.run_frontend` output parity. Runtime promotion must still define
OpenJTalk package/runtime ownership, fallback ordering, payload validation,
logging text, Python-facing error mapping, and rollback. The explicit Unicode
digit table should be rechecked if the project upgrades to a Python or Unicode
version with different `str.isdigit()` membership.

## Reversal

Rollback remains keeping `JaG2p` and `KATA_TO_ROMAJI` in
`inference.LyricFA.tools.JaG2p` as the runtime owners. No production bridge was
introduced.
