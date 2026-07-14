# 0002 - Add Project Rewrite Skills

## Context

The rewrite workspace needs repeatable entrypoints so future sessions can start
work without re-deriving the process. The mvsep-rs rewrite skills provide a good
pattern: one coordinator, one writer role, and one review gate.

Vocal2Midi has an additional early risk: Python dependency and native/FFI source
coverage is broad, and the rewrite explicitly allows capability-level Rust
implementations when package-level parity does not fit.

## Decision

Create four project-specific skills:

- `vocal2midi-rs-rewrite`
- `vocal2midi-rs-dep-bootstrap`
- `vocal2midi-rs-unit-writer`
- `vocal2midi-rs-review-gate`

The fourth skill separates dependency/capability/seam preparation from the code
writer. This keeps small implementation units from quietly absorbing dependency
policy, bridge architecture, or fixture design decisions.

Store the authoritative copies under `rewrite-in-rust/skills/` and mirror them
to `/home/fuurin/.claude/skills/` for direct invocation in later sessions.

## Consequences

- Future work can begin with `$vocal2midi-rs-rewrite`.
- Dependency and bootstrap decisions have a dedicated record path before code
  migration starts.
- Writer and reviewer separation is encoded in skill boundaries.
- The repository copy remains reviewable and versionable even if the user skill
  root is regenerated.

## Reversal

If four skills become too much ceremony, merge `vocal2midi-rs-dep-bootstrap`
back into the coordinator and keep the writer/review split. Do not merge writer
and review roles.
