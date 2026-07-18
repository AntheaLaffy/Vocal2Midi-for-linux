# hfa_textgrid_export_core - dependency_bootstrap_reviewer

## Findings

No findings.

## Decision

Date: 2026-07-18
Decision: pass

Unit: `hfa_textgrid_export_core`
Role: `dependency_bootstrap_reviewer`

Manifest unit boundary: confirmed. The unit should stay split.

The selected seam remains an in-memory planner for
`Exporter.save_textgrids`, not a production export route. The manifest keeps
runtime ownership in legacy Python (`rewrite-in-rust/manifest.yaml:1603`,
`rewrite-in-rust/manifest.yaml:1605`), and record 0093 states that no production
export route, filesystem write, API copy path, GUI, Web, or model-inference code
changed (`rewrite-in-rust/records/0093-implement-hfa-textgrid-export-core.md:12`).

## Bootstrap Evidence

- Capability coverage is complete for dependency/bootstrap scope. The dependency
  record names tier construction, textgrid 1.6.1 serialization, and path
  planning (`rewrite-in-rust/dependencies/hfa_textgrid_export_core.yaml:3`).
  These map directly to `Exporter.save_textgrids` constructing two tiers,
  clamping phone starts, planning `TextGrid` paths, creating directories, and
  calling `tg.write` (`inference/HubertFA/tools/export_tool.py:11`).
- The no-crate/no-bridge decision is justified. The seam record has
  `bridge_dependencies: []`
  (`rewrite-in-rust/dependencies/hfa_textgrid_export_core.yaml:16`), the
  bootstrap doc explicitly says to hand-write only the selected
  `TextGrid`/`IntervalTier` writer subset and not add a general TextGrid crate
  (`rewrite-in-rust/bootstrap/hfa_textgrid_export_core.md:13`), and the Rust
  module documents that Python still owns directory creation, file writes,
  status printing, artifact copying, dispatch, and production routing
  (`rewrite-in-rust/rust/crates/v2m-core/src/hfa_textgrid_export.rs:1`).
- First-layer source coverage is adequate. `pyproject.toml` declares
  `textgrid` (`pyproject.toml:33`), `uv.lock` pins `textgrid==1.6.1` with an
  sdist (`uv.lock:4074`), and the vendored source manifest points to
  `third_party/sources/textgrid-1.6.1`
  (`third_party/sources/manifest.json:870`). The package source files present
  for the imported layer are `__init__.py`, `exceptions.py`, and `textgrid.py`;
  `__init__.py` exports `TextGrid`, `IntervalTier`, `Interval`, `PointTier`,
  `Point`, and `MLF` (`third_party/sources/textgrid-1.6.1/textgrid/__init__.py:1`).
- No targeted transitive expansion is needed for this unit. The package setup
  declares only the `textgrid` package and no install requirements
  (`third_party/sources/textgrid-1.6.1/setup.py:3`), while the selected source
  imports only standard-library modules plus local `TextGridError`
  (`third_party/sources/textgrid-1.6.1/textgrid/textgrid.py:32`,
  `third_party/sources/textgrid-1.6.1/textgrid/textgrid.py:40`).
- The hand-written subset is narrow enough. The dependency record points at
  `export_tool.py`, vendored `textgrid.py`, vendored `exceptions.py`, and the
  verified HFA word state, then explains that direct package parity is broader
  than two `IntervalTier`s and exact writer bytes
  (`rewrite-in-rust/dependencies/hfa_textgrid_export_core.yaml:27`). The
  referenced vendored behavior covers interval validation and ordering
  (`third_party/sources/textgrid-1.6.1/textgrid/textgrid.py:186`,
  `third_party/sources/textgrid-1.6.1/textgrid/textgrid.py:453`), blank-gap
  insertion (`third_party/sources/textgrid-1.6.1/textgrid/textgrid.py:519`),
  quote doubling (`third_party/sources/textgrid-1.6.1/textgrid/textgrid.py:76`),
  and long TextGrid writing
  (`third_party/sources/textgrid-1.6.1/textgrid/textgrid.py:751`).
