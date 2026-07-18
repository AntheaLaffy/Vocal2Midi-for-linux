# Rust Rewrite Glossary

## Migration Unit

The minimum independently verifiable rewrite target. A unit must have a public
behavior boundary, fixture strategy, verification command, and rollback route.

## Provisional Inventory

The current manifest list of candidate units. It is a working hypothesis, not a
promise. Dependency expansion may split, merge, rename, defer, or replace units
before implementation.

## Owner

The implementation that currently owns runtime behavior for a unit.

- `legacy`: current Python implementation.
- `rust`: Rust implementation.

## Control Plane

The manifest and records that describe state, routing intent, verification, and
rollback. The control plane must not contain business logic.

## R0 Behavior Review

The first independent review gate. It checks old/new behavior parity, fixtures,
payloads, public errors, and compatibility. For this project, R0 is batched once
per stage after several small units are reimplemented.

## Promotion

The point where Rust becomes the default runtime owner for a unit. Promotion
requires behavior evidence, required reviews, and a clear rollback route.

## Rollback Route

The specific way to return behavior to the legacy Python owner. During the early
workspace stage, rollback is simply keeping production imports unchanged.

## Capability Coverage

Dependency matching by required behavior rather than by package name. For
example, Python numeric/list transformations may be covered by hand-written Rust
or a small Rust crate rather than a direct replacement for a Python package.

## Dependency Expansion

The discovery pass that follows Python imports, third-party dependencies,
vendored sources, native/FFI links, and public behavior until the unit boundary
is realistic enough to implement and verify.

## Narrow Replacement

A hand-written Rust implementation of one required behavior when direct
dependency parity is too broad, unavailable, or less verifiable. A narrow
replacement is acceptable only when fixture-bound.

## Fixture

A stable input and expected output used to prove Python/Rust behavior parity.
Fixtures should be narrow, deterministic, and tied to one migration unit.

## Verification

Evidence that Rust matches the named legacy behavior at a fixture-backed public
boundary. Verification does not change runtime ownership and must not be
described as production migration.

## Compatibility Adapter

A narrow layer that projects a maintained Rust crate's API or semantics onto the
legacy Python public contract. The adapter owns only the documented gap; the
crate owns the stable lower-level capability.

## Runtime Owner

The implementation selected by production callers by default. `current_owner`
in `manifest.yaml` is authoritative. A verified Rust implementation is not the
runtime owner until a promotion changes that field.

## Living Contract

A current-state document that may be updated in place, such as a bootstrap
contract, dependency YAML file, maintainer guide, or manifest entry. A changed
migration boundary also requires a new decision record.

## Historical Evidence

A dated decision or review report that preserves what was concluded from the
evidence available at that time. Historical evidence is append-only; corrections
use a later record or rerun.
