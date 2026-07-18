# hfa_export_dispatch_contract - dependency_bootstrap_reviewer

Date: 2026-07-18
Decision: pass

Unit: `hfa_export_dispatch_contract`
Role: `dependency_bootstrap_reviewer`

Manifest unit boundary: confirmed. The unit should stay split.

## Findings

No findings.

## Dependency And Bootstrap Evidence

- Capability coverage is complete for the selected dependency/bootstrap scope.
  The dependency record names the three relevant capabilities: exporter
  membership dispatch, `InferenceBase.export` default/status policy, and
  CLI/API caller context kept as evidence only
  (`rewrite-in-rust/dependencies/hfa_export_dispatch_contract.yaml:3`,
  `rewrite-in-rust/dependencies/hfa_export_dispatch_contract.yaml:8`,
  `rewrite-in-rust/dependencies/hfa_export_dispatch_contract.yaml:12`). These
  map directly to legacy `Exporter.export` checking `"textgrid"` before `"htk"`
  (`inference/HubertFA/tools/export_tool.py:65`) and `InferenceBase.export`
  converting only `None` to `["textgrid"]` before printing the final status
  (`inference/HubertFA/tools/infer_base.py:240`).
- The seam choice is appropriate. The manifest keeps the unit at
  `status: reimplemented`, `inventory_status: confirmed`, and
  `current_owner: legacy`, with rollback to Python export/caller routing
  (`rewrite-in-rust/manifest.yaml:1634`, `rewrite-in-rust/manifest.yaml:1636`,
  `rewrite-in-rust/manifest.yaml:1637`, `rewrite-in-rust/manifest.yaml:1638`,
  `rewrite-in-rust/manifest.yaml:1659`). Record `0074` explicitly split HTK
  planning, TextGrid serialization/path planning, and export dispatch as
  separate HFA export lifecycle units
  (`rewrite-in-rust/records/0074-split-hfa-g2p-config-export-lifecycle.md:18`,
  `rewrite-in-rust/records/0074-split-hfa-g2p-config-export-lifecycle.md:20`,
  `rewrite-in-rust/records/0074-split-hfa-g2p-config-export-lifecycle.md:22`).
- Kept-legacy boundaries are clear. The dependency record keeps HTK/TextGrid
  rendering, filesystem effects, ONNX CLI/API routing, and model execution out
  of this unit (`rewrite-in-rust/dependencies/hfa_export_dispatch_contract.yaml:35`,
  `rewrite-in-rust/dependencies/hfa_export_dispatch_contract.yaml:38`). The
  bootstrap record also excludes Click, TextGrid, filesystem, model, bridge,
  and runtime-router dependencies from the dispatch policy
  (`rewrite-in-rust/bootstrap/hfa_export_dispatch_contract.md:14`,
  `rewrite-in-rust/bootstrap/hfa_export_dispatch_contract.md:16`). Source
  inspection confirms the caller context is only evidence: the CLI lowercases a
  Click choice before calling `InferenceBase.export`
  (`inference/HubertFA/onnx_infer.py:71`, `inference/HubertFA/onnx_infer.py:84`),
  while `hfa_api` requests `["textgrid"]` for artifact copying
  (`inference/API/hfa_api.py:145`, `inference/API/hfa_api.py:158`).
- The crate/dependency decision is justified. The dispatch module uses an
  injected sink and no production bridge (`rewrite-in-rust/rust/crates/v2m-core/src/hfa_export_dispatch.rs:1`,
  `rewrite-in-rust/rust/crates/v2m-core/src/hfa_export_dispatch.rs:55`,
  `rewrite-in-rust/rust/crates/v2m-core/src/hfa_export_dispatch.rs:121`,
  `rewrite-in-rust/rust/crates/v2m-core/src/hfa_export_dispatch.rs:144`).
  Its planning adapter composes the already split HTK/TextGrid planner APIs
  without taking ownership of serializer internals or filesystem effects
  (`rewrite-in-rust/rust/crates/v2m-core/src/hfa_export_dispatch.rs:161`,
  `rewrite-in-rust/rust/crates/v2m-core/src/hfa_export_dispatch.rs:190`). The
  `v2m-core` dependency list contains no dispatch-specific bridge, Click,
  TextGrid, PyO3, or filesystem helper crate
  (`rewrite-in-rust/rust/crates/v2m-core/Cargo.toml:12`).
