# Vocal2Midi Rust Rewrite Workspace

This directory is the control plane for the gradual rewrite of Vocal2Midi's
Python library layers into Rust.

The success condition is not "Rust code exists." The success condition is:

```text
At every migration state, Vocal2Midi remains runnable, testable, and rollbackable.
```

## Mission

Rewrite the reusable Python library layers into independently verified Rust
units while preserving the current user-facing Python workflows.

The initial scope is library-first:

- In scope: `application/`, `inference/`, and reusable script logic.
- Caller boundary: `gui/`, `web_server.py`, and WebSocket/HTTP handlers stay as
  Python callers until a specific migration unit promotes a replacement.
- Out of initial scope: replacing the desktop GUI, replacing Flask routing, or
  replacing model assets as a single large batch.

## Borrowed Ideas

This workspace borrows migration discipline from py2rs:

- behavior before architecture
- manifest-driven state
- reversible owner changes
- minimum independently verifiable units
- behavior review before promotion

It borrows stateful workspace habits from teach:

- resources before memory
- durable notes and glossary
- records for decisions and non-obvious lessons
- small scoped progression

It does not borrow a fixed py2rs router architecture. A Python router, PyO3
extension, subprocess bridge, or direct Rust library facade may be introduced
only when a migration unit needs it and records why it fits.

## Vocal2Midi Difference

This project is not a mostly-Rust backend rewrite like mvsep-rs. It is a
cross-language Python dependency rewrite with uncertain dependency expansion.
The module list in `manifest.yaml` is therefore a provisional working inventory,
not a commitment.

Before implementation, a unit may be split, merged, replaced, postponed, or
removed when dependency expansion shows a better boundary. This is expected,
not drift. The durable rule is minimum independent verification, not preserving
the initial unit names.

## Operating Rules

- Existing Python public behavior is the compatibility source until a unit says
  otherwise.
- Runtime/control-plane code must not contain business logic.
- Every unit must have a rollback route before promotion.
- Rust implementation units start outside the production path.
- Dependency alignment is capability-based, not package-name based.
- If Rust dependency coverage is poor, write a narrow Rust implementation
  against the unit's fixtures instead of forcing a one-to-one Python package
  replacement.
- Dependency expansion can invalidate the current manifest unit list. Re-cut the
  unit inventory when that produces smaller, more verifiable, or more realistic
  Rust work.
- Model inference, ONNX Runtime, Qwen ASR, PyQt, and Flask capabilities remain
  legacy-owned until explicitly planned.

## Directory Map

```text
rewrite-in-rust/
  README.md
  manifest.yaml
  resources.md
  notes.md
  dependencies/
  bootstrap/
  reviews/
  records/
  reference/
  rust/
  skills/
```

`rust/` is an independent Cargo workspace. It must be possible to test it without
starting the GUI, Web server, or full model pipeline.

The Rust workspace follows the conventions expected by Rust maintainers:

- crate contracts live in crate/module rustdoc
- README commands are copyable from the repository root
- MSRV is declared in `rust/Cargo.toml`
- style, lint, tests, and docs are all part of the handoff gate
- `unsafe` is avoided unless a migration record defines and reviews the
  invariant

See `rust/README.md` for workspace commands, crate ownership, and the
quantization bridge JSON contract.

## Baseline Verification

Run from the repository root:

```bash
cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check
cargo clippy --manifest-path rewrite-in-rust/rust/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml
RUSTDOCFLAGS="-D warnings" cargo doc --manifest-path rewrite-in-rust/rust/Cargo.toml --no-deps
uv run pytest tests/test_web_api.py
uv run python scripts/audit_vendored_sources.py
```

When a command depends on the uv environment, use the project's uv Python
environment, not the system `python` binary.
