# lyric_matching_file_contract_core - product_ergonomics_reviewer

Date: 2026-07-17
Decision: pass-with-followups

## Findings

No blocking findings for the narrowed no-bridge file-contract seam.

- Severity: low
- Location: rewrite-in-rust/bootstrap/lyric_matching_file_contract_core.md:22
- Issue: Console display text, full directory glob ordering, GUI/Web/CLI routing, and production Rust routing are explicitly outside this unit. That exclusion is acceptable for this review because `rewrite-in-rust/bootstrap/lyric_matching_file_contract_core.md:64` keeps the seam as an independent Rust library, `rewrite-in-rust/bootstrap/lyric_matching_file_contract_core.md:67` keeps legacy Python as runtime owner, and `rewrite-in-rust/bootstrap/lyric_matching_file_contract_core.md:70` states no production caller imports the Rust helpers.
- Evidence: Current production callers still construct/use Python `LyricMatcher`: `inference/API/lfa_api.py:193` creates the Python matcher and `inference/pipeline/auto_lyric_hybrid.py:264` calls that path. A bridge scan found no production imports of `LyricMatchingFilePipeline`; only the Rust workspace module/test references matched.
- Required fix: None before this unit remains `reimplemented`. Promotion must define and verify user-visible display text, directory glob ordering, Python-facing error/log mapping, and rollback before any production caller moves to Rust.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_lyric_matching_file_contract_core.py`: passed.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml lyric_matching_file`: passed; 1 lyric matching file test passed, 0 failed.
- `git diff --check`: passed.
- `rg -n "LyricMatchingFilePipeline|lyric_matching_file|v2m_core" inference application gui web_server.py web_task_manager.py scripts tests rewrite-in-rust/rust/crates --glob '!rewrite-in-rust/rust/target/**'`: inspected; no production Rust lyric matching bridge found.

## Residual Risk

The fixture table covers the core user-relevant file outcomes: missing lyric de-duplication at `rewrite-in-rust/fixtures/lyric_matching_file_contract_core.jsonl:3`, empty ASR skip at line 5, no-match empty JSON at line 6, zh phonetic threshold routing at line 7, non-zh text threshold routing at line 8, and single-file execute JSON/state at line 9. The Rust implementation preserves those outcomes through `process_single_file`, `compare_and_save_result`, and `execute_with_paths` at `rewrite-in-rust/rust/crates/v2m-core/src/lyric_matching_file.rs:121`, `:160`, and `:194`.

What remains unproven is intentionally outside this unit: multi-file `glob.glob` ordering, exact warning/summary/difference display text, task-log wording, batch recovery behavior after write/read errors, and product-facing behavior when integrated into GUI/Web/CLI flows.

## Promotion Note

This role does not block keeping the unit at `reimplemented`. Do not mark the manifest verified from this review alone. Before promotion, require a promotion record and tests for directory glob ordering, display/summary text parity or an accepted text-change policy, task logs, recovery/error handling, batch behavior, downstream JSON consumers, and rollback for any Python/Rust bridge.
