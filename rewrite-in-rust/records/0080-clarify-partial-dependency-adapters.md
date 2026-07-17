# 0080 - Clarify Partial Dependency Adapter Policy

Date: 2026-07-17

## Context

The config/YAML dependency probe showed a process problem: when candidate Rust
crates did not match PyYAML's high-level `safe_load` behavior exactly, the
rewrite workflow leaned too hard toward "do not add a crate" and "hand-write a
narrow replacement." That is too strict for dependencies where Rust already
covers a stable lower layer.

The desired policy is not perfect crate parity. If a Rust crate can own parsing,
events, IO, Unicode tables, numeric primitives, data structures, or another
well-scoped lower layer, and the Python dependency source is available, the
rewrite should consider using the crate plus a compatibility adapter for only
the semantic gaps.

## Decision

Update the Vocal2Midi rewrite skills so dependency alignment has three valid
routes:

1. direct crate coverage when fixtures prove it;
2. partial crate reuse plus a Python-source-guided compatibility adapter when a
   crate covers useful lower-level behavior but differs at the public semantic
   layer;
3. narrow hand-written Rust only for capabilities the crate cannot safely own or
   where crate reuse is larger, less stable, or harder to verify.

Dependency records should now include a `crate_reuse` section when this matters:
candidate crates, covered capabilities, semantic gaps, adapter plan, and
use/reject/defer decision.

## Impact

Future dependency/bootstrap work must not reject a crate merely because its
top-level API is not a Python package drop-in. For the HFA YAML work, this means
`hfa_pyyaml_safe_load_contract` should evaluate a parser/event crate plus a
PyYAML resolver/constructor/error adapter before falling back to a fully
hand-written loader.
