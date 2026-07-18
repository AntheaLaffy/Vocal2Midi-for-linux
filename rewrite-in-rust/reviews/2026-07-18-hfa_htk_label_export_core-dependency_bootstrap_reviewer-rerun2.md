# hfa_htk_label_export_core - dependency_bootstrap_reviewer rerun2

## Findings

No findings.

## Decision

Date: 2026-07-18
Decision: pass

Manifest unit boundary: confirmed. The unit should stay split.

Record 0090 changes only the fixture-bound current-directory path projection:
`output_folder="."`, `output_folder="./"`, and wav-parent mode with
`wav_path="./song.wav"` now render planned paths without a leading `./`
(`rewrite-in-rust/records/0090-fix-hfa-htk-current-directory-paths.md:7`,
`rewrite-in-rust/records/0090-fix-hfa-htk-current-directory-paths.md:19`,
`rewrite-in-rust/records/0090-fix-hfa-htk-current-directory-paths.md:29`).
It does not add a crate, bridge, filesystem route, or broader export
responsibility.

The record-0074 split remains valid. HTK planned files, TextGrid serialization,
and export dispatch are still separate units, with HTK owning only in-memory
planned files, path scaling, and cumulative prediction state
(`rewrite-in-rust/records/0074-split-hfa-g2p-config-export-lifecycle.md:25`).
The manifest still marks `hfa_htk_label_export_core` `reimplemented` and
`confirmed`, keeps source refs to `export_tool.py` and `infer_base.py`, and
keeps rollback on Python `Exporter.save_htk`
(`rewrite-in-rust/manifest.yaml:1566`, `rewrite-in-rust/manifest.yaml:1572`,
`rewrite-in-rust/manifest.yaml:1592`).

## Bootstrap Evidence

- Capability coverage remains complete for dependency/bootstrap scope. The
  dependency record names the selected capabilities as HTK rendering, cumulative
  prediction state, and path planning
  (`rewrite-in-rust/dependencies/hfa_htk_label_export_core.yaml:3`), and now
  points to 17 Python 3.12 cases generated from real `Exporter.save_htk`
  (`rewrite-in-rust/dependencies/hfa_htk_label_export_core.yaml:20`).
- Fixture adequacy covers the record-0090 path behavior. The fixture file has
  17 rows, and rows 13-15 pin `output_folder="."`, `output_folder="./"`, and
  `wav_path="./song.wav"` to `HTK/Phones/...` and `HTK/Words/...` paths without
  a leading `./`
  (`rewrite-in-rust/fixtures/hfa_htk_label_export_core.jsonl:13`,
  `rewrite-in-rust/fixtures/hfa_htk_label_export_core.jsonl:14`,
  `rewrite-in-rust/fixtures/hfa_htk_label_export_core.jsonl:15`).
- The checker uses the real Python seam with monkeypatched `Path.mkdir` and
  `open`, so the fixtures capture ordered planned side effects without writing
  production paths
  (`rewrite-in-rust/bootstrap/check_hfa_htk_label_export_core.py:100`).
- The Rust implementation matches the narrowed dependency decision. It exposes
  only an in-memory planner, applies the pathlib-style `.` component projection,
  and remains outside production routing
  (`rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:1`,
  `rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:87`,
  `rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:182`).
- The no-crate/no-bridge decision remains justified. `save_htk` uses project
  data, standard path composition, string rendering, and Python numeric
  conversion behavior; `textgrid` is imported for `save_textgrids`, not this
  HTK seam (`inference/HubertFA/tools/export_tool.py:2`,
  `inference/HubertFA/tools/export_tool.py:35`). The dependency record has no
  bridge dependencies and explicitly chooses a hand-written replacement for HTK
  content/path planning
  (`rewrite-in-rust/dependencies/hfa_htk_label_export_core.yaml:16`,
  `rewrite-in-rust/dependencies/hfa_htk_label_export_core.yaml:27`). The crate
  dependency list adds no HTK or TextGrid crate
  (`rewrite-in-rust/rust/crates/v2m-core/Cargo.toml:12`).
- First-layer source coverage is adequate. The direct behavior source is
  project-owned `Exporter.save_htk`, while `InferenceBase.export` is only caller
  context that delegates to `Exporter(...).export(...)`
  (`inference/HubertFA/tools/export_tool.py:35`,
  `inference/HubertFA/tools/infer_base.py:240`).
- Rollback remains clear: keep Python `Exporter.save_htk` as runtime owner,
  including cumulative buffers
  (`rewrite-in-rust/bootstrap/hfa_htk_label_export_core.md:34`,
  `rewrite-in-rust/records/0090-fix-hfa-htk-current-directory-paths.md:58`).

## Checks

- `uv run python --version`: passed, Python 3.12.13.
- `wc -l rewrite-in-rust/fixtures/hfa_htk_label_export_core.jsonl`: passed,
  confirmed 17 JSONL rows.
- `jq -r '[input_line_number, .case_id, ...] | @tsv' rewrite-in-rust/fixtures/hfa_htk_label_export_core.jsonl`:
  passed, confirmed rows 13-15 cover the three current-directory projections and
  expected `HTK/...` paths.
- `uv run python -B rewrite-in-rust/bootstrap/check_hfa_htk_label_export_core.py`:
  passed, validated 17 fixtures against Python 3.12 legacy behavior.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_htk_label_export_core`:
  passed, one focused Rust fixture-parity test.
- `uv run python -B scripts/audit_vendored_sources.py`: passed, source audit
  reported 135 Python packages, 41 native-extension packages, 269 foreign
  runtime native binaries, and 0 third_party binary artifacts.
- `rg -n "hfa_htk_export|plan_htk_label_export|save_htk|Exporter\\(|textgrid|bridge|PyO3|pyo3|subprocess" ...`:
  inspected HTK source, Rust exposure, and bridge/crate references; only the
  unrelated existing quantization bridge appears in the Rust workspace.

## Residual Risk

This review covers dependency/bootstrap scope only. It does not re-approve
behavior, data/algorithm, error/tracing, Rust style, architecture, or product
ergonomics. Filesystem IO errors, status printing, TextGrid serialization,
format dispatch, and production routing remain intentionally legacy-owned.

## Promotion Note

This `dependency_bootstrap_reviewer` rerun2 is ready for coordinator state
update for this role and does not block promotion. The unit as a whole still
depends on the requested required review set after record 0090 before it can be
marked `verified`.
