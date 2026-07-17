# lyric_language_processor_contract - dependency_bootstrap_reviewer rerun

Date: 2026-07-17
Decision: pass-with-followups

## Findings

- Severity: low
- Location: rewrite-in-rust/fixtures/lyric_language_processor_contract.jsonl:7
- Issue: The original `LyricData` fixture gap is closed for the ordinary processor path, but not yet for the Japanese reference-lyric path that also returns caller-visible `LyricData`.
- Evidence: The new `lyric_data_shape` fixture directly covers `text_list`, `phonetic_list`, and cleaned `raw_text` for English at `rewrite-in-rust/fixtures/lyric_language_processor_contract.jsonl:7`; the Python checker has a dedicated branch at `rewrite-in-rust/bootstrap/check_lyric_language_processor_contract.py:126`; and the Rust test now calls `Processor::process_text` and serializes the actual `LyricData` fields at `rewrite-in-rust/rust/crates/v2m-core/src/lyric_language.rs:486`. However, legacy `LyricMatcher.process_lyric_text` uses `build_reference_lyric` for Japanese before returning `LyricData` at `inference/LyricFA/tools/lyric_matcher.py:37`, while the current Japanese fixture/checker path compares `build_reference_lyric` tuple output without a `raw_text`/`LyricData` shape at `rewrite-in-rust/fixtures/lyric_language_processor_contract.jsonl:9` and `rewrite-in-rust/bootstrap/check_lyric_language_processor_contract.py:137`. The Rust `process_reference_lyric` API returns `LyricData` at `rewrite-in-rust/rust/crates/v2m-core/src/lyric_language.rs:156`, but no fixture branch calls it; the Japanese test branch calls `JapaneseProcessor::build_reference_lyric` directly at `rewrite-in-rust/rust/crates/v2m-core/src/lyric_language.rs:495`.
- Required fix: Add a small Japanese `LyricData` shape fixture/checker branch before promotion or caller-facing API use. It should exercise Python's `LyricMatcher.process_lyric_text`-equivalent reference flow and Rust `Processor::process_reference_lyric`, including `raw_text == cleaned`. This does not require splitting, merging, deferring, or replacing the unit.

## Boundary Decision

Boundary decision: confirmed. The unit remains a narrow library seam for processor factory selection, clean/split/phonetic orchestration, Japanese reference-lyric behavior, and composition with the already verified Chinese and Japanese G2P helper seams.

The dependency boundary remains sound. The manifest keeps `lyric_language_processor_contract` at `status: reimplemented`, `inventory_status: confirmed`, and `current_owner: legacy` at `rewrite-in-rust/manifest.yaml:1271`. The dependency record declares a library seam with no bridge dependencies at `rewrite-in-rust/dependencies/lyric_language_processor_contract.yaml:28`, and record 0063 explicitly excludes package-level language dependencies, PyO3, subprocess, HTTP, CLI, runtime-router, OpenJTalk, model-runtime, GUI, and Web dependencies at `rewrite-in-rust/records/0063-confirm-lyric-language-processor-boundary.md:50`.

Kept-legacy decisions still hold. Bundled Chinese dictionary file IO remains out of scope at `rewrite-in-rust/dependencies/lyric_language_processor_contract.yaml:50`; `ZhG2p.__init__` still loads project dictionary files at `inference/LyricFA/tools/ZhG2p.py:80`. OpenJTalk frontend behavior remains out of scope at `rewrite-in-rust/dependencies/lyric_language_processor_contract.yaml:53`; `pyproject.toml:21`, `requirements.txt:20`, and `uv.lock:4304` keep `pyopenjtalk` Windows-markered, while the verified Japanese fallback record accepts the pyopenjtalk-absent seam at `rewrite-in-rust/records/0060-close-ja-g2p-fallback-gate.md:31`.

The hand-written Rust replacement remains appropriate. The Rust module imports only existing local G2P helpers at `rewrite-in-rust/rust/crates/v2m-core/src/lyric_language.rs:12`, and `v2m-core` still has no PyO3, bridge, OpenJTalk, GUI, Web, or language-processing crate dependency in `rewrite-in-rust/rust/crates/v2m-core/Cargo.toml:12`. The only tracked Rust exposure for this unit is the independent module export at `rewrite-in-rust/rust/crates/v2m-core/src/lib.rs:18`.

Chinese bracket-regex fixture/docs are aligned with the dependency boundary. Record 0063 calls out the legacy cleaned shape at `rewrite-in-rust/records/0063-confirm-lyric-language-processor-boundary.md:80`, the fixture proves it at `rewrite-in-rust/fixtures/lyric_language_processor_contract.jsonl:5`, and the Rust implementation keeps the compatibility helper local to this module at `rewrite-in-rust/rust/crates/v2m-core/src/lyric_language.rs:324`.

Writer/reviewer separation was preserved. This report covers exactly `dependency_bootstrap_reviewer` for `lyric_language_processor_contract`; I reviewed current state only and did not edit production code, fixtures, bootstrap records, dependency records, manifest entries, or Rust modules.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_lyric_language_processor_contract.py`: passed.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml lyric_language`: passed; 1 `lyric_language` fixture-table test passed, 97 tests filtered in `v2m-core`, and 0 selected bridge tests.
- `uv run python scripts/audit_vendored_sources.py`: passed; 135 Python packages, 41 native-extension packages, 269 foreign runtime native binaries, 0 third_party binary artifacts.
- `git diff --check`: passed before this review report was written.
- Static diff/scope review: tracked code changes are limited to manifest state/evidence updates and exporting `pub mod lyric_language`; untracked unit artifacts add fixture/bootstrap/dependency/review evidence and `v2m-core::lyric_language` without production routing.

## Residual Risk

This review does not prove runtime promotion behavior. A future promotion record must still define Chinese dictionary asset payloads and validation, OpenJTalk ownership or explicit exclusion, Python-facing error mapping, logging text, and rollback. The remaining low fixture gap is Japanese reference-path `LyricData` construction through the Rust `process_reference_lyric` helper.

## Promotion Note

This dependency/bootstrap role does not block the coordinator from recording the role as reviewed with follow-ups. Do not mark the manifest unit verified from this role alone; the coordinator should either keep the Japanese `LyricData` shape item as a promotion-time follow-up or add that fixture before treating the `LyricData` follow-up as fully closed.
