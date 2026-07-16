# ustx_project_export_core - dependency_bootstrap_reviewer

Date: 2026-07-16
Decision: pass-with-followups

## Boundary Decision

Confirmed. The unit is correctly cut as `save_ustx(..., rmvpe_result=None)`
project YAML rendering, with `ustx_pitch_curve_core` retaining RMVPE-derived
`pitd` curve generation.

## Findings

- Severity: low
- Location: rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:197
- Issue: The hand-written YAML scalar renderer is intentionally narrow and the
  current bootstrap fixtures cover empty fallback lyrics, simple ASCII lyrics,
  boolean-like scalar quoting, and UTF-8 lyrics, but not punctuation-heavy
  project names or lyrics that PyYAML may quote or escape differently.
- Evidence: `rewrite-in-rust/dependencies/ustx_project_export_core.yaml:16`
  scopes YAML rendering to a fixture-bound renderer, and
  `rewrite-in-rust/bootstrap/ustx_project_export_core.md:128` keeps YAML
  compatibility under later promotion. The Rust helper at
  `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:197` implements a
  small scalar subset rather than broad PyYAML parity.
- Required fix: Before runtime promotion, either add scalar-edge fixtures for
  project names and lyrics or document the promoted Rust boundary as accepting
  only the already-fixtured scalar subset.

## Evidence Reviewed

- Manifest entry: `rewrite-in-rust/manifest.yaml:894`
- Dependency record: `rewrite-in-rust/dependencies/ustx_project_export_core.yaml:1`
- Bootstrap record: `rewrite-in-rust/bootstrap/ustx_project_export_core.md:1`
- Decision record: `rewrite-in-rust/records/0042-confirm-ustx-project-export-boundary.md:1`
- Legacy source: `inference/API/ustx_api.py:369`
- Fixture checker: `rewrite-in-rust/bootstrap/check_ustx_project_export_core.py:1`
- Fixtures: `rewrite-in-rust/fixtures/ustx_project_export_core.jsonl:1`
- Rust renderer: `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:1`
- Vendored source manifest: `third_party/sources/manifest.json:473` and
  `third_party/sources/manifest.json:702`

The `rmvpe_result=None` boundary is valid because `save_ustx` only calls
`_build_pitd_curve` when `rmvpe_result is not None`
(`inference/API/ustx_api.py:407`), while the project and note YAML path is
deterministic from notes, file stem, and tempo. Record 0042 explicitly keeps
pitch curves, RMVPE model execution, filesystem writes, warning/status output,
runtime routing, and broad PyYAML/NumPy parity out of this unit
(`rewrite-in-rust/records/0042-confirm-ustx-project-export-boundary.md:33`).

The kept-legacy decisions are justified. Runtime filesystem behavior remains
owned by Python at `inference/API/ustx_api.py:458`, and the bootstrap seam
requires Rust to return in-memory YAML text plus skipped-note count without
creating directories, writing files, calling Python, or adding a bridge
(`rewrite-in-rust/bootstrap/ustx_project_export_core.md:67`).

The hand-written replacement choice is sufficient for dependency/bootstrap:
PyYAML and NumPy are declared dependencies, `uv.lock` pins `numpy==1.26.4` and
`pyyaml==6.0.3`, the vendored source manifest records both source directories,
and `audit_vendored_sources.py` passed. The selected Rust unit does not require
NumPy array kernels, OpenBLAS, ONNX Runtime, RMVPE inference, PyO3, subprocess,
or HTTP bridge dependencies.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_ustx_project_export_core.py`:
  pass
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml ustx_project`:
  pass, 3 tests
- `uv run python scripts/audit_vendored_sources.py`: pass
- `git diff --check -- rewrite-in-rust/manifest.yaml rewrite-in-rust/dependencies/ustx_project_export_core.yaml rewrite-in-rust/bootstrap/ustx_project_export_core.md rewrite-in-rust/records/0042-confirm-ustx-project-export-boundary.md rewrite-in-rust/bootstrap/check_ustx_project_export_core.py rewrite-in-rust/fixtures/ustx_project_export_core.jsonl rewrite-in-rust/fixtures/ustx_project_export_core/empty_project.ustx rewrite-in-rust/fixtures/ustx_project_export_core/edge_project.ustx rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs rewrite-in-rust/rust/crates/v2m-core/src/lib.rs`:
  pass

## Residual Risk

This review approves dependency/bootstrap only. It does not approve general
PyYAML serialization, pitch curve rendering, production filesystem writes,
warning output mapping, runtime bridge design, or broad scalar escaping beyond
the current fixture set.

## Promotion Note

This role does not block coordinator state update from dependency/bootstrap.
The low scalar-coverage follow-up should be resolved before runtime promotion
or any claim of broad PyYAML-compatible string rendering.
