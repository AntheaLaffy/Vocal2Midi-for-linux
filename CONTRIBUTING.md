# Contributing to Vocal2Midi

Thank you for contributing to Vocal2Midi. The canonical development guide is
[`docs/contributing.md`](docs/contributing.md). It covers the supported Python
and Rust toolchains, architecture boundaries, documentation rules, tests, and
the review checklist.

Before opening a change:

1. Read the [architecture guide](docs/architecture.md).
2. Run the narrow tests for the behavior you changed.
3. Run the Rust quality gate when the change touches
   `rewrite-in-rust/rust/`.
4. Update the owning documentation in the same change.
5. Read [SECURITY.md](SECURITY.md) before reporting a vulnerability.

Rust migration changes must also follow the control-plane workflow in
[`rewrite-in-rust/README.md`](rewrite-in-rust/README.md). Python remains the
runtime owner until the manifest records a reviewed promotion.
