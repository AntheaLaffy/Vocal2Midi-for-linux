# hfa_textgrid_export_core - behavior_reviewer rerun

Unit: `hfa_textgrid_export_core`
Role: `behavior_reviewer`
Date: 2026-07-18

## Findings

No findings.

The record-0094 double-leading-slash fix is behavior-compatible with Python
`pathlib` for the covered public seam. The legacy exporter plans TextGrid paths
through `self.output_folder / "TextGrid" / wav_path.with_suffix(...).name` or
`wav_path.parent / "TextGrid" / ...` before `Path.mkdir` and `tg.write`
(`inference/HubertFA/tools/export_tool.py:27`,
`inference/HubertFA/tools/export_tool.py:30`,
`inference/HubertFA/tools/export_tool.py:32`). The Rust path projection now
normalizes lexical `.` components while preserving exactly-two-leading-slash
roots and the `//song.wav` parent edge
(`rewrite-in-rust/rust/crates/v2m-core/src/hfa_textgrid_export.rs:370`,
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_textgrid_export.rs:394`,
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_textgrid_export.rs:408`,
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_textgrid_export.rs:418`).
Targeted legacy and Rust probes both produced `//out/TextGrid`,
`//a/b/TextGrid`, and `//TextGrid` for the three record-0094 cases.

The main TextGrid byte contract still matches the Python source and vendored
`textgrid==1.6.1` writer subset. Python constructs `words` then `phones`, clamps
phone starts with `max(0, phoneme.start)`, and writes via `TextGrid.write`
(`inference/HubertFA/tools/export_tool.py:15`,
`inference/HubertFA/tools/export_tool.py:20`,
`inference/HubertFA/tools/export_tool.py:22`,
`inference/HubertFA/tools/export_tool.py:24`,
`inference/HubertFA/tools/export_tool.py:33`). The vendored writer rejects
zero-duration intervals, sorts/rejects interval inserts, fills blank gaps,
doubles quotes, and writes UTF-8 long TextGrid bytes
(`third_party/sources/textgrid-1.6.1/textgrid/textgrid.py:186`,
`third_party/sources/textgrid-1.6.1/textgrid/textgrid.py:453`,
`third_party/sources/textgrid-1.6.1/textgrid/textgrid.py:519`,
`third_party/sources/textgrid-1.6.1/textgrid/textgrid.py:757`,
`third_party/sources/textgrid-1.6.1/textgrid/textgrid.py:787`).
The Rust planner mirrors those behavior-visible pieces in tier construction,
validation/sorting, gap filling, path/name projection, float text, quote
escaping, Python exception projection, partial plans, and repeated-call
statelessness
(`rewrite-in-rust/rust/crates/v2m-core/src/hfa_textgrid_export.rs:99`,
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_textgrid_export.rs:146`,
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_textgrid_export.rs:199`,
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_textgrid_export.rs:250`,
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_textgrid_export.rs:346`,
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_textgrid_export.rs:435`,
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_textgrid_export.rs:454`,
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_textgrid_export.rs:476`).

The current fixture set covers the requested behavior scope: exact UTF-8
TextGrid content, tier order, gaps, sorting, invalid intervals, phone
`max(0,start)`, quotes/newlines/Unicode, Python float text, current-dir and
pathlib suffix behavior, duplicate basenames, partial plans, repeated calls,
prediction isolation, and the three double-slash regressions
(`rewrite-in-rust/fixtures/hfa_textgrid_export_core.jsonl:1`,
`rewrite-in-rust/fixtures/hfa_textgrid_export_core.jsonl:3`,
`rewrite-in-rust/fixtures/hfa_textgrid_export_core.jsonl:5`,
`rewrite-in-rust/fixtures/hfa_textgrid_export_core.jsonl:8`,
`rewrite-in-rust/fixtures/hfa_textgrid_export_core.jsonl:9`,
`rewrite-in-rust/fixtures/hfa_textgrid_export_core.jsonl:10`,
`rewrite-in-rust/fixtures/hfa_textgrid_export_core.jsonl:11`,
`rewrite-in-rust/fixtures/hfa_textgrid_export_core.jsonl:13`,
`rewrite-in-rust/fixtures/hfa_textgrid_export_core.jsonl:15`,
`rewrite-in-rust/fixtures/hfa_textgrid_export_core.jsonl:16`,
`rewrite-in-rust/fixtures/hfa_textgrid_export_core.jsonl:17`,
`rewrite-in-rust/fixtures/hfa_textgrid_export_core.jsonl:18`).

Rollback and production isolation remain intact. The manifest still keeps the
unit `reimplemented`, with `current_owner: legacy`, and says to keep
`Exporter.save_textgrids` and `textgrid 1.6.1` as runtime owners
(`rewrite-in-rust/manifest.yaml:1601`,
`rewrite-in-rust/manifest.yaml:1603`,
`rewrite-in-rust/manifest.yaml:1605`,
`rewrite-in-rust/manifest.yaml:1627`). Record 0093 says no production export
route, filesystem write, API artifact-copying path, GUI, Web, or
model-inference code changed
(`rewrite-in-rust/records/0093-implement-hfa-textgrid-export-core.md:12`), and
record 0094 scoped the path fix to this unit with no production caller route
change (`rewrite-in-rust/records/0094-fix-hfa-textgrid-double-slash-paths.md:18`,
`rewrite-in-rust/records/0094-fix-hfa-textgrid-double-slash-paths.md:23`).

## Decision

Decision: pass

## Checks

- `PYTHONDONTWRITEBYTECODE=1 uv run python rewrite-in-rust/bootstrap/check_hfa_textgrid_export_core.py`: passed; validated 18 Python-generated fixtures against the real monkeypatched `Exporter.save_textgrids`.
- `CARGO_TARGET_DIR=/tmp/v2m-hfa-textgrid-rerun-target cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml --locked hfa_textgrid_export_core -- --nocapture`: passed; 1 focused Rust fixture-parity test passed, 116 `v2m-core` tests and 5 quant bridge tests filtered out.
- `PYTHONDONTWRITEBYTECODE=1 uv run python - <<'PY' ...`: targeted legacy probe for the three record-0094 double-slash cases produced `//out/TextGrid`, `//a/b/TextGrid`, and `//TextGrid` directory paths with matching file paths.
- `rustc --edition=2024 -L dependency=/tmp/v2m-hfa-textgrid-rerun-target/debug/deps --extern v2m_core=... -o /tmp/v2m-hfa-textgrid-rerun-probe - <<'RS' ...`: targeted public Rust planner probe produced the same three double-slash directory/file paths.
- `rg -n "hfa_textgrid_export|plan_textgrid_export|HfaTextGrid|save_textgrids|TextGrid" rewrite-in-rust/rust inference application web_server.py web_task_manager.py scripts tests -g '!target'`: inspected; production references still route through legacy Python `save_textgrids`; Rust references are confined to the independent `v2m-core` workspace.
- `rg -n "std::fs|File::|create_dir|write_all|println!|eprintln!|dbg!|log::|tracing::|pyo3|PyO3|subprocess" rewrite-in-rust/rust/crates/v2m-core/src/hfa_textgrid_export.rs rewrite-in-rust/rust/Cargo.toml rewrite-in-rust/rust/crates/v2m-core/Cargo.toml`: no matches; this unit adds no filesystem, logging, PyO3, or subprocess behavior.
- `git diff --check -- rewrite-in-rust/reviews/2026-07-18-hfa_textgrid_export_core-behavior_reviewer-rerun.md rewrite-in-rust/rust/crates/v2m-core/src/hfa_textgrid_export.rs rewrite-in-rust/bootstrap/check_hfa_textgrid_export_core.py rewrite-in-rust/fixtures/hfa_textgrid_export_core.jsonl rewrite-in-rust/bootstrap/hfa_textgrid_export_core.md rewrite-in-rust/dependencies/hfa_textgrid_export_core.yaml rewrite-in-rust/records/0094-fix-hfa-textgrid-double-slash-paths.md`: passed.

## Residual Risk

This review covers behavior parity for the selected in-memory TextGrid
planned-export seam only. It does not re-review dependency/bootstrap adequacy,
error/tracing design beyond behavior-visible error projections, Rust style,
architecture, product ergonomics, export dispatch, API artifact copying, status
printing, real filesystem IO failures, or bridge promotion. Non-UTF-8 OS path
display and exhaustive finite-float rendering beyond the fixture matrix remain
unproven.

## Coordinator Readiness

This behavior rerun is ready for coordinator state update and no longer blocks
promotion on the record-0094 double-slash issue. The coordinator should combine
this passing behavior evidence with the existing required review roles before
changing the manifest state; this report does not itself mark the unit verified.
