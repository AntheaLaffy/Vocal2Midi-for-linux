# zh_g2p_dictionary_core - data_algorithm_reviewer

Date: 2026-07-17
Decision: fail

## Findings

- Severity: medium
- Location: rewrite-in-rust/rust/crates/v2m-core/src/zh_g2p.rs:303
- Issue: Rust token splitting does not preserve Python's Unicode digit classification. Legacy `split_string` and `split_string_no_regex` use `current_char.isdigit()`, which keeps non-ASCII digit characters as tokens. Rust only accepts ASCII digits and fullwidth digits, so `convert(text)` can silently drop other Unicode digit characters before dictionary conversion.
- Evidence: Python uses `current_char.isdigit()` in `inference/LyricFA/tools/ZhG2p.py:59` and `inference/LyricFA/tools/ZhG2p.py:142`; Rust routes both tokenizers through `is_digit_like` at `rewrite-in-rust/rust/crates/v2m-core/src/zh_g2p.rs:228` and `rewrite-in-rust/rust/crates/v2m-core/src/zh_g2p.rs:259`, with the narrower implementation at `rewrite-in-rust/rust/crates/v2m-core/src/zh_g2p.rs:303`. A legacy probe with `uv run python` showed `split_string("٣中") == ["٣", "中"]`, `split_string("²中") == ["²", "中"]`, and `split_string("９中") == ["９", "中"]`. The fixture table only exercises ASCII `3` in splitter/number cases at `rewrite-in-rust/fixtures/zh_g2p_dictionary_core.jsonl:3`, `rewrite-in-rust/fixtures/zh_g2p_dictionary_core.jsonl:4`, `rewrite-in-rust/fixtures/zh_g2p_dictionary_core.jsonl:9`, `rewrite-in-rust/fixtures/zh_g2p_dictionary_core.jsonl:10`, and `rewrite-in-rust/fixtures/zh_g2p_dictionary_core.jsonl:16`.
- Required fix: Match Python `str.isdigit()` for tokenizer classification, or explicitly record and review a narrowed Rust behavior before promotion. Add fixture rows for at least one non-ASCII digit that Python tokenizes but `number_map` does not convert, plus the existing fullwidth-digit behavior.

- Severity: low
- Location: rewrite-in-rust/rust/crates/v2m-core/src/zh_g2p.rs:331
- Issue: Injected dictionary shape errors are tolerated differently from legacy Python. For malformed maps, Rust's reset path and default-pinyin path use optional lookups and can leave original tokens in place; Python's `reset_zh` and `get_default_pinyin` would raise on missing result entries or empty word lists. This is acceptable only while bundled dictionary loading and bridge payload validation remain legacy-owned, but it is not yet proven or specified for a promoted injected-map API.
- Evidence: Python assigns `final_result[position] = result[index]` in `inference/LyricFA/tools/ZhG2p.py:121` and indexes the first word reading in `inference/LyricFA/tools/ZhG2p.py:236`. Rust guards both reset replacement and default reading lookup at `rewrite-in-rust/rust/crates/v2m-core/src/zh_g2p.rs:184` and `rewrite-in-rust/rust/crates/v2m-core/src/zh_g2p.rs:331`. The bootstrap record intentionally leaves dictionary file IO and packaging legacy-owned at `rewrite-in-rust/bootstrap/zh_g2p_dictionary_core.md:29` and promotion payload validation unresolved at `rewrite-in-rust/bootstrap/zh_g2p_dictionary_core.md:120`.
- Required fix: Before any production bridge or promoted injected-map API, define dictionary payload validation for word-list non-emptiness and phrase-reading arity, or add fixtures and a record for intentional non-parity on malformed dictionaries.

No high or critical findings were found. Source inspection shows the valid-dictionary control flow otherwise matches the legacy algorithm: tone mark normalization, special kana pairing, phrase loop order from length 4 to 2, direct/current phrase replacement, previous/current and previous/current/next windows, `remove_elements` clamping behavior, cursor movement, original-position reset for well-formed result vectors, tone stripping, ASCII `number_map`, and the `HashMap<String, Vec<String>>` dictionary shape used by the parity fixtures.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_zh_g2p_dictionary_core.py`: passed
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml zh_g2p`: passed, 1 `zh_g2p` test run
- `cargo clippy --manifest-path rewrite-in-rust/rust/Cargo.toml --all-targets --all-features -- -D warnings`: passed
- `git diff --check`: passed

## Residual Risk

The fixture table has good coverage for the claimed happy-path surface: tone/v normalization, module and no-regex splitting, passthrough reset, traditional-to-simplified lookup, ASCII number conversion, include-tone behavior, default polyphonic fallback, length-4 direct phrase precedence, and the relative phrase windows. It does not yet prove Unicode digit parity, malformed injected dictionaries, or the full collision matrix for every phrase-window length. Large bundled dictionary files also remain legacy-owned; samples show entries outside the tokenizer Hanzi range, which matches legacy source behavior but is not covered by the fixture table.

## Promotion Note

This data/algorithm review blocks promotion of `zh_g2p_dictionary_core` until the Unicode digit tokenizer mismatch is fixed or explicitly re-scoped with a migration record and fixtures. Do not mark the manifest unit verified from this role's evidence.
