# hfa_textgrid_export_core - dependency_bootstrap_reviewer rerun

## Findings

No findings.

## Decision

Date: 2026-07-18
Decision: pass

Unit: `hfa_textgrid_export_core`
Role: `dependency_bootstrap_reviewer`
Rerun context: record `0094` expanded the fixture table from 15 to 18 cases for
POSIX exactly-two-leading-slash path behavior.

Manifest unit boundary: confirmed. The unit should stay split.

## Bootstrap Evidence

- The boundary remains the narrow in-memory TextGrid export planner, not a
  production route. The manifest keeps `hfa_textgrid_export_core` at
  `status: reimplemented`, `inventory_status: confirmed`, and
  `current_owner: legacy` (`rewrite-in-rust/manifest.yaml:1601`,
  `rewrite-in-rust/manifest.yaml:1603`,
  `rewrite-in-rust/manifest.yaml:1605`). Record `0074` split HTK label
  planning, the TextGrid long-format/path subset, and export dispatch into
  separate confirmed units (`rewrite-in-rust/records/0074-split-hfa-g2p-config-export-lifecycle.md:25`,
  `rewrite-in-rust/records/0074-split-hfa-g2p-config-export-lifecycle.md:27`,
  `rewrite-in-rust/records/0074-split-hfa-g2p-config-export-lifecycle.md:29`).
  Records `0093` and `0094` both state that no production export route,
  filesystem write, API artifact-copying path, GUI, Web, or model-inference code
  changed (`rewrite-in-rust/records/0093-implement-hfa-textgrid-export-core.md:12`,
  `rewrite-in-rust/records/0094-fix-hfa-textgrid-double-slash-paths.md:23`).
- Capability coverage still matches the selected seam. The dependency record
  names tier construction, `textgrid` 1.6.1 serialization, and path planning
  (`rewrite-in-rust/dependencies/hfa_textgrid_export_core.yaml:3`). These map to
  `Exporter.save_textgrids` constructing `words` and `phones` tiers, applying
  `max(0, phoneme.start)`, selecting output-folder versus wav-parent paths,
  creating the parent directory, and calling `tg.write`
  (`inference/HubertFA/tools/export_tool.py:11`,
  `inference/HubertFA/tools/export_tool.py:15`,
  `inference/HubertFA/tools/export_tool.py:22`,
  `inference/HubertFA/tools/export_tool.py:27`,
  `inference/HubertFA/tools/export_tool.py:32`).
- The no-crate/no-bridge decision remains justified. The seam record has
  `bridge_dependencies: []`
  (`rewrite-in-rust/dependencies/hfa_textgrid_export_core.yaml:16`), the
  bootstrap record says to hand-write only the selected
  `TextGrid`/`IntervalTier` writer subset and not add a general TextGrid crate
  (`rewrite-in-rust/bootstrap/hfa_textgrid_export_core.md:13`), and `v2m-core`
  has no TextGrid or bridge dependency
  (`rewrite-in-rust/rust/crates/v2m-core/Cargo.toml:12`). The Rust module
  documents that Python still owns directory creation, file writes, status
  printing, artifact copying, export dispatch, and production routing
  (`rewrite-in-rust/rust/crates/v2m-core/src/hfa_textgrid_export.rs:1`).
- First-layer `textgrid` 1.6.1 source coverage is adequate. `pyproject.toml`
  declares `textgrid` (`pyproject.toml:33`), `uv.lock` pins `textgrid==1.6.1`
  with an sdist (`uv.lock:4074`), and the vendored source manifest points to
  `third_party/sources/textgrid-1.6.1`
  (`third_party/sources/manifest.json:870`). The imported package layer consists
  of `textgrid/__init__.py`, `textgrid/exceptions.py`, and
  `textgrid/textgrid.py`; `__init__.py` exports the package classes used by the
  legacy import (`third_party/sources/textgrid-1.6.1/textgrid/__init__.py:1`).
  `setup.py` declares only the `textgrid` package
  (`third_party/sources/textgrid-1.6.1/setup.py:3`), and the selected source
  imports standard-library modules plus local `TextGridError`
  (`third_party/sources/textgrid-1.6.1/textgrid/textgrid.py:32`,
  `third_party/sources/textgrid-1.6.1/textgrid/textgrid.py:40`), so no targeted
  transitive expansion is needed for this unit.
