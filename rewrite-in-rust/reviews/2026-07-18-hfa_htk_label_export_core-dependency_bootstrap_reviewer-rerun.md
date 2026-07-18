# hfa_htk_label_export_core - dependency_bootstrap_reviewer rerun

Date: 2026-07-18
Decision: pass

## Findings

No findings.

## Boundary Judgment

Manifest unit boundary: confirmed. The unit should stay split.

Record 0089 fixes the two previous HTK review gaps by adding Python big-int
finite time rendering and empty-name `wav_path` failure handling without adding
a crate, bridge, production route, or broader export responsibility
(`rewrite-in-rust/records/0089-fix-hfa-htk-review-findings.md:19`,
`rewrite-in-rust/records/0089-fix-hfa-htk-review-findings.md:25`,
`rewrite-in-rust/records/0089-fix-hfa-htk-review-findings.md:62`). The manifest
still marks `hfa_htk_label_export_core` as `reimplemented` and `confirmed`, with
the public policy limited to HTK prediction order, time rendering, raw text,
path planning, and cumulative buffers
(`rewrite-in-rust/manifest.yaml:1566`, `rewrite-in-rust/manifest.yaml:1575`).

The record-0074 split remains appropriate after the 14-case fixture update. HTK
planned files, TextGrid serialization, and export dispatch are separate
confirmed units (`rewrite-in-rust/records/0074-split-hfa-g2p-config-export-lifecycle.md:25`).
`save_htk` has its own observable cumulative-buffer contract
(`rewrite-in-rust/records/0074-split-hfa-g2p-config-export-lifecycle.md:46`),
while TextGrid's third-party serialization contract is explicitly separate
(`rewrite-in-rust/records/0074-split-hfa-g2p-config-export-lifecycle.md:49`).
No merge, replacement, or deferral is needed for this role.

## Bootstrap Evidence

- Capability coverage is still complete for the dependency/bootstrap scope. The
  dependency record names HTK rendering, cumulative prediction state, and path
  planning as the selected capabilities
  (`rewrite-in-rust/dependencies/hfa_htk_label_export_core.yaml:3`), and now
  points to 14 Python-generated fixture cases covering the prior huge-finite and
  empty-path gaps (`rewrite-in-rust/dependencies/hfa_htk_label_export_core.yaml:20`).
- The seam choice matches the Python source. `Exporter.save_htk` converts times,
  appends unescaped word/phoneme text, computes `HTK/Phones` and `HTK/Words`
  `.lab` paths, creates directories, and writes UTF-8 files
  (`inference/HubertFA/tools/export_tool.py:35`). `InferenceBase.export` remains
  caller context only and delegates to `Exporter(...).export(...)`
  (`inference/HubertFA/tools/infer_base.py:240`).
- Fixture adequacy improved from the first dependency review. The bootstrap doc
  now describes 14 Python-generated cases including huge finite times and empty
  wav-path name errors
  (`rewrite-in-rust/bootstrap/hfa_htk_label_export_core.md:26`), and the checker
  runs the real `Exporter.save_htk` with monkeypatched `mkdir`/`open` sinks so
  ordered side effects are captured without writing production files
  (`rewrite-in-rust/bootstrap/check_hfa_htk_label_export_core.py:100`).
- The no-HTK-crate and no-bridge choice remains justified. This unit needs
  standard float conversion, string concatenation, and path planning, not a
  third-party HTK parser/writer. The dependency record has no bridge
  dependencies (`rewrite-in-rust/dependencies/hfa_htk_label_export_core.yaml:16`)
  and explicitly justifies the hand-written replacement
  (`rewrite-in-rust/dependencies/hfa_htk_label_export_core.yaml:27`). The Rust
  workspace dependency list does not add an HTK/TextGrid crate
  (`rewrite-in-rust/rust/crates/v2m-core/Cargo.toml:12`).
- First-layer source coverage is adequate for this seam. The direct behavior
  source is project-owned `export_tool.py::Exporter.save_htk`; `textgrid` is only
  needed by `save_textgrids` and stays in the separate TextGrid unit
  (`inference/HubertFA/tools/export_tool.py:11`,
  `rewrite-in-rust/dependencies/hfa_htk_label_export_core.yaml:37`). The
  vendored-source audit also passed.
- Rollback remains clear: keep Python `Exporter.save_htk` as runtime owner,
  including cumulative buffers (`rewrite-in-rust/manifest.yaml:1591`,
  `rewrite-in-rust/bootstrap/hfa_htk_label_export_core.md:33`). No production
  import or runtime route points at `v2m-core::hfa_htk_export`.

## Checks

- `uv run python --version`: passed, Python 3.12.13.
- `uv run python -m py_compile inference/HubertFA/tools/export_tool.py rewrite-in-rust/bootstrap/check_hfa_htk_label_export_core.py`: passed.
- `uv run python rewrite-in-rust/bootstrap/check_hfa_htk_label_export_core.py`: passed, validated 14 fixtures.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_htk_label_export_core`: passed, 1 focused Rust fixture-parity test.
- `uv run python scripts/audit_vendored_sources.py`: passed, source audit reported 135 Python packages, 41 native-extension packages, 269 foreign runtime native binaries, and 0 third_party binary artifacts.
- `rg -n "textgrid|HTK|hfa_htk|hfa_htk_export|plan_htk_label_export|save_htk|Exporter\\(" ...`: inspected HTK/TextGrid routing and Rust exposure; Rust exposes only `v2m-core::hfa_htk_export`, while Python `Exporter.save_htk` remains the runtime owner.
- `wc -l rewrite-in-rust/fixtures/hfa_htk_label_export_core.jsonl ...`: confirmed the HTK fixture file has 14 JSONL rows.

## Residual Risk

This review covered dependency/bootstrap scope only. Behavior, data/algorithm,
error/tracing, Rust style, architecture, and product ergonomics are not
re-approved by this report. Filesystem IO failures, status printing, format
dispatch, TextGrid serialization, and production routing remain intentionally
legacy-owned.

## Promotion Note

This `dependency_bootstrap_reviewer` rerun is ready for coordinator state update
for this role and does not block promotion. The unit as a whole is not ready for
`verified` state until the other required `hfa_htk_label_export_core` reviews
pass after the 14-case fixture/dependency update.
