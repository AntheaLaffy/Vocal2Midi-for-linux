# 0058 - Close Zh G2P Dictionary Gate

Date: 2026-07-17

## Context

`zh_g2p_dictionary_core` was confirmed by record 0057 as a fixture-bound unit
for deterministic Chinese lyric G2P dictionary behavior from
`inference/LyricFA/tools/ZhG2p.py`.

The unit keeps full dictionary file IO and packaging, language processor
orchestration, `LyricMatcher` and `lfa_api` orchestration, Japanese G2P, model
execution, GUI/Web/CLI routing, and production bridge wiring legacy-owned.

The unit now has:

- dependency/bootstrap review:
  `rewrite-in-rust/reviews/2026-07-17-zh_g2p_dictionary_core-dependency_bootstrap_reviewer.md`
- behavior review:
  `rewrite-in-rust/reviews/2026-07-17-zh_g2p_dictionary_core-behavior_reviewer.md`
- behavior follow-up review:
  `rewrite-in-rust/reviews/2026-07-17-zh_g2p_dictionary_core-behavior_reviewer-rerun.md`
- data/algorithm review:
  `rewrite-in-rust/reviews/2026-07-17-zh_g2p_dictionary_core-data_algorithm_reviewer.md`
- data/algorithm follow-up review:
  `rewrite-in-rust/reviews/2026-07-17-zh_g2p_dictionary_core-data_algorithm_reviewer-rerun.md`

The initial behavior review returned `pass-with-followups`, and the initial
data/algorithm review returned `fail`, both on the same issue: Rust digit
tokenization only preserved ASCII and full-width digits, while legacy Python
uses `str.isdigit()` in both splitters.

The coordinator fixed the blocker by expanding Rust `is_digit_like` to match
Python 3.12 `str.isdigit()` Unicode membership and adding shared fixtures for
Arabic-Indic, superscript, circled, full-width, and mathematical digit
tokenization plus non-digit numeric character skipping. The behavior rerun
returned `pass`. The data/algorithm rerun returned `pass-with-followups` with
only a low promotion-time requirement for malformed injected dictionary payload
validation.

## Decision

Accept `zh_g2p_dictionary_core` as verified for the current no-bridge,
injected-map Rust library seam.

The verified Rust unit preserves:

- tone mark normalization and optional `v` to `ü` restoration;
- module-level `split_string` grouping for ASCII letters, apostrophe/hyphen
  variants, Hanzi, Python `str.isdigit()` digit tokens, and kana plus small
  kana pairs;
- `split_string_no_regex` behavior for ASCII letter runs, Hanzi, Python
  `str.isdigit()` digit tokens, and kana plus small kana pairs;
- traditional-to-simplified dictionary lookup;
- default pinyin selection for non-polyphonic words;
- polyphonic phrase matching order from length 4 down to 2;
- direct current phrase, previous-current-with-next, previous-current, and
  previous-current-next replacement windows;
- ASCII-only number conversion through the legacy Chinese numeral map;
- `include_tone` stripping of trailing ASCII tone digits before tone-mark
  normalization;
- original-position reset and unknown token fallback for valid dictionary data.

## Verification

```bash
uv run python rewrite-in-rust/bootstrap/check_zh_g2p_dictionary_core.py
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml zh_g2p
```

Broader checks also passed during coordinator review:

```bash
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml
cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check
cargo clippy --manifest-path rewrite-in-rust/rust/Cargo.toml --all-targets --all-features -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --manifest-path rewrite-in-rust/rust/Cargo.toml --no-deps
git diff --check
```

## Residual Risk

The verified proof is fixture-bound and uses injected dictionaries instead of
the full bundled Mandarin and Cantonese dictionary files. Runtime promotion
must still define dictionary asset packaging, payload validation for non-empty
word readings and phrase-reading arity, Python-facing error mapping, and
logging text. The explicit Unicode digit table matches the project uv Python
version observed in review; it should be rechecked if the project upgrades to a
Python or Unicode version with different `str.isdigit()` membership.

## Reversal

Rollback remains keeping `inference.LyricFA.tools.ZhG2p` as the runtime owner.
No production bridge was introduced.
