# lyric_matching_file_contract_core - dependency_bootstrap_reviewer

Date: 2026-07-17
Decision: pass-with-followups

## Findings

- Severity: low
- Location: rewrite-in-rust/rust/crates/v2m-core/src/lyric_matching_file.rs:79
- Issue: The Rust seam narrows `diff_threshold` to `usize`, while legacy Python accepts an unvalidated signed `int` in `LyricMatchingPipeline.__init__` and compares against it directly.
- Evidence: Python stores `diff_threshold: int = 5` without validation in `inference/LyricFA/tools/lyric_matcher.py:91` and uses `diff_count > self.diff_threshold` in `inference/LyricFA/tools/lyric_matcher.py:191`. The Rust API takes `diff_threshold: usize` at `rewrite-in-rust/rust/crates/v2m-core/src/lyric_matching_file.rs:79`; the Rust fixture harness also reads fixture thresholds with `as_u64().unwrap_or(5)` at `rewrite-in-rust/rust/crates/v2m-core/src/lyric_matching_file.rs:546`, so a negative threshold case cannot be represented by the current checker.
- Required fix: Before behavior/error/product gates rely on this API, either add negative-threshold parity coverage and change the Rust seam to a signed threshold, or record a promotion-time invariant that Python-facing payload validation rejects negative thresholds before reaching Rust.

## Boundary Review

Manifest unit boundary: confirmed with the follow-up above.

Splitting the original `lyric_matching_pipeline_contract` is justified. The record identifies that the original unit mixed language processor selection, G2P, sequence alignment, file IO/state, JSON persistence, console display, and summary behavior in one broad surface (`rewrite-in-rust/records/0061-split-lyric-matching-pipeline-contract.md:13`). The split keeps this unit on file/state/JSON behavior and moves processor flow to `lyric_language_processor_contract` (`rewrite-in-rust/records/0061-split-lyric-matching-pipeline-contract.md:34`).

Dependency/bootstrap evidence correctly keeps language processors, G2P, and alignment internals outside this unit. `language_processors.py` imports and owns `ZhG2p`, `JaG2p`, and processor selection (`inference/LyricFA/tools/language_processors.py:6`, `inference/LyricFA/tools/language_processors.py:111`), while adjacent records keep sequence alignment, Chinese G2P, and Japanese fallback G2P in their own seams (`rewrite-in-rust/records/0055-confirm-lyric-sequence-alignment-boundary.md:24`, `rewrite-in-rust/records/0057-confirm-zh-g2p-dictionary-boundary.md:31`, `rewrite-in-rust/records/0059-confirm-ja-g2p-fallback-boundary.md:32`). The current Rust module only calls the already verified `calculate_difference_count` helper for threshold routing (`rewrite-in-rust/rust/crates/v2m-core/src/lyric_matching_file.rs:15`, `rewrite-in-rust/rust/crates/v2m-core/src/lyric_matching_file.rs:174`).

The injected matcher seam is appropriate and specific enough for dependency/bootstrap. `LyricMatcherBackend` exposes only lyric-file processing, ASR-content processing, and alignment result injection (`rewrite-in-rust/rust/crates/v2m-core/src/lyric_matching_file.rs:48`), which maps to the legacy pipeline calls at `inference/LyricFA/tools/lyric_matcher.py:123`, `inference/LyricFA/tools/lyric_matcher.py:156`, and `inference/LyricFA/tools/lyric_matcher.py:162` without pulling processor or alignment implementation details into this unit.

The fixture/checker and rollback route are sufficient for writer/review gates, subject to the threshold follow-up. Fixtures cover filename extraction, lab-to-lyric mapping, missing lyric de-duplication, success, empty-ASR skip, no-match JSON, zh/non-zh diff routing, result JSON schema, and single-file execute state (`rewrite-in-rust/fixtures/lyric_matching_file_contract_core.jsonl:1`). Rollback is production imports staying on `LyricMatcher` and `LyricMatchingPipeline` (`rewrite-in-rust/bootstrap/lyric_matching_file_contract_core.md:79`). No production bridge is introduced; `lib.rs` only exposes the Rust library module (`rewrite-in-rust/rust/crates/v2m-core/src/lib.rs:18`).

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_lyric_matching_file_contract_core.py`: passed.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml lyric_matching_file`: passed, 1 test passed and 96 filtered in `v2m-core`; 0 tests run in `v2m_quant_bridge`.
- `git diff --check`: passed.

## Residual Risk

Console display text, full directory glob ordering, Python-facing error mapping, language processor payloads, and production bridge wiring remain intentionally unproven and legacy-owned. The current dependency/bootstrap gate also does not prove negative `diff_threshold` parity until the follow-up is resolved.

## Promotion Note

This dependency/bootstrap role does not block writer/review progression, but the signed threshold mismatch should be closed or explicitly recorded before promotion planning. Do not mark the manifest verified from this review alone.