- Fixture adequacy matches the claimed dependency scope. The manifest,
  dependency record, and bootstrap doc all describe the current 15-case fixture
  surface (`rewrite-in-rust/manifest.yaml:1618`,
  `rewrite-in-rust/dependencies/hfa_textgrid_export_core.yaml:20`,
  `rewrite-in-rust/bootstrap/hfa_textgrid_export_core.md:20`). The checker uses
  the real legacy `Exporter.save_textgrids` with monkeypatched `Path.mkdir` and
  `codecs.open` sinks (`rewrite-in-rust/bootstrap/check_hfa_textgrid_export_core.py:100`),
  and asserts every JSONL case against legacy output
  (`rewrite-in-rust/bootstrap/check_hfa_textgrid_export_core.py:164`). The Rust
  module includes the same fixture file and compares the projected Rust plan to
  each expected row (`rewrite-in-rust/rust/crates/v2m-core/src/hfa_textgrid_export.rs:504`,
  `rewrite-in-rust/rust/crates/v2m-core/src/hfa_textgrid_export.rs:605`).
- Kept-legacy capabilities are correctly excluded. General TextGrid
  read/PointTier/MLF APIs and directory creation, writes, IO errors, status
  printing, artifact copying, and production routing remain legacy-owned
  (`rewrite-in-rust/dependencies/hfa_textgrid_export_core.yaml:35`). The caller
  dispatch/default behavior still lives in `InferenceBase.export`
  (`inference/HubertFA/tools/infer_base.py:240`), and API artifact copying is
  separate caller behavior (`inference/API/hfa_api.py:145`).
- Rollback is explicit and sufficient. The manifest says to keep
  `Exporter.save_textgrids` and textgrid 1.6.1 as runtime owners
  (`rewrite-in-rust/manifest.yaml:1626`), and record 0093 says reversal is
  removing the independent Rust module, fixture, checker, and manifest entries
  if the boundary is later re-cut
  (`rewrite-in-rust/records/0093-implement-hfa-textgrid-export-core.md:68`).
- The record-0074 split still holds. HTK label planning, TextGrid
  serialization/path planning, and export dispatch are separate confirmed units
  (`rewrite-in-rust/records/0074-split-hfa-g2p-config-export-lifecycle.md:25`,
  `rewrite-in-rust/records/0074-split-hfa-g2p-config-export-lifecycle.md:27`,
  `rewrite-in-rust/records/0074-split-hfa-g2p-config-export-lifecycle.md:29`).

## Checks

- `PYTHONDONTWRITEBYTECODE=1 uv run python --version`: passed; Python 3.12.13.
- `uv run python rewrite-in-rust/bootstrap/check_hfa_textgrid_export_core.py`:
  passed; validated 15 `hfa_textgrid_export_core` fixtures.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_textgrid_export_core`:
  passed; one focused Rust fixture-parity test passed, 116 `v2m-core` tests and
  5 `v2m-quant-bridge` tests filtered out.
- `rg --files third_party/sources/textgrid-1.6.1/textgrid | sort`: passed;
  listed `__init__.py`, `exceptions.py`, and `textgrid.py`.
- `rg -n "textgrid|TextGrid|pyo3|PyO3|subprocess|bridge_dependencies|bridge" ...`:
  inspected the unit dependency/bootstrap docs, Rust module, and Cargo files.
  No `hfa_textgrid_export_core` PyO3, subprocess, bridge dependency, or TextGrid
  crate was found; the only bridge hit in Cargo metadata is the existing
  unrelated `v2m-quant-bridge` workspace member.
- `python - <<'PY' ...`: parsed the fixture JSONL and confirmed there are 15
  case ids covering empty/gapped/sorted tiers, negative phone starts, text
  escaping, path planning, float text, invalid intervals, partial plans, and
  repeated calls.

## Residual Risk

This review covers dependency/bootstrap scope only. It does not re-approve
behavior, error/tracing, Rust style, architecture, or product ergonomics.
General TextGrid read/PointTier/MLF behavior, filesystem IO failures, status
printing, artifact copying, export dispatch, and production routing remain
intentionally legacy-owned.

I did not run `uv run python scripts/audit_vendored_sources.py` because the
script rewrites `third_party/source_audit.json`; first-layer source coverage was
instead checked through `uv.lock`, `third_party/sources/manifest.json`, and the
vendored textgrid source tree.

## Promotion Note

This `dependency_bootstrap_reviewer` role is ready for coordinator state update
and does not block promotion. The coordinator should not mark the whole unit
`verified` from this report alone; the remaining required review roles still
need to pass.
