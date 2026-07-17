# ja_g2p_fallback_core - data_algorithm_reviewer

Date: 2026-07-17
Decision: pass

## Findings

No findings.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_ja_g2p_fallback_core.py`: passed.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml ja_g2p`: passed.
- `cargo clippy --manifest-path rewrite-in-rust/rust/Cargo.toml --all-targets --all-features -- -D warnings`: passed.
- `git diff --check`: passed.
- `uv run python - <<'PY' ...`: passed targeted audit that `rewrite-in-rust/rust/crates/v2m-core/src/ja_g2p.rs:244` matches the current uv Python `str.isdigit()` codepoint set exactly, including full-width digits already accepted by the legacy helper at `inference/LyricFA/tools/JaG2p.py:18`.

## Data And Algorithm Review

The implementation stays within the accepted fallback boundary from `rewrite-in-rust/records/0059-confirm-ja-g2p-fallback-boundary.md:38`: no OpenJTalk ownership, no Python/Rust bridge, and no production caller wiring. The public Rust module is exposed from `rewrite-in-rust/rust/crates/v2m-core/src/lib.rs:17`, but the runtime owner remains legacy Python per the manifest entry at `rewrite-in-rust/manifest.yaml:1215`.

Token grouping matches the Python tokenizer shape. Rust `split_input_string_no_regex` groups ASCII/special-letter runs, numeric-like runs, and Japanese-character runs at `rewrite-in-rust/rust/crates/v2m-core/src/ja_g2p.rs:175`; this mirrors `JaG2p.split_input_string_no_regex` at `inference/LyricFA/tools/JaG2p.py:127`. The Rust implementation collects `chars` before slicing, so mixed-width Unicode inputs avoid invalid UTF-8 slicing.

The recursive fallback path is structurally equivalent and bounded by token shrinkage. Python splits Japanese segments before recursively analyzing each smaller token with `convert_number=False` at `inference/LyricFA/tools/JaG2p.py:223`; Rust follows the same flow at `rewrite-in-rust/rust/crates/v2m-core/src/ja_g2p.rs:117`. For kana and long-vowel tokens, recursion terminates because one-token segments skip the recursive branch and then parse directly or fall back.

`KATA_TO_ROMAJI`, long-vowel expansion, and `cl`/`n` long-vowel skipping match the Python table and mora algorithm. The Rust table at `rewrite-in-rust/rust/crates/v2m-core/src/ja_g2p.rs:502` covers the same entries as `inference/LyricFA/tools/JaG2p.py:48`, and the mora parser at `rewrite-in-rust/rust/crates/v2m-core/src/ja_g2p.rs:433` mirrors the Python two-character lookup, one-character lookup, long mark handling, and alphabetic fallback at `inference/LyricFA/tools/JaG2p.py:257`.

The hiragana/katakana conversion ranges match Python exactly: Rust uses `0x30A1..=0x30F6` and `0x3041..=0x3096` at `rewrite-in-rust/rust/crates/v2m-core/src/ja_g2p.rs:393` and `rewrite-in-rust/rust/crates/v2m-core/src/ja_g2p.rs:406`, matching `inference/LyricFA/tools/JaG2p.py:100` and `inference/LyricFA/tools/JaG2p.py:111`.

Numeric behavior matches the accepted boundary. Rust uses the same `number_map` coverage at `rewrite-in-rust/rust/crates/v2m-core/src/ja_g2p.rs:475` as Python at `inference/LyricFA/tools/JaG2p.py:89`, preserves unsupported Unicode digit grouping through `is_numeric_like`, and only converts tokens whose every character is in `number_map`, matching `inference/LyricFA/tools/JaG2p.py:243`.

Fixture strength is appropriate for this deterministic unit. `rewrite-in-rust/fixtures/ja_g2p_fallback_core.jsonl:1` covers helper classification, unsupported Unicode digit grouping, mixed token splitting, contracted kana, long vowels, uncommon kana fallback, numeric conversion and non-conversion, kanji fallback, empty punctuation skipping, romaji output, and kana output. The Rust test consumes the same fixture table through `include_str!` at `rewrite-in-rust/rust/crates/v2m-core/src/ja_g2p.rs:639`, so Python/Rust fixture drift is constrained to the shared expected JSONL table.

## Residual Risk

The Rust `is_digit` table is hard-coded at `rewrite-in-rust/rust/crates/v2m-core/src/ja_g2p.rs:244`. It matches the current uv Python environment, but future Python or Unicode database upgrades can change `str.isdigit()` membership. Keep the targeted digit-table audit or equivalent fixture-generation check before promotion if the Python version changes.

The fixture suite is strong for the accepted pyopenjtalk-absent fallback path, but it is still fixture-bound. It does not prove OpenJTalk frontend parity, frontend payload/error handling, or language processor orchestration, all of which are explicitly outside this unit.

## Promotion Note

This data/algorithm review does not block promotion of `ja_g2p_fallback_core` within the current no-bridge, pyopenjtalk-absent fallback boundary. This report does not mark the manifest verified.
