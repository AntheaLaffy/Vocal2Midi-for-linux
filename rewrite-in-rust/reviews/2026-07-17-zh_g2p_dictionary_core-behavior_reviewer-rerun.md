# zh_g2p_dictionary_core - behavior_reviewer rerun

Date: 2026-07-17
Decision: pass

## Findings

No findings.

The previous behavior finding is closed. Legacy Python keeps characters where `str.isdigit()` is true in both splitters at `inference/LyricFA/tools/ZhG2p.py:59` and `inference/LyricFA/tools/ZhG2p.py:142`. The Rust splitters still route digit classification through `is_digit_like` at `rewrite-in-rust/rust/crates/v2m-core/src/zh_g2p.rs:228` and `rewrite-in-rust/rust/crates/v2m-core/src/zh_g2p.rs:259`, and the updated implementation covers the Unicode digit ranges at `rewrite-in-rust/rust/crates/v2m-core/src/zh_g2p.rs:303`. The fixture table now exercises Arabic-Indic, superscript, circled, full-width, mathematical digits, and skipped non-digit numeric characters at `rewrite-in-rust/fixtures/zh_g2p_dictionary_core.jsonl:5`, `rewrite-in-rust/fixtures/zh_g2p_dictionary_core.jsonl:6`, and `rewrite-in-rust/fixtures/zh_g2p_dictionary_core.jsonl:13`.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_zh_g2p_dictionary_core.py`: pass.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml zh_g2p`: pass; `zh_g2p::tests::zh_g2p_dictionary_follows_parity_fixture_table` passed.
- `git diff --check`: pass.
- `cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check`: pass.
- `cargo clippy --manifest-path rewrite-in-rust/rust/Cargo.toml --all-targets --all-features -- -D warnings`: pass.
- Exhaustive single-Unicode-scalar public splitter probe: pass. I compared legacy Python `split_string` and `ZhG2p.split_string_no_regex` against Rust `split_string` and `split_string_no_regex` for all valid Unicode scalar values; the diff was empty. The ad hoc `rustc` probe emitted dead-code warnings because it included `zh_g2p.rs` only to call the public splitter functions.

## Reviewed Parity Surface

- `tone_to_normal` still matches the legacy tone-mark table and `v_to_u` post-replacement behavior covered by the fixture table.
- `split_string` and `split_string_no_regex` match the fixture-covered ASCII letters, special-letter handling, Hanzi, kana pair grouping, punctuation skipping, ASCII digits, and the newly covered Python `str.isdigit()` Unicode digit behavior.
- `zh_position`, traditional-to-simplified lookup, ASCII number conversion/non-conversion, `include_tone` stripping, tone mark normalization, unknown/Latin/kana/punctuation passthrough through `reset_zh`, polyphonic fallback, direct length-4 phrase precedence, previous-current replacement, previous-current-with-next replacement, previous-current-next replacement, `convert(text)` no-regex splitting, output string order, and injected dictionary behavior all match the current Python/Rust fixture harness.
- Rollback remains intact: the manifest keeps `current_owner: legacy`, the rollback text keeps `inference.LyricFA.tools.ZhG2p` as runtime owner, and production callers in `language_processors.py` and `lfa_api.py` still import the Python `ZhG2p`.
- No production bridge was introduced. The Rust module is exported only inside the independent Rust workspace crate; no Python caller imports or routes through it.

## Residual Risk

The behavior fixture harness still uses small injected dictionaries rather than the full bundled Mandarin/Cantonese dictionary files. It does not prove malformed dictionary payload behavior, empty pinyin entries, arbitrary inconsistent phrase-length data, or Python-facing error mapping for a future bridge. Dictionary file IO, dictionary packaging, payload validation, logging text, and bridge error semantics remain intentionally outside this unit.

## Promotion Note

This behavior role no longer blocks promotion. From behavior parity evidence, `zh_g2p_dictionary_core` is ready for coordinator state update once the other required review roles are also accepted. I did not edit production code or the manifest.