- The hand-written subset remains narrow and source-backed. The vendored
  `textgrid` source covers interval rejection for zero/invalid durations,
  out-of-range checks, sorted strict insertion, blank-gap filling, quote
  doubling, and long TextGrid UTF-8 writing
  (`third_party/sources/textgrid-1.6.1/textgrid/textgrid.py:186`,
  `third_party/sources/textgrid-1.6.1/textgrid/textgrid.py:453`,
  `third_party/sources/textgrid-1.6.1/textgrid/textgrid.py:459`,
  `third_party/sources/textgrid-1.6.1/textgrid/textgrid.py:519`,
  `third_party/sources/textgrid-1.6.1/textgrid/textgrid.py:76`,
  `third_party/sources/textgrid-1.6.1/textgrid/textgrid.py:751`). The Rust
  module mirrors only planned tiers, paths, UTF-8 content, and partial failure
  plans (`rewrite-in-rust/rust/crates/v2m-core/src/hfa_textgrid_export.rs:99`,
  `rewrite-in-rust/rust/crates/v2m-core/src/hfa_textgrid_export.rs:121`,
  `rewrite-in-rust/rust/crates/v2m-core/src/hfa_textgrid_export.rs:136`).
- Fixture adequacy is now aligned with the 18-case state. The manifest,
  dependency record, bootstrap record, and `0094` all describe 18 cases and the
  double-slash addition (`rewrite-in-rust/manifest.yaml:1618`,
  `rewrite-in-rust/dependencies/hfa_textgrid_export_core.yaml:20`,
  `rewrite-in-rust/bootstrap/hfa_textgrid_export_core.md:20`,
  `rewrite-in-rust/records/0094-fix-hfa-textgrid-double-slash-paths.md:28`).
  Fixture rows 16-18 lock `output_folder="//out"`, wav-parent
  `wav_path="//a/b/song.wav"`, and root wav-parent `wav_path="//song.wav"` to
  Python `pathlib`'s exactly-two-slash projection
  (`rewrite-in-rust/fixtures/hfa_textgrid_export_core.jsonl:16`,
  `rewrite-in-rust/fixtures/hfa_textgrid_export_core.jsonl:17`,
  `rewrite-in-rust/fixtures/hfa_textgrid_export_core.jsonl:18`). The checker
  exercises the real legacy `Exporter.save_textgrids` with monkeypatched
  `Path.mkdir` and `codecs.open` sinks, then asserts every JSONL row
  (`rewrite-in-rust/bootstrap/check_hfa_textgrid_export_core.py:100`,
  `rewrite-in-rust/bootstrap/check_hfa_textgrid_export_core.py:127`,
  `rewrite-in-rust/bootstrap/check_hfa_textgrid_export_core.py:164`).
  The Rust fixture test includes the same file and checks each row
  (`rewrite-in-rust/rust/crates/v2m-core/src/hfa_textgrid_export.rs:522`,
  `rewrite-in-rust/rust/crates/v2m-core/src/hfa_textgrid_export.rs:624`).
- Kept-legacy capabilities are still correctly excluded. General TextGrid
  read/PointTier/MLF APIs and directory creation, writes, IO errors, status
  printing, artifact copying, and production routing remain legacy-owned
  (`rewrite-in-rust/dependencies/hfa_textgrid_export_core.yaml:35`).
  Dispatch/default behavior remains in `InferenceBase.export`
  (`inference/HubertFA/tools/infer_base.py:240`), and API artifact copying is
  separate caller behavior (`inference/API/hfa_api.py:145`).
- Rollback remains explicit and sufficient. The manifest says to keep
  `Exporter.save_textgrids` and textgrid 1.6.1 as runtime owners
  (`rewrite-in-rust/manifest.yaml:1627`), record `0093` says reversal is removing
  the independent Rust module, fixture, checker, and manifest entries if the
  boundary is later re-cut
  (`rewrite-in-rust/records/0093-implement-hfa-textgrid-export-core.md:70`),
  and record `0094` confirms no production caller route changed
  (`rewrite-in-rust/records/0094-fix-hfa-textgrid-double-slash-paths.md:59`).

