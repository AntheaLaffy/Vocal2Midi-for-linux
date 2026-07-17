# lyric_matching_file_contract_core - dependency_bootstrap_reviewer rerun

Date: 2026-07-17
Decision: pass

## Findings

No findings.

The previous low follow-up is closed. Legacy Python stores `diff_threshold` as an unvalidated signed `int` (`inference/LyricFA/tools/lyric_matcher.py:91`) and compares `diff_count > self.diff_threshold` directly (`inference/LyricFA/tools/lyric_matcher.py:191`). The Rust file-contract seam now stores and accepts `diff_threshold: i64` (`rewrite-in-rust/rust/crates/v2m-core/src/lyric_matching_file.rs:68`, `rewrite-in-rust/rust/crates/v2m-core/src/lyric_matching_file.rs:79`) and compares `calculate_difference_count(...) as i64` against it (`rewrite-in-rust/rust/crates/v2m-core/src/lyric_matching_file.rs:174`).

The parity fixture now includes `compare_negative_threshold_counts_equal_sequences`, proving Python/Rust behavior for a negative threshold where equal sequences still count as threshold-exceeding because `0 > -1` (`rewrite-in-rust/fixtures/lyric_matching_file_contract_core.jsonl:9`). The Python checker passes that value through `int(...)` (`rewrite-in-rust/bootstrap/check_lyric_matching_file_contract_core.py:200`), and the Rust fixture harness reads it with `as_i64()` (`rewrite-in-rust/rust/crates/v2m-core/src/lyric_matching_file.rs:542`).

## Boundary Review

Manifest unit boundary: confirmed.

The dependency/bootstrap decision remains sound after the threshold fix. The split from the original `lyric_matching_pipeline_contract` still keeps this unit on file/state/JSON orchestration while `lyric_language_processor_contract` owns language processor behavior. No new bridge, production caller, language processor, G2P, alignment-internal, console display, or glob-ordering dependency was introduced.

The injected matcher seam remains specific enough: `LyricMatcherBackend` only supplies lyric-file processing, ASR-content processing, and alignment outputs (`rewrite-in-rust/rust/crates/v2m-core/src/lyric_matching_file.rs:48`). Sequence difference counting stays delegated to the already verified lyric sequence helper (`rewrite-in-rust/rust/crates/v2m-core/src/lyric_matching_file.rs:15`).

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_lyric_matching_file_contract_core.py`: passed.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml lyric_matching_file`: passed, 1 test passed and 96 filtered in `v2m-core`; 0 tests run in `v2m_quant_bridge`.
- `git diff --check`: passed.

## Residual Risk

Console display text, full directory glob ordering, Python-facing error mapping, language processor payloads, and production bridge wiring remain intentionally unproven and legacy-owned. No dependency/bootstrap blocker remains for the prior signed-threshold concern.

## Promotion Note

This dependency/bootstrap rerun does not block coordinator state update for this role. Do not mark the manifest verified from this report alone; required non-dependency reviews and coordinator policy still apply.
