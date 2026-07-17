# lyric_language_processor_contract - data_algorithm_reviewer rerun

Date: 2026-07-17
Decision: fail

Unit: `lyric_language_processor_contract`
Role: `data_algorithm_reviewer`

## Findings

- Severity: medium
- Location: rewrite-in-rust/rust/crates/v2m-core/src/lyric_language.rs:307
- Issue: Rust whitespace handling does not preserve Python regex `\s` semantics for `clean_text`. Python base cleaning keeps regex whitespace during filtering and then compacts `re.sub(r'\s+', ' ', cleaned).strip()` at `inference/LyricFA/tools/language_processors.py:15`, while the Japanese override also compacts `re.sub(r'\s+', ' ', text).strip()` at `inference/LyricFA/tools/language_processors.py:62`. Rust uses `char::is_whitespace()` in `collapse_whitespace` at `rewrite-in-rust/rust/crates/v2m-core/src/lyric_language.rs:307` and in English filtering at `rewrite-in-rust/rust/crates/v2m-core/src/lyric_language.rs:219`. That differs from Python regex `\s` for U+001C, U+001D, U+001E, and U+001F. The result is observable parity drift: Python `ChineseProcessor.clean_text("中\x1d文")` returns `"中 文"`, but Rust keeps `"中\u{1d}文"`; Python `JapaneseProcessor.clean_text("き\x1fょう")` returns `"き ょう"`, but Rust keeps `"き\u{1f}ょう"`; Python `EnglishProcessor.clean_text("A\x1cB")` returns `"A B"`, but Rust returns `"AB"` because the separator is dropped rather than treated as whitespace. The manifest requires preserving base clean-text regex behavior at `rewrite-in-rust/manifest.yaml:1281`, and the bootstrap includes regex filtering plus whitespace compaction in this unit at `rewrite-in-rust/bootstrap/lyric_language_processor_contract.md:15`.
- Evidence: `uv run python - <<'PY' ... re.match(r'\s', chr(cp)) ... PY` showed Python regex whitespace includes 29 code points up to U+3000, including U+001C..U+001F. A Rust stdin-fed probe over `char::is_whitespace()` showed 25 code points and omitted U+001C..U+001F. Processor probes then showed the concrete `clean_text` mismatches above. The target fixture/checker passes because `rewrite-in-rust/fixtures/lyric_language_processor_contract.jsonl` does not include those four regex-whitespace separators.
- Required fix: Use a Python-regex-compatible whitespace predicate for this unit's clean/filter paths, including U+001C..U+001F, instead of plain `char::is_whitespace()`. Apply it both to English allowed-whitespace filtering and shared whitespace compaction, then add fixture rows for these separators across base/Chinese, English, and Japanese clean-text paths and rerun the Python checker plus `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml lyric_language`.

## Scope Notes

The previous Chinese bracket-adjacent regex blocker appears fixed. Python `LanguageProcessor.clean_text` still derives the odd Chinese pattern from `_CHINESE_CHAR_RANGE` at `inference/LyricFA/tools/language_processors.py:15` and `inference/LyricFA/tools/language_processors.py:28`. Rust now routes Chinese cleaning through `clean_chinese_legacy_regex` at `rewrite-in-rust/rust/crates/v2m-core/src/lyric_language.rs:193` and implements the legacy local rule at `rewrite-in-rust/rust/crates/v2m-core/src/lyric_language.rs:324`. The fixture locks `"a]b[c^d-e\\f中  文" -> "b[c^d-e\\f中 文"` at `rewrite-in-rust/fixtures/lyric_language_processor_contract.jsonl:5`, and targeted Python/Rust probes matched for `abc]中`, `1]中`, `[Verse]中`, `[]中`, and `ab]]中`.

