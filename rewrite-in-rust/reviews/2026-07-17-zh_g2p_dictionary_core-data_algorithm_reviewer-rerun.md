# zh_g2p_dictionary_core - data_algorithm_reviewer rerun

Date: 2026-07-17
Decision: pass-with-followups

## Findings

- Severity: low
- Location: rewrite-in-rust/rust/crates/v2m-core/src/zh_g2p.rs:184
- Issue: Injected dictionary shape errors remain intentionally softer than legacy Python for malformed maps. For valid dictionary rows and fixture-backed inputs this does not affect parity, but Rust uses optional lookups for default pinyin and reset replacement while Python indexes directly and would raise for empty or arity-broken dictionary data.
- Evidence: Python assigns `final_result[position] = result[index]` in `inference/LyricFA/tools/ZhG2p.py:121` and indexes the first word reading in `inference/LyricFA/tools/ZhG2p.py:236`. Rust uses `values.first()` at `rewrite-in-rust/rust/crates/v2m-core/src/zh_g2p.rs:189` and guards reset replacement at `rewrite-in-rust/rust/crates/v2m-core/src/zh_g2p.rs:420`. The bootstrap record keeps dictionary file IO and production payload validation legacy-owned at `rewrite-in-rust/bootstrap/zh_g2p_dictionary_core.md:29` and `rewrite-in-rust/bootstrap/zh_g2p_dictionary_core.md:120`.
- Required fix: Before any production bridge or promoted injected-map API, define dictionary payload validation for non-empty word readings and phrase-reading arity, or record the intended non-parity for malformed injected dictionaries.

No medium, high, or critical findings were found in this rerun. The previous Python `str.isdigit()` tokenizer parity blocker is resolved in the current workspace: `is_digit_like` now covers the same 808 Unicode scalar values as Python 3.12 `str.isdigit()`, and fixtures cover Arabic-Indic, superscript, circled, full-width, and mathematical digit tokenization plus `convert(text)` preservation of non-ASCII digit tokens. Source inspection also found the valid-dictionary algorithm still matches the legacy behavior for special kana pairing, ASCII/special-letter grouping, traditional-to-simplified lookup, default pinyin selection, phrase loop order from length 4 to 2, direct/current phrase matching, previous-current-with-next and previous-current-next replacement, `remove_elements` clamping, cursor movement, original-position reset, tone stripping, ASCII-only `number_map`, and injected `HashMap<String, Vec<String>>` dictionary shape.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_zh_g2p_dictionary_core.py`: passed
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml zh_g2p`: passed; 1 `zh_g2p` fixture-table test passed, 0 failed
- `cargo clippy --manifest-path rewrite-in-rust/rust/Cargo.toml --all-targets --all-features -- -D warnings`: passed
- `git diff --check`: passed
- Targeted Unicode digit audit with `uv run python`: passed; parsed the Rust `is_digit_like` range table and compared it against Python 3.12 `chr(cp).isdigit()` for all Unicode scalar values, with `py_only_count 0` and `rust_only_count 0`
- Targeted legacy splitter probe with `uv run python`: passed; `split_string("A٣²¼Ⅷ⑦３𝟘。")` and `ZhG2p.split_string_no_regex(...)` both returned `["A", "٣", "²", "⑦", "３", "𝟘"]`

## Residual Risk

The explicit Unicode digit range table is correct for the project uv Python version observed in this review, but it can drift if a future Python/Unicode version changes `str.isdigit()` membership. The fixture table is strong for the selected deterministic surface, but it does not exhaust every phrase-window collision length or malformed dictionary payload shape. Full bundled dictionary loading and production payload validation remain legacy-owned.

## Promotion Note

This data/algorithm rerun does not block coordinator verification for the current no-bridge, injected-map library seam. Do not mark the manifest verified from this report alone; the coordinator owns manifest state updates after the required reviews are complete.
