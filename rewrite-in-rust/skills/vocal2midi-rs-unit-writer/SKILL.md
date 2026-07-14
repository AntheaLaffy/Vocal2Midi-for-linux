---
name: vocal2midi-rs-unit-writer
description: Implement exactly one confirmed Vocal2Midi Rust migration unit as the writer role. Use when dependency/capability discovery has accepted the unit boundary and the user asks to add fixtures, port Python library behavior into the Rust workspace, or prepare one unit for independent review. Do not use for review-only requests.
---

# Vocal2Midi Rust Unit Writer

Implement one migration unit while preserving Python behavior and preparing the
work for independent review.

## Required Context

Read these before editing:

- `rewrite-in-rust/README.md`
- `rewrite-in-rust/manifest.yaml`
- `rewrite-in-rust/resources.md`
- `rewrite-in-rust/notes.md`
- `rewrite-in-rust/reference/glossary.md`
- relevant `rewrite-in-rust/dependencies/<unit-id>.yaml` if present
- relevant `rewrite-in-rust/bootstrap/<unit-id>.md` if present
- source refs named by the selected unit
- existing tests around the touched behavior

Completion criterion: the writer knows the public behavior boundary, fixture
strategy, rollback route, and required reviews before editing.

## Boundary Gate

The manifest inventory is provisional. Before editing, confirm one of these is
true:

- `rewrite-in-rust/dependencies/<unit-id>.yaml` confirms the current boundary
- the unit is pure stdlib/local behavior with no meaningful third-party/native
  dependency expansion
- the coordinator explicitly records why dependency bootstrap is unnecessary

If the unit touches unclear Python dependencies, native/FFI behavior, broad
package APIs, or missing fixtures, stop and route to
`vocal2midi-rs-dep-bootstrap`.

Completion criterion: the writer is not blindly implementing a temporary module
list entry.

## Writer Workflow

1. Select or confirm exactly one unit from `manifest.yaml`.
2. Confirm the boundary gate above.
3. If the unit is `planned`, mark it `active` only when implementation actually
   starts.
4. Shrink the work to one independently verifiable behavior.
5. Add or update Python/Rust fixtures before changing production routing.
6. Implement in `rewrite-in-rust/rust/` unless the manifest explicitly permits a
   production bridge.
7. Preserve Python behavior, including accepted inputs, output shape, ordering,
   error behavior, and edge cases named by fixtures.
8. Run the narrowest useful Rust test, then relevant Python parity checks.
9. If Rust implementation exists but independent review is incomplete, mark the
   unit `reimplemented`, not `verified`.
10. Add a rewrite record when the work changes a boundary or reveals a reusable
   lesson.
11. Final response must request the required review roles.

Completion criterion: one unit is implemented or the blocker is concrete.

## Source Boundary

- Do not wire Rust into GUI, Web, CLI, or inference runtime paths unless the unit
  is explicitly in promotion planning.
- Do not add PyO3, CLI/subprocess, HTTP, or runtime router architecture as a
  shortcut.
- Do not migrate model inference, ONNX Runtime ownership, Qwen ASR, PyQt, or
  Flask as part of a small library unit.
- Do not broaden a unit to nearby modules because they are convenient.
- Do not preserve an initial unit boundary after discovery shows it should be
  re-cut.

## State Rules

- `planned`: described but not started.
- `active`: writer has started implementation work.
- `reimplemented`: Rust path and fixtures exist; independent review incomplete.
- `verified`: required review reports passed; writer must not set this for its
  own work.
- `promoted`: Rust is runtime owner; requires explicit promotion work.
- `blocked`: cannot proceed without a concrete missing decision or dependency.

Completion criterion: manifest status is factual, never aspirational.

## Checks

Prefer focused commands:

```bash
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml <unit-filter>
```

Run broader checks when the touched behavior can affect callers:

```bash
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml
uv run pytest tests/test_web_api.py
```

If tests cannot run, state exactly why.

## Completion Response

List changed files, tests run, manifest state, residual risks, and required
review roles. Do not claim review success for your own patch.