Processor data ownership otherwise stays inside the intended structures and helper seams. Rust `LyricData` is the expected `text_list`, `phonetic_list`, and `raw_text` triple at `rewrite-in-rust/rust/crates/v2m-core/src/lyric_language.rs:15`, with `process_text` and `process_reference_lyric` preserving cleaned raw text at `rewrite-in-rust/rust/crates/v2m-core/src/lyric_language.rs:143` and `rewrite-in-rust/rust/crates/v2m-core/src/lyric_language.rs:156`. The fixture has direct `lyric_data_shape` coverage at `rewrite-in-rust/fixtures/lyric_language_processor_contract.jsonl:7`; the Python checker branch compares the shape at `rewrite-in-rust/bootstrap/check_lyric_language_processor_contract.py:126`; and the Rust fixture test serializes the actual `LyricData` from `process_text` at `rewrite-in-rust/rust/crates/v2m-core/src/lyric_language.rs:486`.

Chinese and Japanese algorithm ownership remains properly delegated. Chinese splitting and injected-map phonetic conversion call `zh_g2p::split_string` and `ZhG2p::convert_list` at `rewrite-in-rust/rust/crates/v2m-core/src/lyric_language.rs:197` and `rewrite-in-rust/rust/crates/v2m-core/src/lyric_language.rs:201`, matching the verified injected-map seam accepted in `rewrite-in-rust/records/0058-close-zh-g2p-dictionary-gate.md:41`. Japanese split, ordinary phonetic conversion, and reference lyric composition call `JaG2p` at `rewrite-in-rust/rust/crates/v2m-core/src/lyric_language.rs:262`, `rewrite-in-rust/rust/crates/v2m-core/src/lyric_language.rs:270`, and `rewrite-in-rust/rust/crates/v2m-core/src/lyric_language.rs:281`, matching the pyopenjtalk-absent fallback seam accepted in `rewrite-in-rust/records/0060-close-ja-g2p-fallback-gate.md:29`.

Ordering, empty-input shapes, and complexity do not add separate data/algorithm findings. Factory dispatch lowercases and returns `zh`, `en`, `ja` in order at `rewrite-in-rust/rust/crates/v2m-core/src/lyric_language.rs:71` and `rewrite-in-rust/rust/crates/v2m-core/src/lyric_language.rs:81`. Chinese empty phonetics preserve Python `.split(' ')` behavior through `split_on_ascii_space_preserving_empty` at `rewrite-in-rust/rust/crates/v2m-core/src/lyric_language.rs:201` and `rewrite-in-rust/rust/crates/v2m-core/src/lyric_language.rs:346`, with fixture coverage at `rewrite-in-rust/fixtures/lyric_language_processor_contract.jsonl:6`. The processor-local loops are linear in input characters; the heavier tokenizer and G2P algorithms remain in the verified `zh_g2p` and `ja_g2p` helper seams.

Writer/reviewer separation is preserved. This report covers exactly `lyric_language_processor_contract` as `data_algorithm_reviewer`; I reviewed current state only and did not edit production code, fixtures, bootstrap records, dependency records, or manifest entries.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_lyric_language_processor_contract.py`: passed.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml lyric_language`: passed; 1 `lyric_language` fixture-table test passed, 97 filtered in `v2m-core`, and 0 selected bridge tests.
- `git diff --check`: passed.
- Targeted Python/Rust probe for the prior Chinese bracket-regex examples: passed; Rust now matches Python for `abc]中`, `1]中`, `[Verse]中`, `a]b[c^d-e\\f中  文`, `[]中`, and `ab]]中`.
- Targeted Python/Rust whitespace-membership and processor probes: failed parity for U+001C..U+001F as described in the finding.

## Residual Risk

This review did not re-review Chinese dictionary internals or Japanese fallback tokenizer/table internals; it relies on the closed helper gates in `rewrite-in-rust/records/0058-close-zh-g2p-dictionary-gate.md` and `rewrite-in-rust/records/0060-close-ja-g2p-fallback-gate.md`. No production bridge, bundled Chinese dictionary asset payload, OpenJTalk frontend path, Python-facing error mapping, logging text, GUI/Web/CLI routing, or runtime rollback wiring is proven by this unit.

## Promotion Note

This data/algorithm rerun blocks coordinator update for the `data_algorithm_reviewer` role. Do not mark this role reviewed as passing, and do not mark the manifest unit verified, until the Python regex whitespace parity issue is fixed or explicitly re-scoped with fixtures and a migration record.
