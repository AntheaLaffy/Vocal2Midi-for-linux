# lyric_matching_file_contract_core - behavior_reviewer rerun

Date: 2026-07-17
Decision: pass-with-followups

## Findings

- Severity: low
- Location: rewrite-in-rust/fixtures/lyric_matching_file_contract_core.jsonl:4
- Issue: IO-error behavior is still under-fixtured for behavior parity. The current shared fixture table proves successful lab processing, empty-ASR skip, no-match JSON, diff-threshold routing, negative threshold routing, and single-file execute state, but it does not directly exercise lab read failures, lyric load failures, or JSON write failures after counters mutate.
- Evidence: Python returns `None` for lab read errors at `inference/LyricFA/tools/lyric_matcher.py:148` and wraps JSON write errors at `inference/LyricFA/tools/lyric_matcher.py:73`. Rust returns `None` for lab read errors at `rewrite-in-rust/rust/crates/v2m-core/src/lyric_matching_file.rs:137` and returns `io::Result` from JSON writes at `rewrite-in-rust/rust/crates/v2m-core/src/lyric_matching_file.rs:160` and `rewrite-in-rust/rust/crates/v2m-core/src/lyric_matching_file.rs:185`. Code inspection shows compatible counter order for diff/no-match write failures, but the shared checker does not prove those branches.
- Required fix: Before Python-facing promotion, add focused parity cases for unreadable/missing lab after lyric lookup succeeds, lyric load failure during `load_all_lyrics`, failed JSON write after `diff_count` increments, and failed no-match JSON write after `no_match_count` increments but before `success_count`.

- Severity: low
- Location: rewrite-in-rust/bootstrap/lyric_matching_file_contract_core.md:30
- Issue: Full directory `execute()` glob behavior remains intentionally outside this Rust API, so behavior parity is proven for caller-supplied path lists rather than Python's own directory scan.
- Evidence: Python creates the output directory, glob-loads `*.txt`, glob-loads `*.lab`, and sets `total_files = len(asr_lab_files)` at `inference/LyricFA/tools/lyric_matcher.py:236`. Rust creates the output directory but accepts caller-supplied `lyric_paths` and `lab_paths` at `rewrite-in-rust/rust/crates/v2m-core/src/lyric_matching_file.rs:193`. The bootstrap explicitly excludes full directory glob ordering at `rewrite-in-rust/bootstrap/lyric_matching_file_contract_core.md:30`, and the fixture covers only single-file execute state at `rewrite-in-rust/fixtures/lyric_matching_file_contract_core.jsonl:10`.
- Required fix: Before replacing Python `execute()`, either keep glob and extension filtering in Python or add/record parity coverage for extension filtering, missing directories, duplicate stems, glob ordering, and path encoding edge cases.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_lyric_matching_file_contract_core.py`: passed.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml lyric_matching_file`: passed; 1 lyric matching file test passed, 96 filtered in `v2m-core`, and 0 matching tests ran in `v2m_quant_bridge`.
- `cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check`: passed.
- `git diff --check`: passed.
- `rg -n "LyricMatchingFilePipeline|lyric_matching_file|v2m_core" inference application gui web_server.py web_task_manager.py scripts tests rewrite-in-rust/rust/crates --glob '!rewrite-in-rust/rust/target/**'`: inspected; no production Python GUI/Web/CLI caller wiring to the Rust lyric matching file contract was found.

The prior medium behavior finding for signed `diff_threshold` is closed. Python stores the threshold as an `int` at `inference/LyricFA/tools/lyric_matcher.py:91` and compares at `inference/LyricFA/tools/lyric_matcher.py:191`; Rust now stores and constructs it as `i64` at `rewrite-in-rust/rust/crates/v2m-core/src/lyric_matching_file.rs:68` and `rewrite-in-rust/rust/crates/v2m-core/src/lyric_matching_file.rs:79`, compares signed values at `rewrite-in-rust/rust/crates/v2m-core/src/lyric_matching_file.rs:174`, and fixture line 9 proves negative-threshold parity.

## Residual Risk

The current proof is fixture-bound and library-only. It covers filename extraction, lab-to-lyric mapping, missing lyric de-duplication, successful lab processing, empty-ASR skip, no-match output, zh/non-zh diff routing, negative threshold behavior, JSON schema, and single-file execute state. It does not prove exact console text, full directory glob behavior, production caller integration, language processor behavior, G2P behavior, or sequence alignment internals, all of which are excluded or owned by adjacent units.

## Promotion Note

This behavior role can be recorded as reviewed with follow-ups. The follow-ups do not block the independent Rust library seam from remaining `reimplemented`, but they should be resolved or explicitly assigned before any Python-facing promotion of `LyricMatchingPipeline.execute()` or JSON-error behavior. Do not mark the manifest verified from this report alone.
