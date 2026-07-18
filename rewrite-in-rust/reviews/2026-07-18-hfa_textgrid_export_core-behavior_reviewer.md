# hfa_textgrid_export_core - behavior_reviewer

Date: 2026-07-18
Decision: fail

Unit: `hfa_textgrid_export_core`
Role: `behavior_reviewer`

## Findings

- Severity: low
- Location: rewrite-in-rust/rust/crates/v2m-core/src/hfa_textgrid_export.rs:399
- Issue: POSIX paths with exactly two leading slashes do not preserve Python
  `pathlib` path text. Python `Exporter.save_textgrids` keeps exactly-two-slash
  roots when planning both `output_folder / "TextGrid" / name` and
  `wav_path.parent / "TextGrid" / name`
  (`inference/HubertFA/tools/export_tool.py:27`,
  `inference/HubertFA/tools/export_tool.py:30`). Rust normalizes paths by
  rebuilding from `Path::components()` while dropping `CurDir`, which collapses
  `//out` and `//a/b/song.wav` to `/out` and `/a/b/song.wav`
  (`rewrite-in-rust/rust/crates/v2m-core/src/hfa_textgrid_export.rs:370`,
  `rewrite-in-rust/rust/crates/v2m-core/src/hfa_textgrid_export.rs:399`).
- Evidence: A monkeypatched legacy exporter probe produced
  `//out/TextGrid/song.TextGrid`, `//TextGrid/song.TextGrid`,
  `//a/b/TextGrid/song.TextGrid`, and `//TextGrid/song.TextGrid`. A public Rust
  `plan_textgrid_export` probe for the same inputs produced
  `/out/TextGrid/song.TextGrid`, `/TextGrid/song.TextGrid`,
  `/a/b/TextGrid/song.TextGrid`, and `/TextGrid/song.TextGrid`.
- Required fix: Preserve Python's exactly-two-leading-slash POSIX root behavior
  in the Rust path projection, or explicitly narrow the public path contract to
  exclude that `pathlib` edge and add fixture coverage for the chosen contract.

## Behavior Evidence

The main TextGrid serialization seam matches the documented fixture contract.
The Python reference constructs `words` then `phones`, clamps phone starts with
`max(0, phoneme.start)`, appends tiers in that order, then writes via
`textgrid.TextGrid.write` (`inference/HubertFA/tools/export_tool.py:13`,
`inference/HubertFA/tools/export_tool.py:20`,
`inference/HubertFA/tools/export_tool.py:22`,
`inference/HubertFA/tools/export_tool.py:24`,
`inference/HubertFA/tools/export_tool.py:33`). Vendored `textgrid==1.6.1`
sorts interval inserts, rejects invalid/overlapping intervals, fills blank
gaps, doubles quotes, and writes long TextGrid text through UTF-8
(`third_party/sources/textgrid-1.6.1/textgrid/textgrid.py:453`,
`third_party/sources/textgrid-1.6.1/textgrid/textgrid.py:459`,
`third_party/sources/textgrid-1.6.1/textgrid/textgrid.py:519`,
`third_party/sources/textgrid-1.6.1/textgrid/textgrid.py:757`,
`third_party/sources/textgrid-1.6.1/textgrid/textgrid.py:770`,
`third_party/sources/textgrid-1.6.1/textgrid/textgrid.py:787`).

