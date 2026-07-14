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
