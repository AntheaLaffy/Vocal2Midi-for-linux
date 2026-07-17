# lyric_matching_file_contract_core - error_tracing_reviewer rerun

Date: 2026-07-17
Decision: pass-with-followups

## Findings

- Severity: low
- Location: rewrite-in-rust/rust/crates/v2m-core/src/lyric_matching_file.rs:112
- Issue: Lyric file processing errors are collapsed into a skipped lyric entry with no retained diagnostic. Legacy Python keeps processing, but it prints the lyric name and wrapped read/process error; the Rust seam drops the backend `Err(String)` entirely.
- Evidence: Python wraps lyric file read failures with `Cannot read lyric file ...` at `inference/LyricFA/tools/lyric_matcher.py:28` and reports caught load failures at `inference/LyricFA/tools/lyric_matcher.py:127`. Rust ignores `Err` from `backend.process_lyric_file` at `rewrite-in-rust/rust/crates/v2m-core/src/lyric_matching_file.rs:112`. Exact console text is excluded at `rewrite-in-rust/bootstrap/lyric_matching_file_contract_core.md:22` and `rewrite-in-rust/dependencies/lyric_matching_file_contract_core.yaml:41`.
- Required fix: Before production bridge promotion, define a structured skip diagnostic or callback that preserves lyric read/process failure reason and path context without requiring exact console output parity.

- Severity: low
- Location: rewrite-in-rust/rust/crates/v2m-core/src/lyric_matching_file.rs:137
- Issue: Lab read errors return the same `None` shape as missing lyrics and empty ASR skips. This matches the selected branch shape, but it prevents bridge-level error tracing from distinguishing unreadable lab files from expected skip states.
- Evidence: Legacy Python prints `Error reading lab file ...` and returns `None` at `inference/LyricFA/tools/lyric_matcher.py:148`. Rust uses `fs::read_to_string(lab_path).ok()?` at `rewrite-in-rust/rust/crates/v2m-core/src/lyric_matching_file.rs:137`. Fixtures cover missing lyric and empty-ASR skips at `rewrite-in-rust/fixtures/lyric_matching_file_contract_core.jsonl:3` and `rewrite-in-rust/fixtures/lyric_matching_file_contract_core.jsonl:5`, but read-error diagnostics are out of fixture scope.
- Required fix: Before bridge promotion, decide whether `process_single_file` exposes structured skip reasons for lab read errors, missing lyrics, and empty ASR, or document that callers only receive legacy-compatible counters and skipped results.

- Severity: low
- Location: rewrite-in-rust/rust/crates/v2m-core/src/lyric_matching_file.rs:251
- Issue: JSON write failures propagate as raw `io::Error` without the legacy `Cannot write JSON file <path>: <error>` context. The current no-bridge library seam can still fail correctly, but promotion-time Python error mapping would not have a durable path/context shape unless the bridge adds it.
- Evidence: Python wraps JSON persistence errors at `inference/LyricFA/tools/lyric_matcher.py:67` and `inference/LyricFA/tools/lyric_matcher.py:77`. Rust writes with `fs::write(path, payload)` at `rewrite-in-rust/rust/crates/v2m-core/src/lyric_matching_file.rs:253`; callers propagate with `?` from `rewrite-in-rust/rust/crates/v2m-core/src/lyric_matching_file.rs:180` and `rewrite-in-rust/rust/crates/v2m-core/src/lyric_matching_file.rs:188`. The bootstrap explicitly defers Python-facing error mapping until promotion at `rewrite-in-rust/bootstrap/lyric_matching_file_contract_core.md:128`.
- Required fix: Before bridge promotion, wrap JSON persistence failures in a path-aware error shape or define a bridge mapping that recreates the legacy `Cannot write JSON file ...` diagnostic.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_lyric_matching_file_contract_core.py`: passed.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml lyric_matching_file`: passed; 1 lyric matching file test passed, 96 filtered in `v2m-core`, and `v2m_quant_bridge` had 0 matching tests.
- `git diff --check`: passed.
- `rg -n "LyricMatchingFilePipeline|lyric_matching_file|LyricMatchingPipeline|v2m_core" inference application gui web_server.py web_task_manager.py scripts tests rewrite-in-rust/rust/crates --glob '!rewrite-in-rust/rust/target/**'`: inspected; no production Python GUI/Web/CLI caller or bridge imports the Rust lyric matching file-contract helper.

## Residual Risk

Threshold and no-match state ordering is compatible with legacy Python for the selected seam. Python increments `diff_count` before JSON persistence and `success_count` only after a successful write at `inference/LyricFA/tools/lyric_matcher.py:191` and `inference/LyricFA/tools/lyric_matcher.py:197`; Rust mirrors that at `rewrite-in-rust/rust/crates/v2m-core/src/lyric_matching_file.rs:174` and `rewrite-in-rust/rust/crates/v2m-core/src/lyric_matching_file.rs:180`. Python increments `no_match_count` before writing the empty JSON and `success_count` after the write at `inference/LyricFA/tools/lyric_matcher.py:200`; Rust mirrors that at `rewrite-in-rust/rust/crates/v2m-core/src/lyric_matching_file.rs:185`.

Exact console output, display highlighting, full directory glob ordering, production bridge wiring, and Python-facing error mapping remain intentionally out of this unit. I did not find a redaction issue in current Rust code because it emits no logs and the fixtures use generic temporary filenames, but any future diagnostic side channel should classify or redact paths deliberately.

## Promotion Note

This role does not block coordinator state update for the current no-bridge file/state/JSON seam. The coordinator can record this role as reviewed with low promotion follow-ups, but should not mark the manifest verified from this report alone.