The Rust planner mirrors those behavior-visible pieces for the covered cases:
it builds word and phone interval vectors in prediction order, clamps phone
starts to Python integer text `0`, sorts/rejects intervals, fills gaps, doubles
quotes only, stores planned UTF-8 content, returns Python exception type/message
strings, and carries completed side-effect plans on later failures
(`rewrite-in-rust/rust/crates/v2m-core/src/hfa_textgrid_export.rs:99`,
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_textgrid_export.rs:146`,
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_textgrid_export.rs:163`,
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_textgrid_export.rs:199`,
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_textgrid_export.rs:250`,
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_textgrid_export.rs:346`,
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_textgrid_export.rs:436`,
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_textgrid_export.rs:480`).

The 15-case fixture table covers exact UTF-8 TextGrid bytes for empty, gapped,
contiguous, sorted, quoted, newline, Unicode, negative phone-start, and Python
float text cases; invalid zero-duration, overlap, out-of-range, and empty-name
errors; partial plans after a prior write; duplicate basenames; output-folder,
wav-parent, empty-output, current-directory, suffix-name, prediction isolation,
and repeated-call behavior (`rewrite-in-rust/fixtures/hfa_textgrid_export_core.jsonl:1`,
`rewrite-in-rust/fixtures/hfa_textgrid_export_core.jsonl:3`,
`rewrite-in-rust/fixtures/hfa_textgrid_export_core.jsonl:4`,
`rewrite-in-rust/fixtures/hfa_textgrid_export_core.jsonl:5`,
`rewrite-in-rust/fixtures/hfa_textgrid_export_core.jsonl:8`,
`rewrite-in-rust/fixtures/hfa_textgrid_export_core.jsonl:9`,
`rewrite-in-rust/fixtures/hfa_textgrid_export_core.jsonl:10`,
`rewrite-in-rust/fixtures/hfa_textgrid_export_core.jsonl:11`,
`rewrite-in-rust/fixtures/hfa_textgrid_export_core.jsonl:13`,
`rewrite-in-rust/fixtures/hfa_textgrid_export_core.jsonl:15`).

Rollback remains intact. The manifest keeps the unit `reimplemented`,
`current_owner: legacy`, and says to keep `Exporter.save_textgrids` and
`textgrid 1.6.1` as runtime owners
(`rewrite-in-rust/manifest.yaml:1601`,
`rewrite-in-rust/manifest.yaml:1603`,
`rewrite-in-rust/manifest.yaml:1605`,
`rewrite-in-rust/manifest.yaml:1626`). Record 0093 confirms no production export
route, filesystem write, API artifact-copying path, GUI, Web, or model-inference
code changed (`rewrite-in-rust/records/0093-implement-hfa-textgrid-export-core.md:12`).

## Checks

- `PYTHONDONTWRITEBYTECODE=1 uv run python rewrite-in-rust/bootstrap/check_hfa_textgrid_export_core.py`: passed, validated 15 Python-generated fixtures against the real monkeypatched `Exporter.save_textgrids`.
- `CARGO_TARGET_DIR=/tmp/v2m-hfa-textgrid-review-target cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml --locked hfa_textgrid_export_core -- --nocapture`: passed, 1 focused Rust fixture-parity test; 116 `v2m-core` tests and 5 quant bridge tests filtered out.
- `PYTHONDONTWRITEBYTECODE=1 uv run python - <<'PY' ...`: targeted monkeypatched legacy exporter probe confirmed `pathlib` preserves exactly-two-leading-slash roots in planned TextGrid directory/file paths.
- `rustc --edition=2024 -L dependency=/tmp/v2m-hfa-textgrid-review-target/debug/deps --extern v2m_core=... -o /tmp/v2m-hfa-textgrid-public-probe - <<'RS' ...`: targeted public Rust planner probe confirmed the current planner collapses those same roots to one leading slash.
- Source inspection covered the requested rewrite docs, records 0074 and 0093, dependency/bootstrap records, Python reference files, vendored `textgrid` source, Rust `hfa_word`/`hfa_textgrid_export` code, checker, and fixture table.

## Residual Risk

This review covers only behavior parity for the selected in-memory TextGrid
planned-export seam. It does not re-review dependency/bootstrap adequacy,
structured error/tracing design beyond public exception projections, export
dispatch, product ergonomics, or bridge promotion. Real filesystem IO errors,
status printing, artifact copying, and streaming partial file contents on
exception remain intentionally legacy-owned. Non-UTF-8 OS path display and a
fully exhaustive Python/Rust finite-float search are not fixture-proven.

## Promotion Note

This `behavior_reviewer` pass is not ready for coordinator state update as a
passing behavior gate. Keep `hfa_textgrid_export_core` at `reimplemented` until
the double-leading-slash path contract is either fixed with a fixture or
explicitly scoped out by the coordinator and recorded. The production rollback
route remains keeping Python `Exporter.save_textgrids` and `textgrid 1.6.1` as
runtime owners.
