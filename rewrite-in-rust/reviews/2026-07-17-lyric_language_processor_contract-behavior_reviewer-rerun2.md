# lyric_language_processor_contract - behavior_reviewer rerun2

Date: 2026-07-17
Decision: pass

Unit: `lyric_language_processor_contract`
Role: `behavior_reviewer` (manifest `stage_behavior_reviewer` evidence)

## Findings

No behavior parity findings.

Evidence:

- Both harnesses consume every non-comment line in the same 14-case JSONL table and compare the actual result against the complete expected object, including object keys, array lengths, ordering, values, and errors (`rewrite-in-rust/bootstrap/check_lyric_language_processor_contract.py:29`, `rewrite-in-rust/bootstrap/check_lyric_language_processor_contract.py:169`, `rewrite-in-rust/rust/crates/v2m-core/src/lyric_language.rs:429`, `rewrite-in-rust/rust/crates/v2m-core/src/lyric_language.rs:450`). The current Python and Rust fixture gates both pass.
- The post-writer whitespace additions cover every `U+001C..U+001F` separator on all three clean-text paths (`rewrite-in-rust/fixtures/lyric_language_processor_contract.jsonl:3`, `rewrite-in-rust/fixtures/lyric_language_processor_contract.jsonl:6`, `rewrite-in-rust/fixtures/lyric_language_processor_contract.jsonl:11`). Legacy Python uses `re.sub(r'\s+', ' ', ...)` in base and Japanese cleaning (`inference/LyricFA/tools/language_processors.py:15`, `inference/LyricFA/tools/language_processors.py:62`). Rust explicitly adds `U+001C..U+001F` to `char::is_whitespace()` and applies the same collapsing helper to Chinese, English, and Japanese (`rewrite-in-rust/rust/crates/v2m-core/src/lyric_language.rs:193`, `rewrite-in-rust/rust/crates/v2m-core/src/lyric_language.rs:219`, `rewrite-in-rust/rust/crates/v2m-core/src/lyric_language.rs:258`, `rewrite-in-rust/rust/crates/v2m-core/src/lyric_language.rs:307`, `rewrite-in-rust/rust/crates/v2m-core/src/lyric_language.rs:346`). An independent Python probe produced `A B C D E`, `中 文 中 文 中`, and `き き き き き`, with their expected token and phonetic lists.
- The new Japanese reference `LyricData` fixture exercises the caller-visible reference path, not only the lower-level tuple (`rewrite-in-rust/fixtures/lyric_language_processor_contract.jsonl:13`). The Python harness follows the production `LyricMatcher.process_lyric_text` dispatch and returns `text_list`, `phonetic_list`, then cleaned `raw_text` (`rewrite-in-rust/bootstrap/check_lyric_language_processor_contract.py:137`, `inference/LyricFA/tools/lyric_matcher.py:37`). The Rust fixture arm calls `Processor::process_reference_lyric`, whose Japanese branch builds the same fields from `build_reference_lyric` (`rewrite-in-rust/rust/crates/v2m-core/src/lyric_language.rs:156`, `rewrite-in-rust/rust/crates/v2m-core/src/lyric_language.rs:499`). The independent Python probe returned kana `['きゃ', 'っ', 'と', '一', '二', 'a']`, phonetics `['kya', 'cl', 'to', '一', '二', 'a']`, and cleaned raw text `キャット １２ A`, matching the fixture and Rust gate.
- The remaining current fixtures preserve factory ordering, case-insensitive selection, original unsupported-code error text, English filtering/lowercase/identity phonetics, Chinese injected-map conversion and legacy bracket-regex/empty-list shapes, Japanese fallback conversion and number policy, and empty reference behavior (`rewrite-in-rust/fixtures/lyric_language_processor_contract.jsonl:1`, `rewrite-in-rust/fixtures/lyric_language_processor_contract.jsonl:2`, `rewrite-in-rust/fixtures/lyric_language_processor_contract.jsonl:4`, `rewrite-in-rust/fixtures/lyric_language_processor_contract.jsonl:5`, `rewrite-in-rust/fixtures/lyric_language_processor_contract.jsonl:7`, `rewrite-in-rust/fixtures/lyric_language_processor_contract.jsonl:8`, `rewrite-in-rust/fixtures/lyric_language_processor_contract.jsonl:9`, `rewrite-in-rust/fixtures/lyric_language_processor_contract.jsonl:10`, `rewrite-in-rust/fixtures/lyric_language_processor_contract.jsonl:12`, `rewrite-in-rust/fixtures/lyric_language_processor_contract.jsonl:14`). All are reached by both passing table loops.
- Rollback remains valid. The manifest keeps the unit `reimplemented` with `current_owner: legacy`, and the production caller still imports Python `ProcessorFactory` and `LyricData` (`rewrite-in-rust/manifest.yaml:1271`, `inference/LyricFA/tools/lyric_matcher.py:7`). The Rust implementation is only exposed by the independent `v2m-core` crate (`rewrite-in-rust/rust/crates/v2m-core/src/lib.rs:18`); the targeted repository search found no production caller routed to it.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_lyric_language_processor_contract.py`: passed all 14 current fixture cases.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml lyric_language -- --nocapture`: passed; 1 selected `v2m-core` table test passed, 97 tests were filtered out, and no bridge test was selected.
- `UV_CACHE_DIR=/tmp/vocal2midi-uv-cache uv run python -c <targeted fixture probe>`: passed; printed expected actual outputs for all three `U+001C..U+001F` cases and Japanese reference `LyricData`.
- `cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check`: passed.
- `git diff --check`: passed.
- Static review of the manifest unit, records 0063/0064, dependency/bootstrap evidence, all current fixtures, Python sources, Rust implementation/test dispatch, and runtime-owner search: no behavior blocker or production Rust routing found.

## Residual Risk

This review proves the current no-bridge, fixture-bound Linux fallback surface. It does not prove bundled Chinese dictionary asset loading/packaging, malformed external dictionary payloads, OpenJTalk-backed Japanese frontend behavior, Python-facing bridge error/log mapping, model execution, GUI/Web/CLI integration, or a future production routing change. Those remain legacy-owned or require separate promotion evidence.

## Promotion Note

This rerun does not block the coordinator from recording the manifest `stage_behavior_reviewer` requirement as passed for `lyric_language_processor_contract`. It does not by itself justify changing the unit to `verified` or `promoted`.
