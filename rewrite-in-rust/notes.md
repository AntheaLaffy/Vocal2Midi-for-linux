# Rust Rewrite Notes

## User Constraints

- The rewrite target is the Python library layer, not a one-shot rewrite of the
  whole application.
- If Python third-party dependencies or their FFI-linked native sources are
  available locally, they may be used as references.
- Dependency mismatches should be handled by capability-level Rust
  implementations when practical.
- This project has harder cross-language dependency alignment than mvsep-rs.
  Python dependency expansion may reveal native/FFI or algorithm boundaries that
  require changing the rewrite plan.
- The current module/unit list is temporary. Planned units are allowed to be
  split, merged, renamed, deferred, or replaced after discovery.
- Hand-written Rust replacements are acceptable when dependency parity is
  unrealistic, as long as they are narrow and fixture-bound.
- Reference source for hand-written replacements is under `third_party/`:
  Python sdists in `third_party/sources/`, upstream fallbacks in
  `third_party/upstream_sources/`, native/FFI sources in
  `third_party/native_sources/`, and Rust-backed package crates in
  `third_party/cargo_vendor/`.
- The rewrite must be stepwise and stateful.
- Each small unit should focus on one independently verifiable thing.
- Review frequency should be much lower than one review per unit: one R0
  behavior review per stage is the default.
- Resources, references, records, and style constraints must be indexed like a
  teach workspace.

## Current Repository Facts

- Intended dependency direction is `gui -> application -> inference` and
  `web -> application -> inference`.
- Existing fast test command is `uv run pytest tests/test_web_api.py`.
- Vendored dependency sources are already represented under `third_party/`.
- The initial Rust workspace is intentionally not connected to the production
  Python runtime.
- `rewrite-in-rust/manifest.yaml` is a control plane, not a frozen project plan.

## Defaults Chosen

- Directory name: `rewrite-in-rust/`.
- Scope boundary: library-first.
- Bridge default: no PyO3, router, or subprocess bridge at initialization time.
- Owner default: legacy Python remains runtime owner for all planned units.
- Review default: stage-level R0 behavior review after 3-5 reimplemented units.
- Inventory default: dependency discovery may rewrite planned units before
  implementation starts.
