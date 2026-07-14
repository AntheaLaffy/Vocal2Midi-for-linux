# 0004 - Index Hand-written Replacement Sources

## Context

The rewrite allows narrow hand-written Rust implementations when Python/Rust
dependency parity is too broad or unrealistic. That policy is only useful if the
agent knows where to inspect trustworthy source inputs.

The project already vendors Python sdists, upstream fallbacks, native/FFI
sources, and Rust-backed package crates under `third_party/`.

## Decision

Make the `third_party/` source directories first-class reference locations for
manual Rust replacements:

- `third_party/sources/<package-version>/`
- `third_party/upstream_sources/<package-version>/`
- `third_party/native_sources/<library-version>/`
- `third_party/cargo_vendor/<source-path>/`

Require `vocal2midi-rs-dep-bootstrap` dependency records to cite the exact
reference source path when recommending a hand-written replacement.

## Consequences

- Future agents should not stop at dependency manifests; they must inspect the
  actual vendored source when behavior depends on third-party implementation.
- `.venv` binaries are not the reference surface when source mirrors exist.
- Native/FFI mismatch can be handled by reading source trees first, then writing
  narrow fixture-bound Rust behavior.

## Reversal

If a source mirror is missing or stale, rerun the vendoring scripts and source
audit before using memory or binary behavior as a substitute.
