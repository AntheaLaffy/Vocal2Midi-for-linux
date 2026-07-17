# lyric_language_processor_contract - data_algorithm_reviewer

Date: 2026-07-17
Decision: fail

## Findings

- Severity: medium
- Location: rewrite-in-rust/rust/crates/v2m-core/src/lyric_language.rs:195
- Issue: Chinese `clean_text` does not preserve the legacy base regex behavior for bracket-adjacent text. Python `LanguageProcessor.clean_text` applies `re.sub(rf'[^{self._allowed_chars}]', '', text)` before whitespace compaction at `inference/LyricFA/tools/language_processors.py:15`, and `ChineseProcessor` passes `r'[\u4e00-\u9fa5]'` as the allowed range at `inference/LyricFA/tools/language_processors.py:29`. That exact regex has observable behavior beyond whitespace normalization: for example, legacy Python returns `ab中` for `abc]中`, `中` for `1]中`, and `[Vers中` for `[Verse]中`. Rust currently routes Chinese cleaning directly to `collapse_whitespace` at `rewrite-in-rust/rust/crates/v2m-core/src/lyric_language.rs:195`, so the same `abc]中` input remains `abc]中`. The fixture table only covers ordinary Chinese mixed text and empty input at `rewrite-in-rust/fixtures/lyric_language_processor_contract.jsonl:4` and `rewrite-in-rust/fixtures/lyric_language_processor_contract.jsonl:5`, so this mismatch is not caught.
- Evidence: `uv run python - <<'PY' ... ChineseProcessor().clean_text(...) ... PY` showed `abc]中 -> ab中`, `1]中 -> 中`, and `[Verse]中 -> [Vers中`. A temporary `/tmp` Rust probe linked against `v2m_core` showed `ChineseProcessor::clean_text("abc]中") -> abc]中`. The manifest requires preserving base clean-text regex behavior at `rewrite-in-rust/manifest.yaml:1281`, and the bootstrap scope includes base regex filtering plus whitespace compaction at `rewrite-in-rust/bootstrap/lyric_language_processor_contract.md:15`.
- Required fix: Either implement the exact legacy base-regex behavior for Chinese cleaning, including the bracket edge cases, or explicitly re-scope the unit with a migration record and fixtures that document a deliberate behavior change. Add fixture rows for cases such as `abc]中`, `1]中`, and `[Verse]中`, then rerun the Python checker and `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml lyric_language`.

## Scope Notes

Processor selection and ordering match the stated contract: Rust lowercases factory input and returns `zh`, `en`, `ja` in order at `rewrite-in-rust/rust/crates/v2m-core/src/lyric_language.rs:71` and `rewrite-in-rust/rust/crates/v2m-core/src/lyric_language.rs:82`, matching Python at `inference/LyricFA/tools/language_processors.py:119` and `inference/LyricFA/tools/language_processors.py:126`.

Chinese and Japanese algorithm ownership mostly stays in the verified helper seams. Chinese split and phonetic conversion delegate to `zh_g2p::split_string` and `ZhG2p::convert_list` at `rewrite-in-rust/rust/crates/v2m-core/src/lyric_language.rs:199` and `rewrite-in-rust/rust/crates/v2m-core/src/lyric_language.rs:203`. Japanese split, phonetic conversion, and reference lyric construction delegate to `JaG2p` at `rewrite-in-rust/rust/crates/v2m-core/src/lyric_language.rs:264`, `rewrite-in-rust/rust/crates/v2m-core/src/lyric_language.rs:272`, and `rewrite-in-rust/rust/crates/v2m-core/src/lyric_language.rs:283`. This matches the bootstrap exclusion of Chinese G2P internals and Japanese fallback internals at `rewrite-in-rust/bootstrap/lyric_language_processor_contract.md:25`.

Empty input behavior is covered for Chinese and Japanese. The Chinese `[""]` phonetic shape is recorded in `rewrite-in-rust/records/0063-confirm-lyric-language-processor-boundary.md:80`, represented in the fixture at `rewrite-in-rust/fixtures/lyric_language_processor_contract.jsonl:5`, and preserved by Rust through `split_on_ascii_space_preserving_empty` at `rewrite-in-rust/rust/crates/v2m-core/src/lyric_language.rs:203` and `rewrite-in-rust/rust/crates/v2m-core/src/lyric_language.rs:326`.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_lyric_language_processor_contract.py`: passed.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml lyric_language`: passed; 1 `lyric_language` test passed, 0 failed.
- `git diff --check`: passed.
- `uv run python - <<'PY' ... ChineseProcessor().clean_text(...) ... PY`: exposed the Chinese bracket-regex mismatch described above.
- Temporary `/tmp` Rust probe linked against `v2m_core`: confirmed Rust currently keeps `abc]中` unchanged.

## Residual Risk

This review did not re-review the full Chinese and Japanese G2P helper algorithms; it relies on the closed helper gates in `rewrite-in-rust/records/0058-close-zh-g2p-dictionary-gate.md` and `rewrite-in-rust/records/0060-close-ja-g2p-fallback-gate.md`. No Python/Rust bridge, production dictionary asset payload, OpenJTalk frontend path, or runtime routing is proven by this unit.

## Promotion Note

This data/algorithm role blocks coordinator update for `lyric_language_processor_contract`. Do not mark the manifest unit verified from this role until the Chinese clean-text regex mismatch is fixed or explicitly re-scoped and re-reviewed.
