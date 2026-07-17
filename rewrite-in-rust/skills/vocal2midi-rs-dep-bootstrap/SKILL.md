---
name: vocal2midi-rs-dep-bootstrap
description: Expand Python dependencies and align capability coverage, Rust crate reuse, compatibility adapters, hand-written replacements, fixtures, and seam/bootstrap records for Vocal2Midi. Use before implementation when imports, native/FFI sources, dependency mismatch, fixture strategy, or manifest re-cut decisions affect a migration unit.
---

# Vocal2Midi Rust Dependency And Bootstrap

Prepare or re-cut migration units so implementation can start without guessing
about dependencies, fixtures, or seam shape.

## Required Context

Read these first:

- `rewrite-in-rust/README.md`
- `rewrite-in-rust/manifest.yaml`
- `rewrite-in-rust/resources.md`
- `rewrite-in-rust/notes.md`
- `rewrite-in-rust/reference/glossary.md`
- `pyproject.toml`
- `uv.lock`
- `requirements.txt`, `requirements-linux.txt`, `requirements-web.txt`
- `third_party/README.md`
- `third_party/sources/manifest.json`
- `third_party/sources/MISSING_SOURCES.md`
- `third_party/native_sources/manifest.json`
- `third_party/source_audit.json`
- Source refs and verification notes for the selected unit

Completion criterion: every dependency claim is grounded in a file or marked as
an assumption.

## Dependency Expansion Pass

Start from the selected Python source refs, then follow only the dependencies
needed to preserve the public behavior boundary.

Inspect:

- project imports and local helper calls
- Python package requirements and lockfile entries
- `third_party/sources/<package-version>/` for Python source distributions
- `third_party/upstream_sources/<package-version>/` for packages without sdists
- `third_party/native_sources/<library-version>/` for native/FFI source trees
- `third_party/cargo_vendor/<source-path>/` for Rust-backed Python package crates
- existing tests, fixtures, and public caller expectations

Stop when the unit can name its behavior, dependencies, fixtures, rollback, and
kept-legacy capabilities. Do not expand just to map the whole Python ecosystem.

Completion criterion: the unit boundary is confirmed or a re-cut is proposed.

## Capability Coverage

Decide by capability, not by Python package name.

- Pure validation, parsing, formatting, alignment, and deterministic algorithms
  can be hand-written in Rust against fixtures.
- A Rust crate does not need to be a perfect drop-in to be useful. When a crate
  covers a stable lower layer, such as tokenization, parsing events, IO,
  Unicode tables, numeric primitives, or data structures, prefer reusing that
  layer and writing a small compatibility adapter for the Python-specific
  behavior it does not cover.
- If Python source for the legacy behavior is available in project files,
  `third_party/`, or another recorded source snapshot, use it to implement only
  the observed semantic delta between the Rust crate and Python behavior. Do not
  reject a crate merely because its high-level API disagrees with the Python
  package.
- Heavy model inference, ONNX Runtime, Qwen ASR, PyQt, and Flask remain
  legacy-owned unless the manifest changes.
- Local vendored Python, Rust, and native sources may be used as references.
- Use `third_party/source_audit.json` and the source manifests to justify the
  exact reference source path used for a compatibility adapter or hand-written
  replacement.
- A direct crate replacement is optional. A narrow Rust replacement is preferred
  when package parity is broader, less stable, or harder to verify, but a
  partial crate plus a focused adapter is preferred over fully hand-writing the
  same lower-level machinery.
- If a planned unit is too broad after expansion, split it. If several units
  share the same required Rust data model or fixture harness, merge or extract a
  prerequisite unit.
- Do not install or add bridge dependencies that the selected seam does not need.

Completion criterion: kept-legacy capabilities and Rust-covered capabilities are
both named.

## Seam Default

The default seam is an independent Rust library plus fixtures under
`rewrite-in-rust/rust/`. Do not introduce PyO3, CLI/subprocess, HTTP, or a Python
runtime router during bootstrap unless the unit needs runtime promotion planning.

If a non-default seam is needed, record:

- seam kind
- public payload shape
- error mapping
- trace/log context
- repeated-call behavior
- rollback path

Completion criterion: the next writer can implement without choosing a new
architecture.

## Output

Write or update:

- `rewrite-in-rust/dependencies/<unit-id>.yaml`
- `rewrite-in-rust/bootstrap/<unit-id>.md` when a seam or fixture harness is
  proven
- `rewrite-in-rust/records/NNNN-*.md` when the decision changes a boundary or
  teaches a reusable lesson
- `rewrite-in-rust/manifest.yaml` when dependency discovery confirms, splits,
  merges, replaces, or defers provisional units

Use this dependency record shape:

```yaml
unit: unit_id
status: planned | active | done | blocked
capabilities:
  capability_name:
    legacy: "Python source or dependency"
    rust: "crate or narrow implementation"
    reason: "why this covers behavior"
seam:
  kind: library | ffi | cli | service | pipeline
  default_owner: legacy
  bridge_dependencies: []
fixtures:
  required:
    - "fixture or golden output needed before writer starts"
crate_reuse:
  candidates:
    - crate: "crate-name"
      covered_capabilities:
        - "what the crate can own safely"
      gaps:
        - "Python behavior not covered by crate"
      adapter_plan: "how the unit will patch the gaps using legacy source/fixtures"
      decision: use | reject | defer
inventory_impact:
  decision: confirmed | split | merged | renamed | deferred | replaced
  reason: "why the manifest unit boundary did or did not change"
hand_written_replacements:
  - capability: "behavior implemented directly in Rust instead of by crate"
    reference_sources:
      - "third_party/sources/package-version or native/upstream/cargo source path"
    reason: "why this is better than crate reuse plus an adapter"
legacy_kept:
  - capability: "capability retained in Python"
    reason: "why not moving now"
verification:
  - "copyable command"
```

Completion criterion: output records are specific enough for
`vocal2midi-rs-unit-writer`, or the manifest has been re-cut before writer work.

## Checks

Run non-mutating checks where useful:

```bash
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml
uv run python scripts/audit_vendored_sources.py
```

If a required command fails because the environment is missing or blocked, report
the exact blocker and do not pretend dependency alignment is complete.
