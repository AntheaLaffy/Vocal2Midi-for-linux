# lyric_matching_file_contract_core - error_tracing_reviewer

Date: 2026-07-17
Decision: pass-with-followups

## Findings

- Severity: low
- Location: rewrite-in-rust/rust/crates/v2m-core/src/lyric_matching_file.rs:103
- Issue: Lyric file processing errors are collapsed into a skipped entry with no retained diagnostic. Legacy Python catches lyric read/process failures and prints the lyric name plus the underlying error, including `Cannot read lyric file ...`, before continuing. Exact console text is intentionally out of scope, but a future bridge would not be able to distinguish a skipped unreadable lyric from an intentionally absent/invalid injected backend output.
- Evidence: Python wraps lyric file read failures at inference/LyricFA/tools/lyric_matcher.py:28 and reports caught load failures at inference/LyricFA/tools/lyric_matcher.py:117. Rust discards `Err` from `backend.process_lyric_file` at rewrite-in-rust/rust/crates/v2m-core/src/lyric_matching_file.rs:112. The boundary excludes exact console output at rewrite-in-rust/bootstrap/lyric_matching_file_contract_core.md:22 and rewrite-in-rust/dependencies/lyric_matching_file_contract_core.yaml:36.
- Required fix: Before production bridge promotion, define a diagnostic side channel or result type that can preserve skipped lyric read/process reasons without requiring exact console text parity.

- Severity: low
- Location: rewrite-in-rust/rust/crates/v2m-core/src/lyric_matching_file.rs:119
- Issue: Lab read errors are also collapsed into `None`, the same shape used for missing lyrics and empty ASR skips. This preserves the selected branch shape, but it is too silent for bridge-level error tracing unless a later promotion layer records skip reasons.
- Evidence: Legacy Python prints a lab read error and returns `None` at inference/LyricFA/tools/lyric_matcher.py:148. Rust uses `fs::read_to_string(lab_path).ok()?` at rewrite-in-rust/rust/crates/v2m-core/src/lyric_matching_file.rs:137. Missing-lyric and empty-ASR skip behavior are separately fixture-covered at rewrite-in-rust/fixtures/lyric_matching_file_contract_core.jsonl:3 and rewrite-in-rust/fixtures/lyric_matching_file_contract_core.jsonl:5, but there is no fixture that proves read-error diagnostics because exact console text is excluded.
- Required fix: Before bridge promotion, decide whether `process_single_file` should expose a structured skip reason for lab read errors, missing lyrics, and empty ASR, or document that bridge callers only receive legacy-compatible skipped-file counts.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_lyric_matching_file_contract_core.py`: passed.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml lyric_matching_file`: passed; 1 lyric matching file test passed, 96 filtered in `v2m-core`, and the quant bridge binary had 0 matching tests.
- `git diff --check`: passed.

## Residual Risk

JSON write failures are compatible with a future bridge at the current seam: Rust returns `io::Result` from `compare_and_save_result`, `handle_no_match`, and `execute_with_paths`, while Python raises `IOError` from `save_to_json`. Counter ordering is compatible: `diff_count` and `no_match_count` are incremented before the write, and `success_count` is incremented only after a successful write in both implementations.

No-match reason handling is currently display-only in Python and exact console text is explicitly out of scope, so Rust not emitting the reason is acceptable for this no-bridge unit. If a later bridge wants user-facing no-match diagnostics, that bridge needs a separate product/error contract.

I did not find redaction issues in the records or fixtures. The fixture file uses generic relative filenames and temporary directories through the harness, not user-specific absolute paths.

## Promotion Note

This role does not block promotion of the no-bridge file/state/JSON seam. Promotion planning should carry the two low-severity follow-ups if Rust becomes a Python-facing lyric matching boundary.
