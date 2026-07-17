# lyric_language_processor_contract - behavior_reviewer

Date: 2026-07-17
Decision: pass

Unit: `lyric_language_processor_contract`
Role: `behavior_reviewer`

## Findings

No behavior parity findings.

Evidence:
- Factory ordering, case-insensitive selection, and unsupported-language error text are preserved: Python uses `{'zh', 'en', 'ja'}` insertion order and raises `ValueError(f"Unsupported language: {language_code}")` (`inference/LyricFA/tools/language_processors.py:111`), while Rust returns `["zh", "en", "ja"]` and formats the same original rejected code (`rewrite-in-rust/rust/crates/v2m-core/src/lyric_language.rs:36`, `rewrite-in-rust/rust/crates/v2m-core/src/lyric_language.rs:71`, `rewrite-in-rust/rust/crates/v2m-core/src/lyric_language.rs:81`).
- Clean/split/phonetic flow is fixture-backed for English, Chinese, and Japanese, including the legacy Chinese empty-input phonetic shape `[""]` (`rewrite-in-rust/fixtures/lyric_language_processor_contract.jsonl:2`, `rewrite-in-rust/fixtures/lyric_language_processor_contract.jsonl:4`, `rewrite-in-rust/fixtures/lyric_language_processor_contract.jsonl:5`, `rewrite-in-rust/fixtures/lyric_language_processor_contract.jsonl:6`). The Python checker injects Mandarin maps and forces `pyopenjtalk = None` before checking legacy output (`rewrite-in-rust/bootstrap/check_lyric_language_processor_contract.py:20`, `rewrite-in-rust/bootstrap/check_lyric_language_processor_contract.py:60`, `rewrite-in-rust/bootstrap/check_lyric_language_processor_contract.py:76`).
- Base and language-specific cleaning behavior matches the selected boundary: Python base cleaning is defined at `inference/LyricFA/tools/language_processors.py:15`, Chinese currently inherits the regex behavior (`inference/LyricFA/tools/language_processors.py:28`), English filters ASCII letters/digits/punctuation before `zh_split_string(text.lower())` (`inference/LyricFA/tools/language_processors.py:42`), and Japanese only normalizes whitespace (`inference/LyricFA/tools/language_processors.py:62`). Rust mirrors those paths in `ChineseProcessor::clean_text`, `EnglishProcessor::clean_text`, and `JapaneseProcessor::clean_text` (`rewrite-in-rust/rust/crates/v2m-core/src/lyric_language.rs:193`, `rewrite-in-rust/rust/crates/v2m-core/src/lyric_language.rs:221`, `rewrite-in-rust/rust/crates/v2m-core/src/lyric_language.rs:260`).
- Chinese processor flow composes `zh_g2p::split_string` and injected-map `ZhG2p::convert_list` as required (`inference/LyricFA/tools/language_processors.py:35`, `inference/LyricFA/tools/language_processors.py:38`, `rewrite-in-rust/rust/crates/v2m-core/src/lyric_language.rs:199`, `rewrite-in-rust/rust/crates/v2m-core/src/lyric_language.rs:203`). The upstream G2P close record accepts the verified injected-map seam and keeps full dictionary file I/O legacy-owned (`rewrite-in-rust/records/0058-close-zh-g2p-dictionary-gate.md:41`, `rewrite-in-rust/records/0058-close-zh-g2p-dictionary-gate.md:81`).
- Japanese processor flow and `build_reference_lyric` preserve the pyopenjtalk-absent fallback path: Python uses `split_kana_no_regex`, `convert_list(..., convert_number=True)` for ordinary flow, and `convert_number=False` for reference lyrics (`inference/LyricFA/tools/language_processors.py:66`, `inference/LyricFA/tools/language_processors.py:73`, `inference/LyricFA/tools/language_processors.py:78`). Rust follows the same split, phonetic, and reference tuple behavior (`rewrite-in-rust/rust/crates/v2m-core/src/lyric_language.rs:264`, `rewrite-in-rust/rust/crates/v2m-core/src/lyric_language.rs:272`, `rewrite-in-rust/rust/crates/v2m-core/src/lyric_language.rs:283`). The Japanese G2P close record confirms this is the accepted Linux fallback seam and excludes OpenJTalk parity (`rewrite-in-rust/records/0060-close-ja-g2p-fallback-gate.md:25`, `rewrite-in-rust/records/0060-close-ja-g2p-fallback-gate.md:70`).
- Output shapes and ordering remain compatible: Python `LyricData` is `text_list`, `phonetic_list`, `raw_text` (`inference/LyricFA/tools/language_processors.py:104`), and `LyricMatcher.process_lyric_text` stores the cleaned raw text after either Japanese reference flow or ordinary split/phonetic flow (`inference/LyricFA/tools/lyric_matcher.py:37`). Rust `LyricData`, `process_text`, and `process_reference_lyric` preserve the same field order and cleaned raw text behavior (`rewrite-in-rust/rust/crates/v2m-core/src/lyric_language.rs:15`, `rewrite-in-rust/rust/crates/v2m-core/src/lyric_language.rs:141`, `rewrite-in-rust/rust/crates/v2m-core/src/lyric_language.rs:154`).
- Rollback and no-production-wiring constraints are intact. The manifest keeps this unit `current_owner: legacy` and `status: reimplemented` rather than verified/promoted (`rewrite-in-rust/manifest.yaml:1271`), the bootstrap states no production caller imports Rust language processor helpers (`rewrite-in-rust/bootstrap/lyric_language_processor_contract.md:72`, `rewrite-in-rust/bootstrap/lyric_language_processor_contract.md:146`), and the Rust crate remains an independent test surface (`rewrite-in-rust/rust/crates/v2m-core/src/lib.rs:1`).

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_lyric_language_processor_contract.py`: passed.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml lyric_language`: passed; 1 `lyric_language` test passed, 97 filtered in `v2m-core`, and 0 selected bridge tests.
- `git diff --check`: passed.
- Targeted Python probes for factory order/error text, zh/en/ja clean/split/phonetic output, Chinese empty-input `[""]`, and Japanese reference tuples: passed.

## Residual Risk

This review proves the current fixture-bound, no-bridge behavior surface. It does not prove bundled Chinese dictionary asset packaging, malformed dictionary payload validation, OpenJTalk frontend-backed Japanese G2P, Python-facing bridge error/log mapping, GUI/Web/CLI integration, or production routing. Those remain outside this unit's behavior review and must be handled before any promotion wires Rust into production callers.

## Promotion Note

This behavior review does not block coordinator update for the `behavior_reviewer` role. The coordinator should not mark the unit verified or promoted from this report alone; the manifest still lists dependency/bootstrap and data/algorithm review roles for this unit.