## Checks

- `PYTHONDONTWRITEBYTECODE=1 uv run python rewrite-in-rust/bootstrap/check_hfa_textgrid_export_core.py`:
  passed; validated 18 Python 3.12 fixtures against the real monkeypatched
  legacy exporter.
- `CARGO_TARGET_DIR=/tmp/v2m-hfa-textgrid-dep-rerun-target cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml --locked hfa_textgrid_export_core -- --nocapture`:
  passed; 1 focused `v2m-core` fixture-parity test passed, 116 `v2m-core` tests
  and 5 `v2m-quant-bridge` tests filtered out.
- `PYTHONDONTWRITEBYTECODE=1 uv run python - <<'PY' ...`: parsed
  `rewrite-in-rust/fixtures/hfa_textgrid_export_core.jsonl`; confirmed 18 case
  ids and rows 16-18 covering `//out/TextGrid`,
  `//a/b/TextGrid`, and `//TextGrid` planned paths.
- `rg --files third_party/sources/textgrid-1.6.1 | sort`: listed the vendored
  `textgrid` 1.6.1 source tree, including `textgrid/__init__.py`,
  `textgrid/exceptions.py`, and `textgrid/textgrid.py`.
- `rg -n "textgrid|TextGrid|pyo3|PyO3|subprocess|bridge_dependencies|bridge" ...`:
  inspected the TextGrid dependency/bootstrap docs, manifest entry, Cargo files,
  and Rust module. No TextGrid crate, PyO3, subprocess, or new bridge dependency
  is present for this unit; the only bridge hit in Cargo workspace metadata is
  the unrelated existing `v2m-quant-bridge`.
- `rg -n "std::fs|File::|create_dir|write_all|OpenOptions|println!|eprintln!|dbg!|log::|tracing::" rewrite-in-rust/rust/crates/v2m-core/src/hfa_textgrid_export.rs`:
  no matches; the Rust planner has no filesystem, logging, or stdout/stderr
  side effects.
- `git diff --check -- rewrite-in-rust/manifest.yaml rewrite-in-rust/dependencies/hfa_textgrid_export_core.yaml rewrite-in-rust/bootstrap/hfa_textgrid_export_core.md rewrite-in-rust/bootstrap/check_hfa_textgrid_export_core.py rewrite-in-rust/fixtures/hfa_textgrid_export_core.jsonl rewrite-in-rust/rust/crates/v2m-core/src/hfa_textgrid_export.rs rewrite-in-rust/records/0093-implement-hfa-textgrid-export-core.md rewrite-in-rust/records/0094-fix-hfa-textgrid-double-slash-paths.md`:
  passed.
- Source inspection covered the required rewrite docs, records `0074`/`0093`/`0094`,
  dependency/bootstrap records, manifest entry, source refs, vendored `textgrid`
  source, checker, 18-row fixture file, and Rust module.

## Residual Risk

This review covers dependency/bootstrap scope only. It does not re-approve
behavior parity, error/tracing, Rust style, architecture, or product ergonomics.
General TextGrid read/PointTier/MLF behavior, real filesystem IO failures,
status printing, artifact copying, export dispatch, and production routing
remain intentionally legacy-owned.

I did not run `uv run python scripts/audit_vendored_sources.py` because prior
review evidence notes that it rewrites `third_party/source_audit.json`; first
layer source coverage was instead checked through `uv.lock`,
`third_party/sources/manifest.json`, and the vendored `textgrid` source tree.
I also did not run `py_compile` because it can create `__pycache__`; the legacy
checker imported and exercised the relevant source under `uv`.

## Coordinator Readiness

This `dependency_bootstrap_reviewer` rerun is ready for coordinator state update
as passing review evidence for this role. It does not by itself justify marking
the unit `verified`; the coordinator still needs the required post-`0094`
behavior evidence and any other required role evidence before updating the
manifest.
