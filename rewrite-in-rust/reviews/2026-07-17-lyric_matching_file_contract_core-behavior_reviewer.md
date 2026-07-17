# lyric_matching_file_contract_core - behavior_reviewer

Date: 2026-07-17
Decision: pass-with-followups

## Findings

- Severity: medium
- Location: rewrite-in-rust/rust/crates/v2m-core/src/lyric_matching_file.rs:68
- Issue: `diff_threshold` is narrowed to `usize`, while the Python public constructor accepts any `int`. A negative threshold is valid Python input and makes any non-negative diff count exceed the threshold.
- Evidence: Python stores `diff_threshold: int = 5` directly and compares `diff_count > self.diff_threshold` in `inference/LyricFA/tools/lyric_matcher.py:85` and `inference/LyricFA/tools/lyric_matcher.py:191`; Rust stores and constructs the threshold as `usize` in `rewrite-in-rust/rust/crates/v2m-core/src/lyric_matching_file.rs:68` and `rewrite-in-rust/rust/crates/v2m-core/src/lyric_matching_file.rs:79`. The fixture table covers `0` and `5`, but not negative threshold behavior in `rewrite-in-rust/fixtures/lyric_matching_file_contract_core.jsonl:6`.
- Required fix: Either add a parity fixture for negative thresholds and change the Rust threshold type to preserve it, or record a non-negative precondition before this unit is promoted into a Python-facing caller boundary.

- Severity: low
- Location: rewrite-in-rust/fixtures/lyric_matching_file_contract_core.jsonl:4
- Issue: Read/write error behavior is under-fixtured. The implemented branch shapes look compatible for the happy-path and skip cases, but fixture evidence does not prove lab read failures, lyric read failures during load, JSON write failures, or state mutation order when JSON writes fail.
- Evidence: Python catches lab open errors and returns `None` in `inference/LyricFA/tools/lyric_matcher.py:148`; Rust maps `fs::read_to_string` failure to `None` in `rewrite-in-rust/rust/crates/v2m-core/src/lyric_matching_file.rs:137`. Python wraps JSON write failures as `IOError` in `inference/LyricFA/tools/lyric_matcher.py:73`, while Rust returns `io::Result` from `rewrite-in-rust/rust/crates/v2m-core/src/lyric_matching_file.rs:160` and `rewrite-in-rust/rust/crates/v2m-core/src/lyric_matching_file.rs:185`. Current fixtures only cover successful lab reads and successful JSON writes.
- Required fix: Add focused parity cases for unreadable/missing lab after lyric lookup succeeds, failed lyric processing in load, failed JSON write after a diff-threshold increment, and failed no-match JSON write after `no_match_count` increments but before `success_count`.

- Severity: low
- Location: rewrite-in-rust/rust/crates/v2m-core/src/lyric_matching_file.rs:103
- Issue: Directory glob behavior remains deliberately outside the Rust API, so execute parity is proven only for caller-supplied single-file path lists. This matches the current bootstrap exclusion, but it is a promotion risk if `execute()` itself is later wired across the language boundary.
- Evidence: Python owns `os.makedirs`, `glob.glob` for `*.txt`, `glob.glob` for `*.lab`, and `total_files = len(asr_lab_files)` in `inference/LyricFA/tools/lyric_matcher.py:236`; Rust accepts `lyric_paths` and `lab_paths` from the caller in `rewrite-in-rust/rust/crates/v2m-core/src/lyric_matching_file.rs:193`. The bootstrap explicitly excludes full directory glob ordering while covering single-file execute state.
- Required fix: Before promotion of an `execute()` replacement, add or record coverage for extension filtering, missing directories, duplicate stems, glob ordering expectations, and non-UTF-8/path edge handling, or keep glob ownership in Python.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_lyric_matching_file_contract_core.py`: passed
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml lyric_matching_file`: passed; 1 lyric matching file test passed, 96 filtered in `v2m-core`, bridge binary had 0 matching tests
- `cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check`: passed
- `git diff --check`: passed
- `rg -n "lyric_matching_file|LyricMatchingFilePipeline|LyricMatchingPipeline|lyric_matching_file_contract_core" .`: no production Python GUI/Web/CLI caller wiring found; matches were limited to legacy Python source, Rust crate export/module, and rewrite artifacts

## Residual Risk

The current fixture table proves the accepted happy-path, skip, no-match, diff-routing, JSON schema, missing-lyric de-duplication, and single-file execute state. It does not yet prove write/read error mutation order, full directory glob behavior, or negative diff-threshold compatibility. Language processors, G2P, sequence alignment internals, display text, model execution, and runtime caller integration remain intentionally outside this unit.

## Promotion Note

This behavior review does not block continued rewrite work, but promotion should not treat the manifest as verified yet. The unit is acceptable as an independent Rust library seam with legacy Python still owning runtime calls; address or explicitly record the follow-ups before any Python-facing promotion of this file contract.
