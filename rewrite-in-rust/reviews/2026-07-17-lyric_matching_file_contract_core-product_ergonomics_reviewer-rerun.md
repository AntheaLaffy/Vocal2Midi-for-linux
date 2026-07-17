# lyric_matching_file_contract_core - product_ergonomics_reviewer rerun

Date: 2026-07-17
Decision: pass-with-followups

## Findings

No blocking findings for the current legacy-owned, no-bridge file/state/JSON seam.

- Severity: low
- Location: rewrite-in-rust/bootstrap/lyric_matching_file_contract_core.md:22
- Issue: User-visible console and task-log text remain intentionally outside this unit. That is acceptable while Python remains the runtime owner, but it is a promotion-time product risk because lyric matching warnings, no-match reasons, difference displays, and summaries are user-visible in GUI/Web workflows through stdout/stderr capture and exported logs.
- Evidence: The bootstrap excludes exact console output, SmartHighlighter display rendering, GUI/Web/CLI routing, and production Rust routing at `rewrite-in-rust/bootstrap/lyric_matching_file_contract_core.md:22`. The dependency record keeps console display text and production bridge wiring outside the independent seam at `rewrite-in-rust/dependencies/lyric_matching_file_contract_core.yaml:41`. Legacy Python prints missing lyric, empty-ASR, diff, no-match, and summary messages at `inference/LyricFA/tools/lyric_matcher.py:110`, `inference/LyricFA/tools/lyric_matcher.py:143`, `inference/LyricFA/tools/lyric_matcher.py:158`, `inference/LyricFA/tools/lyric_matcher.py:207`, and `inference/LyricFA/tools/lyric_matcher.py:227`. Web tasks redirect stdout/stderr into SocketIO log entries at `web_task_manager.py:209`, and Web API docs define pipeline log entry fields at `docs/web-api.md:369`. The Rust module documents that console display text and GUI/Web/CLI callers remain Python-owned at `rewrite-in-rust/rust/crates/v2m-core/src/lyric_matching_file.rs:3`.
- Required fix: None before this unit remains `reimplemented` and legacy-owned. Before any production bridge, define either exact message parity or an accepted message-change policy for missing lyrics, empty ASR skips, no-match reasons, diff-threshold displays, summaries, WebSocket log entries, and exported ASR matching logs.

- Severity: low
- Location: rewrite-in-rust/bootstrap/lyric_matching_file_contract_core.md:30
- Issue: Multi-file discovery, ordering, and output workflow naming are not proven by this file-contract seam. The current fixtures cover stable single-file outcomes and JSON schema, but not full `glob.glob` directory behavior, extension filtering, duplicate stems, cross-platform path separator expectations, or downstream ASR matching log file naming.
- Evidence: The bootstrap explicitly excludes full directory glob ordering and production routing at `rewrite-in-rust/bootstrap/lyric_matching_file_contract_core.md:30`. Legacy `execute()` creates the JSON directory, discovers `*.txt` and `*.lab`, stores `total_files`, processes in `glob.glob` order, and prints a summary at `inference/LyricFA/tools/lyric_matcher.py:236`. Rust instead accepts caller-supplied lyric and lab path lists in `execute_with_paths` at `rewrite-in-rust/rust/crates/v2m-core/src/lyric_matching_file.rs:193`, preserving the no-bridge seam but leaving path discovery to a future caller. The current primary workflow also writes `<output_key>_asr_match_log.txt` when ASR match logs are requested at `inference/pipeline/auto_lyric_hybrid.py:442`; this unit only owns result JSON payloads, whose three fields are documented at `rewrite-in-rust/dependencies/lyric_matching_file_contract_core.yaml:12` and implemented at `rewrite-in-rust/rust/crates/v2m-core/src/lyric_matching_file.rs:228`.
- Required fix: Before replacing any Python-facing directory or workflow boundary, decide whether Python keeps path discovery/log naming ownership or add promotion fixtures for extension filtering, glob ordering, duplicate stem handling, output filename conventions, JSON directory creation, and downstream log/export consumers.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_lyric_matching_file_contract_core.py`: passed.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml lyric_matching_file`: passed; 1 lyric matching file test passed, 96 filtered in `v2m-core`, and 0 matching tests in `v2m_quant_bridge`.
- `git diff --check`: passed.
- `rg -n "LyricMatchingFilePipeline|lyric_matching_file|v2m_core|v2m-core" inference application gui web_server.py web_task_manager.py scripts tests rewrite-in-rust/rust/crates --glob '!rewrite-in-rust/rust/target/**'`: inspected; matches were limited to the Rust workspace and quantization Rust crate naming. No production GUI/Web/CLI or Python inference caller uses the Rust lyric matching file-contract seam.

## Residual Risk

The fixture table covers filename extraction, lab-to-lyric mapping, missing lyric de-duplication, successful lab processing, empty-ASR skip, no-match JSON output, zh/non-zh diff routing, negative threshold behavior, JSON schema, and single-file execute state at `rewrite-in-rust/fixtures/lyric_matching_file_contract_core.jsonl:1`. The Python checker exercises those cases through a fake matcher and the real `LyricMatcher.save_to_json` at `rewrite-in-rust/bootstrap/check_lyric_matching_file_contract_core.py:60`, and the Rust test consumes the same fixture table at `rewrite-in-rust/rust/crates/v2m-core/src/lyric_matching_file.rs:464`.

What remains unproven is intentionally outside this unit: language processor payloads, exact console/log strings, full multi-file discovery behavior, GUI/Web/CLI routing, exported ASR matching log text, structured skip reasons for recovery, and Python-facing bridge error mapping.

## Promotion Note

This role does not block coordinator state update for the `product_ergonomics_reviewer` review result. The coordinator can record this role as complete with follow-ups for the current `reimplemented` no-bridge unit, but must not mark the manifest verified or promote a runtime owner change from this report alone. Promotion requires a separate record and tests for user-visible logs/messages, path discovery, recovery/error mapping, workflow file naming, downstream JSON/log consumers, and rollback.
