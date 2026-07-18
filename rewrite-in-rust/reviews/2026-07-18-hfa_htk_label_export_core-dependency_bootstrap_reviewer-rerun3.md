# hfa_htk_label_export_core - dependency_bootstrap_reviewer rerun3

## Findings

No findings.

## Decision

Date: 2026-07-18
Decision: pass

Unit: `hfa_htk_label_export_core`
Role: `dependency_bootstrap_reviewer`
Rerun after: `rewrite-in-rust/records/0091-fix-hfa-htk-pathlib-suffix-names.md`

Manifest unit boundary: confirmed. The unit should stay split.

Record 0091 changed only the Python-pathlib label-name projection and expanded
fixtures from 17 to 20 Python-generated rows. The new rows lock
`wav_path="song."` to `song..lab`, `wav_path=".."` to `...lab`, and a compact
suffix matrix for all-dot, hidden, trailing-dot, multi-suffix, and `..a` names
(`rewrite-in-rust/records/0091-fix-hfa-htk-pathlib-suffix-names.md:19`,
`rewrite-in-rust/records/0091-fix-hfa-htk-pathlib-suffix-names.md:32`,
`rewrite-in-rust/fixtures/hfa_htk_label_export_core.jsonl:16`,
`rewrite-in-rust/fixtures/hfa_htk_label_export_core.jsonl:17`,
`rewrite-in-rust/fixtures/hfa_htk_label_export_core.jsonl:18`). This does not
add a crate, bridge, filesystem route, TextGrid responsibility, or export
dispatch responsibility.

## Bootstrap Evidence

- Capability coverage remains complete for dependency/bootstrap scope. The
  dependency record names the required capabilities as HTK label rendering,
  cumulative prediction state, and path planning
  (`rewrite-in-rust/dependencies/hfa_htk_label_export_core.yaml:3`), which
  correspond to `Exporter.save_htk` time rendering, raw label accumulation,
  `Path.with_suffix(".lab").name`, mkdir, and UTF-8 write behavior
  (`inference/HubertFA/tools/export_tool.py:35`).
- The record-0074 split still holds. HTK planned files own only paths, scaling,
  and cumulative prediction state; TextGrid serialization and export dispatch
  remain separate units (`rewrite-in-rust/records/0074-split-hfa-g2p-config-export-lifecycle.md:25`,
  `rewrite-in-rust/records/0074-split-hfa-g2p-config-export-lifecycle.md:27`,
  `rewrite-in-rust/records/0074-split-hfa-g2p-config-export-lifecycle.md:29`).
- The no-crate/no-bridge decision remains justified. The dependency record has
  `bridge_dependencies: []`, names a hand-written HTK content/path replacement,
  and keeps filesystem effects plus dispatch legacy-owned
  (`rewrite-in-rust/dependencies/hfa_htk_label_export_core.yaml:19`,
  `rewrite-in-rust/dependencies/hfa_htk_label_export_core.yaml:27`,
  `rewrite-in-rust/dependencies/hfa_htk_label_export_core.yaml:34`). The
  bootstrap doc also says no bridge or new crate
  (`rewrite-in-rust/bootstrap/hfa_htk_label_export_core.md:24`), and the
  `v2m-core` dependency list adds no HTK/TextGrid crate
  (`rewrite-in-rust/rust/crates/v2m-core/Cargo.toml:12`).
- First-layer source coverage is adequate. The direct behavior source is
  project-owned `Exporter.save_htk`; `InferenceBase.export` is caller context
  that delegates to `Exporter(...).export(...)`
  (`inference/HubertFA/tools/export_tool.py:35`,
  `inference/HubertFA/tools/infer_base.py:240`). The `textgrid` import is used by
  `save_textgrids`, not the selected HTK seam
  (`inference/HubertFA/tools/export_tool.py:2`,
  `inference/HubertFA/tools/export_tool.py:11`).
