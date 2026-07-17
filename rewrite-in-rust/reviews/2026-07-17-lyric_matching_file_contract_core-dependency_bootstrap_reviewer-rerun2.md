# lyric_matching_file_contract_core - dependency_bootstrap_reviewer rerun2

Date: 2026-07-17
Decision: pass

## Findings

No findings.

The manifest unit boundary is confirmed. The previous provisional
`lyric_matching_pipeline_contract` was correctly split: record 0061 shows the
original mixed language processing, G2P, sequence alignment, file IO, state,
JSON, and console concerns (`rewrite-in-rust/records/0061-split-lyric-matching-pipeline-contract.md:18`),
then assigns this unit only the file/state/JSON contract with injected matcher
results (`rewrite-in-rust/records/0061-split-lyric-matching-pipeline-contract.md:44`,
`rewrite-in-rust/records/0061-split-lyric-matching-pipeline-contract.md:51`).
The manifest now lists `lyric_matching_file_contract_core` as `reimplemented`
and `confirmed`, while runtime ownership remains legacy Python
(`rewrite-in-rust/manifest.yaml:1241`, `rewrite-in-rust/manifest.yaml:1243`,
`rewrite-in-rust/manifest.yaml:1244`, `rewrite-in-rust/manifest.yaml:1245`).

Capability coverage matches the chosen seam. The dependency record maps filename
handling, pipeline state, JSON schema, and diff-threshold routing to the Rust
file-contract implementation while keeping the sequence difference helper behind
the already verified lyric sequence unit
(`rewrite-in-rust/dependencies/lyric_matching_file_contract_core.yaml:4`,
`rewrite-in-rust/dependencies/lyric_matching_file_contract_core.yaml:8`,
`rewrite-in-rust/dependencies/lyric_matching_file_contract_core.yaml:12`,
`rewrite-in-rust/dependencies/lyric_matching_file_contract_core.yaml:16`).
The legacy source supports that scope: `lyric_matcher.py` owns filename
extraction, lab-to-lyric mapping, missing lyric tracking, lab read/empty-ASR
skip, diff routing, JSON persistence, and execute counters
(`inference/LyricFA/tools/lyric_matcher.py:131`,
`inference/LyricFA/tools/lyric_matcher.py:140`,
`inference/LyricFA/tools/lyric_matcher.py:143`,
`inference/LyricFA/tools/lyric_matcher.py:148`,
`inference/LyricFA/tools/lyric_matcher.py:158`,
`inference/LyricFA/tools/lyric_matcher.py:177`,
`inference/LyricFA/tools/lyric_matcher.py:196`,
`inference/LyricFA/tools/lyric_matcher.py:236`).

Kept-legacy decisions remain accurate for this role. The bootstrap record
explicitly excludes processor selection, Chinese/Japanese G2P, pyopenjtalk,
alignment internals, display text, full glob ordering, model execution,
GUI/Web/CLI routing, and production Rust routing
(`rewrite-in-rust/bootstrap/lyric_matching_file_contract_core.md:22`).
That matches the source dependency shape: `lyric_matcher.py` imports
`ProcessorFactory`, `LyricData`, `SequenceAligner`,
`calculate_difference_count`, and `SmartHighlighter`
(`inference/LyricFA/tools/lyric_matcher.py:7`,
`inference/LyricFA/tools/lyric_matcher.py:8`), while language processors own
Zh/Ja G2P and processor selection
(`inference/LyricFA/tools/language_processors.py:6`,
`inference/LyricFA/tools/language_processors.py:7`,
`inference/LyricFA/tools/language_processors.py:111`). The adjacent dependency
records close the required upstream seams for lyric sequence, Chinese G2P, and
Japanese fallback G2P (`rewrite-in-rust/records/0056-close-lyric-sequence-alignment-gate.md:36`,
`rewrite-in-rust/records/0058-close-zh-g2p-dictionary-gate.md:38`,
`rewrite-in-rust/records/0060-close-ja-g2p-fallback-gate.md:28`).

