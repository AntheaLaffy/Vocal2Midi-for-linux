# zh_g2p_dictionary_core - behavior_reviewer

Date: 2026-07-17
Decision: pass-with-followups

## Findings

- Severity: low
- Location: `inference/LyricFA/tools/ZhG2p.py:59`, `inference/LyricFA/tools/ZhG2p.py:142`, `rewrite-in-rust/rust/crates/v2m-core/src/zh_g2p.rs:303`
- Issue: `split_string` and `split_string_no_regex` do not fully match legacy Python digit tokenization for non-ASCII digit characters. Python keeps any character where `str.isdigit()` is true, including Arabic-Indic digits and superscript digits. Rust keeps only ASCII digits and full-width digits through `is_digit_like`.
- Evidence: `uv run python - <<'PY' ...` returned `['٣', '中']`, `['²', '中']`, and `['３', '中']` for both legacy splitters. A Rust probe against `zh_g2p.rs` returned `["中"]`, `["中"]`, and `["３", "中"]`. The main fixture table covers ASCII digit conversion and full-width splitting is source-compatible, but the broader legacy `isdigit()` surface is not covered.
- Required fix: Before production owner promotion, either align the Rust splitters with Python `str.isdigit()` semantics and add fixtures for non-ASCII digit tokens, or explicitly narrow the accepted public boundary to ASCII/full-width digit tokenization in the manifest/bootstrap records.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_zh_g2p_dictionary_core.py`: pass.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml zh_g2p`: pass; `zh_g2p::tests::zh_g2p_dictionary_follows_parity_fixture_table` passed.
- `git diff --check`: pass.
- Targeted legacy probe for digit splitting: confirmed Python preserves Arabic-Indic `٣`, superscript `²`, and full-width `３` as digit tokens.
- Targeted Rust probe for digit splitting: confirmed Rust preserves full-width `３` but drops Arabic-Indic `٣` and superscript `²`.

## Reviewed Parity Surface

- `tone_to_normal` matches the legacy tone-mark table and `v_to_u` post-replacement behavior for the fixture cases (`lǜèḿǹüABC` with both `v_to_u` states).
- `split_string` and `split_string_no_regex` match the fixture-covered ASCII letters, apostrophe/hyphen handling, Hanzi, kana pair grouping, punctuation skipping, ASCII digit handling, and no-regex exclusion of apostrophe/hyphen from letter runs.
- `zh_position`, traditional-to-simplified lookup, ASCII number conversion/non-conversion, `include_tone` stripping, tone mark normalization, unknown/Latin/kana/punctuation passthrough through `reset_zh`, polyphonic fallback, direct length-4 phrase precedence, previous-current replacement, previous-current-with-next replacement, previous-current-next replacement, `convert(text)` no-regex splitting, output string order, and fixture-backed injected dictionary behavior all match the legacy fixture harness.
- Rollback remains intact: the manifest keeps `current_owner: legacy`, the rollback text keeps `inference.LyricFA.tools.ZhG2p` as runtime owner, and production callers in `language_processors.py` and `lfa_api.py` still import the Python `ZhG2p`.
- No production bridge was introduced. The Rust module is exported only inside the independent Rust workspace crate; no Python caller imports or routes through it.

## Residual Risk

The fixture harness uses small injected dictionaries, not the full bundled Mandarin/Cantonese dictionaries. It does not prove malformed dictionary shape behavior, empty pinyin entries, arbitrary multi-character `convert_list` tokens, or exact Python exception behavior for inconsistent phrase lengths. Dictionary file IO, dictionary packaging, payload validation, logging text, and Python-facing error mapping are intentionally still legacy-owned and unproven for runtime promotion.

## Promotion Note

This behavior role does not find a fixture-covered parity blocker, and rollback/no-bridge constraints are satisfied. The unit can proceed only as `pass-with-followups`: runtime owner promotion should wait until the non-ASCII digit-tokenization mismatch is either fixed with fixtures or explicitly accepted as out of boundary by a coordinator record.