- Fixture adequacy is now aligned with the 20-case state. The checker runs the
  real Python `Exporter.save_htk` with monkeypatched `Path.mkdir` and `open`
  sinks, then compares every JSONL case to legacy Python output
  (`rewrite-in-rust/bootstrap/check_hfa_htk_label_export_core.py:100`,
  `rewrite-in-rust/bootstrap/check_hfa_htk_label_export_core.py:164`). The
  manifest, dependency record, bootstrap doc, and record 0091 all describe the
  current 20-case/pathlib suffix coverage
  (`rewrite-in-rust/manifest.yaml:1582`,
  `rewrite-in-rust/dependencies/hfa_htk_label_export_core.yaml:22`,
  `rewrite-in-rust/bootstrap/hfa_htk_label_export_core.md:26`,
  `rewrite-in-rust/records/0091-fix-hfa-htk-pathlib-suffix-names.md:32`).
- The Rust module matches the dependency decision: an in-memory planner outside
  production routing, with Python-style current-directory path normalization and
  Python-style suffix-name projection
  (`rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:1`,
  `rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:87`,
  `rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:182`,
  `rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:213`).
- Rollback remains clear: keep Python `Exporter.save_htk` as runtime owner,
  including cumulative buffers (`rewrite-in-rust/manifest.yaml:1593`,
  `rewrite-in-rust/bootstrap/hfa_htk_label_export_core.md:35`,
  `rewrite-in-rust/records/0091-fix-hfa-htk-pathlib-suffix-names.md:64`).

## Checks

- `PYTHONDONTWRITEBYTECODE=1 uv run python --version && PYTHONDONTWRITEBYTECODE=1 uv run python rewrite-in-rust/bootstrap/check_hfa_htk_label_export_core.py`: passed; Python 3.12.13 validated 20 fixtures.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_htk_label_export_core`: passed; one focused Rust fixture-parity test, 115 filtered in `v2m-core`, 5 filtered in `v2m-quant-bridge`.
- `PYTHONDONTWRITEBYTECODE=1 uv run python - <<'PY' ...`: passed; parsed 20 JSONL rows, confirmed the three pathlib suffix edge case ids and expected `song..lab`, `...lab`, `song...lab`, `....lab`, `.hidden..lab`, `song.tar.lab`, and `..lab` outputs; parsed manifest and dependency YAML.
- `PYTHONDONTWRITEBYTECODE=1 uv run python scripts/audit_vendored_sources.py`: passed; reported 135 Python packages, 41 native-extension packages, 269 foreign runtime native binaries, and 0 third_party binary artifacts.
- `rg -n "hfa_htk_export|plan_htk_label_export|HfaHtk" rewrite-in-rust/rust inference application web_server.py web_task_manager.py scripts tests -g '!target' -S`: inspected HTK Rust exposure; hits are limited to `v2m-core::hfa_htk_export` and its `lib.rs` module export.
- `rg -n "textgrid|PyO3|pyo3|htk|HTK|bridge_dependencies|bridge|subprocess" ...`: inspected HTK dependency/bootstrap docs, the HTK Rust module, Cargo dependencies, and Python source refs; found no HTK-specific bridge or crate dependency.

## Residual Risk

This review covers dependency/bootstrap scope only. It does not re-approve
behavior, data/algorithm, error/tracing, Rust style, architecture, or product
ergonomics. Filesystem IO errors, status printing, TextGrid serialization,
format dispatch, and production routing remain intentionally legacy-owned.

The 20 fixtures cover the known Python 3.12 pathlib suffix cases that triggered
record 0091, but this report does not claim exhaustive compatibility for every
possible platform path spelling outside the selected public fixture surface.

## Promotion Note

This `dependency_bootstrap_reviewer` rerun3 is ready for coordinator state
update for this role and does not block promotion. The coordinator should not
mark the whole unit `verified` from this report alone; record 0091 still requires
the appropriate fresh behavior evidence after the pathlib suffix fix.
