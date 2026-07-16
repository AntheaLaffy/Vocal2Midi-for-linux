# ustx_project_export_core - dependency_bootstrap_reviewer rerun

Date: 2026-07-16
Decision: pass

## Boundary Decision

Confirmed. `ustx_project_export_core` remains correctly cut as the deterministic
`save_ustx(..., rmvpe_result=None)` project YAML renderer. The boundary keeps
RMVPE-derived `pitd` curve generation, model/runtime execution, production
filesystem writes, warning/status output, runtime routing, and broad
PyYAML/NumPy package parity legacy-owned or assigned to later units.

The previous dependency-bootstrap scalar follow-up is sufficiently
fixture-bound for this unit. The fixture table now drives a punctuation-sensitive
output stem and YAML-sensitive lyrics through the legacy checker and Rust tests,
and the golden YAML pins the PyYAML quoting that this narrowed renderer must
match.

## Findings

No findings.

## Evidence Reviewed

- Manifest entry: `rewrite-in-rust/manifest.yaml:894` through
  `rewrite-in-rust/manifest.yaml:917` records `status: reimplemented`,
  `inventory_status: confirmed`, the `rmvpe_result=None` verification scope,
  dependency/bootstrap/record evidence, and the Python runtime rollback route.
- Dependency record:
  `rewrite-in-rust/dependencies/ustx_project_export_core.yaml:1` through
  `rewrite-in-rust/dependencies/ustx_project_export_core.yaml:53` keeps the seam
  as an independent Rust library, justifies a narrow hand-written YAML renderer,
  keeps general PyYAML serialization legacy-owned, and names the checker,
  Rust test, and source audit commands.
- Bootstrap record:
  `rewrite-in-rust/bootstrap/ustx_project_export_core.md:5` through
  `rewrite-in-rust/bootstrap/ustx_project_export_core.md:26` defines the
  `save_ustx(..., rmvpe_result=None)` compatibility surface and exclusions, and
  `rewrite-in-rust/bootstrap/ustx_project_export_core.md:67` through
  `rewrite-in-rust/bootstrap/ustx_project_export_core.md:79` requires an
  in-memory Rust YAML result with no filesystem writes, Python calls, or bridge.
- Decision record:
  `rewrite-in-rust/records/0042-confirm-ustx-project-export-boundary.md:18`
  through `rewrite-in-rust/records/0042-confirm-ustx-project-export-boundary.md:39`
  confirms project/note YAML assembly while excluding pitch curves, RMVPE model
  execution, production writes, warning/status printing, runtime routing, and
  broad PyYAML/NumPy parity.
- Legacy source: `inference/API/ustx_api.py:369` through
  `inference/API/ustx_api.py:460` sorts finite notes, converts ticks and tones,
  builds the fixed USTX project tree, uses `filepath.stem` and note lyrics as
  dynamic strings, keeps curves empty when `rmvpe_result is None`, and writes
  with `yaml.safe_dump(..., allow_unicode=True, sort_keys=False)`.
- PyYAML evidence:
  `third_party/sources/pyyaml-6.0.3/lib/yaml/__init__.py:263`,
  `third_party/sources/pyyaml-6.0.3/lib/yaml/representer.py:103`,
  `third_party/sources/pyyaml-6.0.3/lib/yaml/representer.py:147`, and
  `third_party/sources/pyyaml-6.0.3/lib/yaml/emitter.py:494` show the SafeDumper,
  mapping-order, string representation, and scalar-style paths used by the
  legacy export.
- NumPy evidence:
  `third_party/sources/numpy-1.26.4/numpy/core/_methods.py:90` through
  `third_party/sources/numpy-1.26.4/numpy/core/_methods.py:99` supports the
  scalar clip capability used by tone clamping; the Rust unit does not need
  NumPy array kernels or BLAS.
- Scalar fixture:
  `rewrite-in-rust/fixtures/ustx_project_export_core.jsonl:2` now includes stem
  `a: b` plus lyrics `yes`, `#tag`, and `{x: y}`. The golden YAML pins quoted
  project and voice-part names at
  `rewrite-in-rust/fixtures/ustx_project_export_core/edge_project.ustx:1` and
  `rewrite-in-rust/fixtures/ustx_project_export_core/edge_project.ustx:242`,
  and quoted lyrics at
  `rewrite-in-rust/fixtures/ustx_project_export_core/edge_project.ustx:274`,
  `rewrite-in-rust/fixtures/ustx_project_export_core/edge_project.ustx:298`,
  and `rewrite-in-rust/fixtures/ustx_project_export_core/edge_project.ustx:322`.
- Checker:
  `rewrite-in-rust/bootstrap/check_ustx_project_export_core.py:77` through
  `rewrite-in-rust/bootstrap/check_ustx_project_export_core.py:88` regenerates
  legacy YAML with `save_ustx(..., rmvpe_result=None)` and compares exact text
  against the golden files; lines 102 through 107 validate the selected
  project-core shape keeps one voice part with empty curves.
- Rust implementation:
  `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:36` through
  `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:112` renders
  in-memory YAML and skipped counts only, lines 197 through 270 implement the
  fixture-bound scalar renderer, lines 629 through 665 compare Rust output to
  the same golden YAML fixture table, and lines 677 through 690 pin
  YAML-sensitive scalar quoting cases directly.
- Previous reports reviewed:
  `rewrite-in-rust/reviews/2026-07-16-ustx_project_export_core-dependency_bootstrap_reviewer.md`,
  `rewrite-in-rust/reviews/2026-07-16-ustx_project_export_core-behavior_reviewer.md`,
  `rewrite-in-rust/reviews/2026-07-16-ustx_project_export_core-data_algorithm_reviewer.md`,
  and
  `rewrite-in-rust/reviews/2026-07-16-ustx_project_export_core-error_tracing_reviewer.md`.
  Their shared YAML scalar concern is now represented in the fixture and Rust
  unit-test evidence above for dependency/bootstrap purposes.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_ustx_project_export_core.py`:
  pass
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml ustx_project`:
  pass, 3 tests
- `uv run python scripts/audit_vendored_sources.py`: pass
- `uv run python -c "import yaml; vals=['a: b','#tag','yes','1','[x]','{x: y}','a #b','a#b',':start','你']; print(yaml.safe_dump({'name':'a: b','voice_parts':[{'name':'a: b','notes':[{'lyric':v} for v in vals]}]}, allow_unicode=True, sort_keys=False), end='')"`:
  pass; PyYAML quoted the scalar hazards now represented in the fixture while
  leaving `a#b`, `:start`, and `你` plain.
- `git diff --check -- rewrite-in-rust/reviews/2026-07-16-ustx_project_export_core-dependency_bootstrap_reviewer-rerun.md`:
  pass

## Residual Risk

This review approves the dependency/bootstrap boundary only. It does not approve
general PyYAML serialization, optional pitch-curve rendering, production
filesystem/write behavior, warning output mapping, runtime bridge design, or
promotion-time error mapping. Those remain explicitly outside this unit's
dependency/bootstrap scope.

## Promotion Note

This dependency-bootstrap rerun does not block coordinator state update for this
role. The manifest unit boundary is confirmed, should not be split or merged for
the reviewed scope, and the previous scalar follow-up is closed for the
fixture-bound `ustx_project_export_core` boundary.
