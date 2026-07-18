# hfa_htk_label_export_core - dependency_bootstrap_reviewer

Date: 2026-07-18
Decision: pass

## Findings

No findings.

## Boundary Judgment

Manifest unit boundary: confirmed. The unit should stay split.

`hfa_htk_label_export_core` is an appropriate minimum library unit for
`Exporter.save_htk` planned side effects. The manifest marks the unit
`reimplemented` and `confirmed`, with source refs limited to
`export_tool.py`/`infer_base.py` and rollback to Python `Exporter.save_htk`
(`rewrite-in-rust/manifest.yaml:1566`). The dependency record covers the three
capabilities needed for this seam: HTK content rendering, the cross-prediction
cumulative buffers, and path planning (`rewrite-in-rust/dependencies/hfa_htk_label_export_core.yaml:3`).

The split from TextGrid export and format dispatch is supported. Record 0074
identifies HTK planned files, TextGrid serialization, and export dispatch as
separate units (`rewrite-in-rust/records/0074-split-hfa-g2p-config-export-lifecycle.md:25`),
and the dependency record keeps TextGrid serialization and dispatch legacy-owned
(`rewrite-in-rust/dependencies/hfa_htk_label_export_core.yaml:34`). No merge,
replacement, or deferral is needed for this dependency/bootstrap gate.

## Bootstrap Evidence

- Capability boundary matches the Python seam. `Exporter.save_htk` initializes
  `w_out`/`ph_out` before the prediction loop, scales times with
  `int(float(value) * HTK_PAD_VAL)`, plans `HTK/Phones` and `HTK/Words` paths,
  creates two directories, and writes two UTF-8 files
  (`inference/HubertFA/tools/export_tool.py:35`).
- Caller context is covered without broadening the unit. `InferenceBase.export`
  only passes predictions and output folder into `Exporter(...).export(...)`,
  while default format and dispatch behavior are left to separate units
  (`inference/HubertFA/tools/infer_base.py:240`).
- Fixture strategy is adequate for dependency/bootstrap scope. The bootstrap
  checker monkeypatches `pathlib.Path.mkdir` and `open` around the real
  `Exporter.save_htk`, preserving ordered planned side effects without touching
  production paths (`rewrite-in-rust/bootstrap/check_hfa_htk_label_export_core.py:100`).
  The 12 fixture rows cover empty/single/multiple predictions, cumulative later
  files, duplicate basenames, nested paths, output-folder and wav-parent modes,
  special times, Unicode/quote/newline text, empty phones/words, conversion
  errors with partial side effects, and repeated exporter calls.
- Kept-legacy choices are appropriate. Directory creation, file writes, IO
  errors, status printing, production routing, TextGrid serialization, and
  dispatch remain legacy-owned (`rewrite-in-rust/dependencies/hfa_htk_label_export_core.yaml:34`).
- No unnecessary dependency or bridge is introduced for this unit. The bootstrap
  doc states "no bridge or new crate" (`rewrite-in-rust/bootstrap/hfa_htk_label_export_core.md:18`);
  the Rust implementation is exported as `v2m-core::hfa_htk_export`
  (`rewrite-in-rust/rust/crates/v2m-core/src/lib.rs:19`) and uses the existing
  Rust workspace dependency set without adding an HTK/TextGrid crate
  (`rewrite-in-rust/rust/crates/v2m-core/Cargo.toml:12`).

## Checks

- `uv run python --version`: passed, Python 3.12.13.
- `uv run python -m py_compile inference/HubertFA/tools/export_tool.py rewrite-in-rust/bootstrap/check_hfa_htk_label_export_core.py`: passed.
- `uv run python rewrite-in-rust/bootstrap/check_hfa_htk_label_export_core.py`: passed, validated 12 fixtures.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_htk_label_export_core`: passed, 1 test.
- `uv run python scripts/audit_vendored_sources.py`: passed, source audit reported 135 Python packages, 41 native-extension packages, 269 foreign runtime native binaries, and 0 third_party binary artifacts.

## Residual Risk

This review did not perform behavior, data/algorithm, error/tracing, Rust style,
or architecture review. Filesystem error behavior and actual production routing
remain intentionally legacy-owned and are not proven by this dependency gate.

## Promotion Note

This `dependency_bootstrap_reviewer` role does not block promotion. The unit is
not ready for coordinator `verified` state until the remaining required reviews
for `hfa_htk_label_export_core` pass.
