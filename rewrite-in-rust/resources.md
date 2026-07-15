# Rust Rewrite Resources

This file indexes the sources of truth for the rewrite. Prefer these resources
over memory when choosing migration units or dependencies.

## Project Truth

- `README.md`: current user workflows, model paths, CLI commands, and supported
  outputs.
- `docs/architecture.md`: dependency direction and major runtime boundaries.
- `docs/contributing.md`: documentation style, environment assumptions, and
  verification commands.
- `docs/web-api.md`: Web API behavior boundary when Web workflows are affected.
- `rewrite-in-rust/rust/README.md`: Rust workspace toolchain, crate contracts,
  bridge JSON contract, and Rust-specific verification commands.

## Dependency Evidence

- `pyproject.toml`: primary Python dependency declaration.
- `requirements.txt`, `requirements-linux.txt`, `requirements-web.txt`: platform
  and Web dependency views.
- `uv.lock`: locked Python dependency graph.
- `third_party/README.md`: source vendoring policy.
- `third_party/sources/manifest.json`: Python source package coverage.
- `third_party/native_sources/manifest.json`: native/FFI source coverage.
- `third_party/source_audit.json`: repeatable audit result for vendored sources.

## Hand-written Replacement Reference Sources

When Rust dependency parity is poor and a narrow implementation must be written
by hand, inspect these source directories directly:

- `third_party/sources/<package-version>/`: Python source distributions from
  `uv.lock`.
- `third_party/upstream_sources/<package-version>/`: upstream source fallbacks
  for packages without sdists, including `onnxruntime`, `torch`, `flatbuffers`,
  `pyqt5-qt5`, and `dynet38`.
- `third_party/native_sources/<library-version>/`: C, C++, Fortran, and other
  native/FFI source trees used by Python wheels or bundled shared libraries.
- `third_party/cargo_vendor/<source-path>/`: vendored Rust crates for
  Rust-backed Python packages such as tokenizers, safetensors, tiktoken,
  pydantic-core, orjson, rpds-py, and hf-xet.

Use the manifests to locate and justify the source path:

- `third_party/sources/manifest.json`
- `third_party/sources/MISSING_SOURCES.md`
- `third_party/native_sources/manifest.json`
- `third_party/source_audit.json`

Do not use compiled files under `.venv` as implementation references when a
source directory exists. The vendored source tree is the reference surface for
manual Rust replacements.

## Current Public Boundaries

- `application/config.py`: `PipelineConfig` and user-facing validation.
- `application/pipeline.py`: application-layer job entrypoint.
- `inference/device_utils.py`: runtime device normalization and ONNX provider
  selection.
- `inference/pipeline/auto_lyric_hybrid.py`: end-to-end pipeline workflow.
- `scripts/slice_asr_cli.py`: batch CLI behavior.
- `web_server.py` and `web_task_manager.py`: Web caller boundary and task
  lifecycle.

## Project Rewrite Skills

- `rewrite-in-rust/skills/vocal2midi-rs-rewrite/`: coordinator for selecting,
  routing, and updating migration units.
- `rewrite-in-rust/skills/vocal2midi-rs-dep-bootstrap/`: dependency,
  capability, fixture, and seam preparation.
- `rewrite-in-rust/skills/vocal2midi-rs-unit-writer/`: writer role for exactly
  one migration unit.
- `rewrite-in-rust/skills/vocal2midi-rs-review-gate/`: independent review role
  for exactly one unit and one review theme.

These repository skills are the source of truth. The matching
`/home/fuurin/.claude/skills/` directories are installation mirrors for future
sessions.

## Durable Rewrite Artifacts

- `rewrite-in-rust/dependencies/`: per-unit capability and dependency records.
- `rewrite-in-rust/bootstrap/`: per-unit seam or fixture-harness proof records.
- `rewrite-in-rust/reviews/`: independent review reports used as promotion
  evidence.

## Provisional Migration Candidates

These candidates are intentionally provisional. They are starting points for
dependency and capability discovery, not a fixed backlog. Re-cut them when
Python dependency expansion reveals a better seam.

- `application/config.py`: small validation behavior with clear fixtures.
- `inference/device_utils.py`: pure normalization logic plus platform defaults.
- `inference/game/alignment_utils.py`: pure list/numeric transforms.
- `inference/io/note_io.py`: deterministic TXT/CSV export behavior.
- `inference/quant/quantization.py`: larger algorithmic unit to defer until the
  fixture workflow is proven.

## Re-cut Signals

- A planned unit depends on heavy native/FFI behavior not needed for its public
  compatibility surface.
- A Python package boundary hides several independent capabilities that can be
  verified separately.
- A supposedly small unit requires shared fixtures or Rust data structures that
  should be extracted first.
- A direct crate replacement is worse than a narrow fixture-bound Rust
  implementation.
- A unit cannot name a rollback route after dependency expansion.

## Verification Commands

Run from the repository root:

```bash
cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check
cargo clippy --manifest-path rewrite-in-rust/rust/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml
RUSTDOCFLAGS="-D warnings" cargo doc --manifest-path rewrite-in-rust/rust/Cargo.toml --no-deps
uv run pytest tests/test_web_api.py
uv run python scripts/audit_vendored_sources.py
```

Manual Web integration still follows the existing project docs:

```bash
uv run python web_server.py
uv run python tests/test_api_integration.py
```

## Environment Facts

- Project Python is managed by uv and is Python 3.12.x.
- System `python` may be a different version and must not be used for project
  verification.
- Rust workspace uses Edition 2024.
- Rust MSRV is 1.85.
