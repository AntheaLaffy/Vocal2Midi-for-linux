# 0003 - Treat Unit Inventory As Provisional

## Context

The initial manifest contains a short list of migration candidates based on the
current Python source shape. That list is useful for orientation but risky as a
fixed backlog.

Vocal2Midi differs materially from mvsep-rs. The hard part is cross-language
dependency alignment across Python packages, native/FFI sources, model runtime
bindings, and hand-written Rust replacements when package parity is not the
right target.

Python dependency expansion is uncertain. A small-looking module may hide a
large native dependency. A large-looking module may contain several small
capabilities that should be split before implementation.

## Decision

Treat `manifest.yaml` units as a provisional inventory until dependency and
capability discovery confirms a unit boundary.

The rewrite process may substantially rewrite planned units. It may split,
merge, rename, defer, or replace them when discovery produces a more verifiable
boundary.

The durable rule is:

```text
Preserve behavior and independent verification, not initial module names.
```

## Consequences

- `vocal2midi-rs-rewrite` must allow re-cutting units before choosing the next
  implementation target.
- `vocal2midi-rs-dep-bootstrap` owns dependency expansion and may recommend
  manifest changes.
- `vocal2midi-rs-unit-writer` must not blindly implement a planned unit whose
  dependency/capability boundary has not been confirmed.
- `dependency_bootstrap_reviewer` should check whether a unit should have been
  re-cut before behavior review or promotion.

## Reversal

If the dependency graph stabilizes later, the manifest can be tightened into a
less provisional backlog. That should be recorded separately after several units
have passed behavior review.