- First-layer source coverage is adequate for this unit. The direct behavior is
  project-owned Python in `export_tool.py` and `infer_base.py`; no new
  third-party package or targeted transitive expansion is needed for membership
  checks, defaulting, status printing, or injected-sink composition. The broader
  source audit also passed.
- Fixture adequacy matches the accepted boundary. The manifest and bootstrap
  record require list/tuple/string/mapping/empty/`None` formats, duplicates,
  unknown/case variants, fixed call order, downstream short-circuiting, repeated
  calls, default versus explicit empty formats, output-folder forwarding, and
  exact status prints (`rewrite-in-rust/manifest.yaml:1651`,
  `rewrite-in-rust/bootstrap/hfa_export_dispatch_contract.md:20`). The JSONL
  file covers those 18 rows, including string substring membership, mapping-key
  membership, `Exporter.export(None)` TypeError, no-op empty formats,
  TextGrid-before-HTK ordering, and status suppression after downstream errors
  (`rewrite-in-rust/fixtures/hfa_export_dispatch_contract.jsonl:1`,
  `rewrite-in-rust/fixtures/hfa_export_dispatch_contract.jsonl:6`,
  `rewrite-in-rust/fixtures/hfa_export_dispatch_contract.jsonl:8`,
  `rewrite-in-rust/fixtures/hfa_export_dispatch_contract.jsonl:10`,
  `rewrite-in-rust/fixtures/hfa_export_dispatch_contract.jsonl:17`). The Python
  checker injects recording/erroring exporters and restores monkeypatches after
  each run (`rewrite-in-rust/bootstrap/check_hfa_export_dispatch_contract.py:24`,
  `rewrite-in-rust/bootstrap/check_hfa_export_dispatch_contract.py:62`,
  `rewrite-in-rust/bootstrap/check_hfa_export_dispatch_contract.py:86`).
- Rollback is explicit and sufficient: keep `Exporter.export`,
  `InferenceBase.export`, ONNX CLI selection, and API artifact routing as the
  runtime owners (`rewrite-in-rust/manifest.yaml:1659`,
  `rewrite-in-rust/bootstrap/hfa_export_dispatch_contract.md:27`,
  `rewrite-in-rust/records/0096-implement-hfa-export-dispatch-contract.md:12`).

## Checks

- `uv run python -B rewrite-in-rust/bootstrap/check_hfa_export_dispatch_contract.py`:
  passed; validated 18 `hfa_export_dispatch_contract` fixtures against legacy
  Python with injected sinks.
- `CARGO_TARGET_DIR=/tmp/v2m-hfa-export-dispatch-review-target cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_export_dispatch_contract`:
  passed; 1 focused Rust fixture-parity test passed, 117 `v2m-core` tests and 5
  `v2m-quant-bridge` tests filtered out.
- `uv run python -B -m py_compile inference/HubertFA/tools/export_tool.py inference/HubertFA/tools/infer_base.py rewrite-in-rust/bootstrap/check_hfa_export_dispatch_contract.py`:
  passed.
- `uv run python -B scripts/audit_vendored_sources.py`: passed; reported 135
  Python packages, 41 native-extension packages, 269 foreign runtime native
  binaries, and 0 `third_party` binary artifacts.

## Residual Risk

This review covers dependency/bootstrap scope only. It does not re-review
behavior parity, error/tracing, Rust style, architecture, or product ergonomics.
HTK/TextGrid serializer behavior, filesystem IO, status presentation in the
production Python runtime, CLI/API routing, model execution, and bridge design
remain intentionally outside this unit.

## Promotion Note

This `dependency_bootstrap_reviewer` report does not block promotion for this
role. The coordinator should not mark the unit `verified` from this report
alone; the manifest still requires separate `stage_behavior_reviewer` and
`error_tracing_reviewer` gates.