The hand-written Rust replacement is narrow enough. `LyricMatcherBackend`
injects only lyric-file processing, ASR-content processing, and alignment
outputs (`rewrite-in-rust/rust/crates/v2m-core/src/lyric_matching_file.rs:48`).
The Rust module implements file/state/JSON orchestration, delegates difference
counting to `lyric_sequence`, and accepts caller-supplied paths instead of
owning full glob ordering (`rewrite-in-rust/rust/crates/v2m-core/src/lyric_matching_file.rs:15`,
`rewrite-in-rust/rust/crates/v2m-core/src/lyric_matching_file.rs:103`,
`rewrite-in-rust/rust/crates/v2m-core/src/lyric_matching_file.rs:121`,
`rewrite-in-rust/rust/crates/v2m-core/src/lyric_matching_file.rs:160`,
`rewrite-in-rust/rust/crates/v2m-core/src/lyric_matching_file.rs:193`).
No production caller imports this Rust lyric matching module; the targeted
repository audit found only documentation references outside `rewrite-in-rust/`.

Fixture/bootstrap coverage is sufficient for dependency/bootstrap. The fixture
table covers filename extraction, lab-to-lyric mapping, missing lyric
de-duplication, success, empty-ASR skip, no-match JSON, zh/non-zh diff routing,
negative threshold routing, result JSON schema, and single-file execute state
(`rewrite-in-rust/fixtures/lyric_matching_file_contract_core.jsonl:1`,
`rewrite-in-rust/fixtures/lyric_matching_file_contract_core.jsonl:9`,
`rewrite-in-rust/fixtures/lyric_matching_file_contract_core.jsonl:10`). The
Python checker uses a fake matcher to inject language and alignment behavior,
then calls the real JSON serializer for schema proof
(`rewrite-in-rust/bootstrap/check_lyric_matching_file_contract_core.py:60`,
`rewrite-in-rust/bootstrap/check_lyric_matching_file_contract_core.py:89`,
`rewrite-in-rust/bootstrap/check_lyric_matching_file_contract_core.py:195`).

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_lyric_matching_file_contract_core.py`: passed.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml lyric_matching_file`: passed; 1 `lyric_matching_file` test passed in `v2m-core`, 96 filtered; 0 matching tests in `v2m_quant_bridge`.
- `git diff --check`: passed.
- `rg -n "lyric_matching_file|LyricMatchingFilePipeline|v2m_core|v2m-core" --glob '!rewrite-in-rust/**' --glob '!target/**' .`: passed; only documentation references to `v2m-core`, no production lyric matching Rust caller.
- `rg -n "pyo3|maturin|cdylib|crate-type|subprocess|HTTP|runtime router|runtime-router" rewrite-in-rust/rust/Cargo.toml rewrite-in-rust/rust/crates/v2m-core/Cargo.toml rewrite-in-rust/rust/crates/v2m-core/src/lyric_matching_file.rs rewrite-in-rust/rust/crates/v2m-core/src/lib.rs`: passed; no matches.
- `cargo tree --manifest-path rewrite-in-rust/rust/Cargo.toml -p v2m-core`: passed; `v2m-core` dependencies are `encoding_rs`, `md-5`, and `serde_json`; this unit uses only the existing `serde_json` dependency for JSON.
- `uv run python scripts/audit_vendored_sources.py`: passed; reported 135 Python packages, 41 native-extension packages, 269 covered foreign runtime native binaries, and 0 third-party binary artifacts.

## Residual Risk

This review does not prove behavior beyond the dependency/bootstrap role.
Console display text, full directory glob ordering across multiple files,
language processor payloads, Python-facing error mapping, production bridge
wiring, and runtime promotion rollback remain intentionally unproven and
legacy-owned. Those are already documented as exclusions or promotion-time
requirements (`rewrite-in-rust/bootstrap/lyric_matching_file_contract_core.md:22`,
`rewrite-in-rust/bootstrap/lyric_matching_file_contract_core.md:128`).

## Promotion Note

This dependency/bootstrap role does not block coordinator state update for this
role. Do not mark the manifest verified from this report alone; the manifest
still requires behavior, error-tracing, and product-ergonomics reviews before
coordinator promotion decisions (`rewrite-in-rust/manifest.yaml:1250`).
