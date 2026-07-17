# lyric_language_processor_contract - dependency_bootstrap_reviewer

Date: 2026-07-17
Decision: pass-with-followups

## Findings

- Severity: low
- Location: rewrite-in-rust/bootstrap/lyric_language_processor_contract.md:14
- Issue: The boundary includes the `LyricData` text/phonetic/raw-text output shape, but the unit fixture/checker does not directly construct or compare the Python `LyricData` dataclass or the Rust `LyricData` returned by `process_text` / `process_reference_lyric`.
- Evidence: The dependency record lists `lyric_data_shape` as an in-scope capability at `rewrite-in-rust/dependencies/lyric_language_processor_contract.yaml:24`. The Python checker encodes processor flows as ad hoc dictionaries containing `cleaned`, `text_list`, and `phonetic_list` at `rewrite-in-rust/bootstrap/check_lyric_language_processor_contract.py:76`, and Japanese reference flow similarly returns dictionaries at `rewrite-in-rust/bootstrap/check_lyric_language_processor_contract.py:126`. The Rust module exposes the actual `LyricData` struct at `rewrite-in-rust/rust/crates/v2m-core/src/lyric_language.rs:15` and returns it from `process_text` / `process_reference_lyric` at `rewrite-in-rust/rust/crates/v2m-core/src/lyric_language.rs:143` and `rewrite-in-rust/rust/crates/v2m-core/src/lyric_language.rs:156`, but the Rust fixture test compares the same ad hoc flow dictionaries at `rewrite-in-rust/rust/crates/v2m-core/src/lyric_language.rs:391`.
- Required fix: Add a small fixture/checker branch that exercises `LyricMatcher.process_lyric_text`-equivalent `LyricData` construction for non-Japanese and Japanese reference flow, including `raw_text == cleaned`, before treating `LyricData` as promotion-ready. This does not require splitting, merging, or deferring the current unit boundary.

## Boundary Decision

Boundary decision: confirmed. `lyric_language_processor_contract` should remain one narrow unit for processor selection, clean/split/phonetic orchestration, Japanese reference lyric flow, and composition with already verified G2P helper seams.

The split from the original lyric matching pipeline remains justified: record 0061 identifies language processor flow, G2P, sequence alignment, file IO/state, JSON persistence, and display text as separable responsibilities at `rewrite-in-rust/records/0061-split-lyric-matching-pipeline-contract.md:18`, and assigns processor flow to this unit at `rewrite-in-rust/records/0061-split-lyric-matching-pipeline-contract.md:53`. The matching file/state unit is already closed with language processors kept out of that boundary at `rewrite-in-rust/records/0062-close-lyric-matching-file-gate.md:7`.

Dependency expansion supports the current seam. `language_processors.py` imports only standard-library modules plus local `ZhG2p` and `JaG2p` helpers at `inference/LyricFA/tools/language_processors.py:1`. The boundary record confirms those imports and keeps package-level language dependencies, PyO3, subprocess, HTTP, CLI, runtime-router, OpenJTalk, model-runtime, GUI, and Web dependencies out of scope at `rewrite-in-rust/records/0063-confirm-lyric-language-processor-boundary.md:17` and `rewrite-in-rust/records/0063-confirm-lyric-language-processor-boundary.md:50`.

The kept-legacy decisions are appropriate. Chinese dictionary file IO remains Python-owned at `rewrite-in-rust/dependencies/lyric_language_processor_contract.yaml:50`, and `ZhG2p.__init__` loads bundled dictionary files at `inference/LyricFA/tools/ZhG2p.py:80`. OpenJTalk frontend behavior remains Python-owned at `rewrite-in-rust/dependencies/lyric_language_processor_contract.yaml:53`; project dependencies mark `pyopenjtalk` as Windows-only in `pyproject.toml:21`, `requirements.txt:20`, and `uv.lock:4304`. The verified adjacent G2P records accept injected-map Chinese G2P and pyopenjtalk-absent Japanese fallback seams at `rewrite-in-rust/records/0058-close-zh-g2p-dictionary-gate.md:43` and `rewrite-in-rust/records/0060-close-ja-g2p-fallback-gate.md:31`.

The hand-written Rust replacement is the right choice. No direct language-processing crate is needed for the selected behavior; the Rust module imports only the existing `zh_g2p` and `ja_g2p` modules at `rewrite-in-rust/rust/crates/v2m-core/src/lyric_language.rs:12`, and `v2m-core` has no new language-processing, bridge, GUI, Web, or OpenJTalk dependency in `rewrite-in-rust/rust/crates/v2m-core/Cargo.toml:12`.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_lyric_language_processor_contract.py`: passed.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml lyric_language`: passed; 1 `lyric_language` fixture-table test passed, 0 failed.
- `uv run python scripts/audit_vendored_sources.py`: passed; 135 Python packages, 41 native-extension packages, 269 foreign runtime native binaries, 0 third_party binary artifacts.
- `cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check`: passed.
- Static review of `git diff -- rewrite-in-rust/manifest.yaml rewrite-in-rust/rust/crates/v2m-core/src/lib.rs`, unit artifacts, and source references: no production Rust/Python routing, bridge, PyO3, subprocess, OpenJTalk, GUI, or Web dependency was introduced by this unit.

## Residual Risk

This review does not prove runtime promotion behavior. A future promotion record must still define Chinese dictionary asset payloads and validation, OpenJTalk ownership or explicit exclusion, Python-facing error mapping, logging text, and rollback. The low `LyricData` fixture gap should be closed before a production bridge or caller-facing API relies on the Rust `LyricData` helpers.

## Promotion Note

This dependency/bootstrap role does not block the coordinator from recording the role as reviewed with follow-ups. Do not mark the manifest unit verified from this role alone; behavior and data/algorithm reviews are still required, and the `LyricData` fixture follow-up should be tracked before promotion.
