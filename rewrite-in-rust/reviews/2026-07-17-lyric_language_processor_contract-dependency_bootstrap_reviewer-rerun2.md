# lyric_language_processor_contract - dependency_bootstrap_reviewer rerun2

Date: 2026-07-17
Decision: pass

Unit: `lyric_language_processor_contract`
Role: `dependency_bootstrap_reviewer`

## Findings

No dependency/bootstrap findings.

The prior Japanese reference-path `LyricData` fixture gap is closed. The shared
fixture now includes `japanese_reference_lyric_data_shape` with `text_list`,
`phonetic_list`, and cleaned `raw_text` at
`rewrite-in-rust/fixtures/lyric_language_processor_contract.jsonl:13`. The
Python checker exercises the `LyricMatcher.process_lyric_text`-equivalent
reference branch at
`rewrite-in-rust/bootstrap/check_lyric_language_processor_contract.py:137`,
and the Rust fixture test calls the actual `Processor::process_reference_lyric`
API at `rewrite-in-rust/rust/crates/v2m-core/src/lyric_language.rs:499`.

## Boundary Decision

Boundary decision: confirmed. The unit remains a narrow independent-library
seam for processor factory selection, clean/split/phonetic orchestration,
Japanese reference-lyric construction, and caller-visible `LyricData` shape.
The manifest keeps the unit `reimplemented`, `inventory_status: confirmed`,
and `current_owner: legacy` at `rewrite-in-rust/manifest.yaml:1271`.

Capability coverage matches the dependency record. Factory selection,
clean-text policy, English flow, Chinese flow, Japanese fallback flow, and
`LyricData` shape remain separately named capabilities at
`rewrite-in-rust/dependencies/lyric_language_processor_contract.yaml:3`.
The fixture table covers those capabilities, including Python regex whitespace
membership, the Chinese bracket-regex artifact, injected Mandarin maps,
Japanese fallback/reference behavior, ordinary `LyricData`, and Japanese
reference-path `LyricData` at
`rewrite-in-rust/fixtures/lyric_language_processor_contract.jsonl:1`.

Dependency expansion still supports this boundary. Legacy
`language_processors.py` imports only standard-library modules plus local
`ZhG2p` and `JaG2p` helpers at
`inference/LyricFA/tools/language_processors.py:1`. The Rust module imports only
`std` and the existing local `ja_g2p` and `zh_g2p` modules at
`rewrite-in-rust/rust/crates/v2m-core/src/lyric_language.rs:9`. No dependency
was added to `rewrite-in-rust/rust/crates/v2m-core/Cargo.toml`; the tracked Rust
integration change only exports `pub mod lyric_language` from
`rewrite-in-rust/rust/crates/v2m-core/src/lib.rs:18`.

Kept-legacy choices remain appropriate. Bundled Chinese dictionary file IO and
packaging, OpenJTalk frontend execution, lyric matching file/state/JSON work,
sequence alignment internals, model execution, GUI/Web/CLI routing, and
production bridge wiring remain outside this unit at
`rewrite-in-rust/dependencies/lyric_language_processor_contract.yaml:50` and
`rewrite-in-rust/records/0063-confirm-lyric-language-processor-boundary.md:50`.
`pyopenjtalk` remains Windows-markered in `pyproject.toml:21`,
`requirements.txt:20`, and `uv.lock:4304`; the Linux seam continues to compose
the already verified pyopenjtalk-absent Japanese fallback.

No bridge or runtime-owner creep was found. The dependency record declares an
empty bridge dependency list at
`rewrite-in-rust/dependencies/lyric_language_processor_contract.yaml:28`, and a
repository search found production `ProcessorFactory` and `LyricData` imports
still resolving to Python at `inference/LyricFA/tools/lyric_matcher.py:7`.

Writer/reviewer separation is preserved. This report reviews current state only
and does not modify production code, fixtures, bootstrap/dependency records,
manifest state, or migration records.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_lyric_language_processor_contract.py`: passed.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml lyric_language`: passed; 1 selected `v2m-core` test passed, 97 were filtered out, and 0 bridge tests were selected.
- `uv run python scripts/audit_vendored_sources.py`: passed; 135 Python packages, 41 native-extension packages, 269 foreign runtime native binaries, and 0 third-party binary artifacts.
- `cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check`: passed.
- `git diff --check`: passed before this report was written.
- Static dependency/runtime search: no PyO3, subprocess, HTTP, OpenJTalk, GUI, Web, or model-runtime dependency was introduced by this unit, and no production caller imports the Rust seam.

## Residual Risk

This review does not prove production promotion. A future promotion record must
still define Chinese dictionary asset packaging and validation, OpenJTalk
ownership or explicit exclusion, Python-facing error mapping, logging text, and
rollback. The verified seam remains fixture-bound, uses injected Mandarin maps,
and forces the Japanese pyopenjtalk-absent path.

## Promotion Note

This dependency/bootstrap role does not block coordinator state update. The
unit boundary is confirmed, the prior fixture follow-up is closed, and this
role is ready to be combined with the separately required behavior and
data/algorithm review evidence. This report does not update the manifest.
