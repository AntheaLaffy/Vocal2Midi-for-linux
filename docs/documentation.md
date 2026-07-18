# Documentation Policy

This policy is for contributors who change user behavior, public APIs, the Rust
workspace, or migration evidence. It defines which document owns each fact and
how documentation is verified.

## Documentation Layers

| Layer | Source of truth | Update when |
| --- | --- | --- |
| Project entry | `README.md` and translated READMEs | User workflows, support status, or top-level commands change. |
| Contributor entry | `CONTRIBUTING.md`, `SECURITY.md` | Contribution or disclosure policy changes. |
| Maintainer guides | `docs/` | Architecture, setup, API, or documentation policy changes. |
| Rust workspace | `rewrite-in-rust/rust/README.md` and rustdoc | Toolchain, crate boundary, public API, error, panic, or safety behavior changes. |
| Migration control plane | `rewrite-in-rust/README.md`, `resources.md`, `notes.md`, `manifest.yaml` | Migration policy, inventory, ownership, or verification changes. |
| Migration evidence | `bootstrap/`, `dependencies/`, `records/`, `reviews/` | A migration decision or gate produces new evidence. |

The English `README.md` owns commands and links shared by the translated
READMEs. Keep `README.zh-CN.md` and `README.ja.md` structurally synchronized,
but translate prose for their readers.

## Current State and Historical Evidence

Maintainer guides describe the current repository. Update stale statements in
place.

Files under `rewrite-in-rust/records/` and `rewrite-in-rust/reviews/` are dated
evidence. Do not rewrite an old decision to match current state. Add a new
numbered record or review rerun and link it from `manifest.yaml`. Bootstrap
contracts and dependency YAML files are living unit contracts and may be
updated when a new record explains a changed boundary.

Generated rustdoc under `rewrite-in-rust/rust/target/doc/` is build output and
must not be committed.

## Markdown Standard

- Begin with one H1 and a short purpose statement naming the target reader.
- Describe current behavior, limits, errors, and side effects explicitly.
- Use paths relative to the repository root in commands and prose.
- Use fenced code blocks with a language tag.
- Keep examples copyable from the repository root unless the document says
  that a different working directory is required.
- Link to the owning document instead of copying a contract into several files.
- Do not describe planned or verified behavior as the current runtime default.
- Prefer descriptive link text over bare URLs.

## Rustdoc Standard

Rust documentation follows the conventions in the official [rustdoc
guide](https://doc.rust-lang.org/rustdoc/how-to-write-documentation.html):

- use `//!` for crate and module contracts and `///` for public items
- start with a concise summary, then document invariants and compatibility
  behavior
- use intra-doc links for Rust items
- add `# Errors` to public functions returning `Result`
- add `# Panics` only when callers can trigger a documented panic
- add `# Safety` to every `unsafe` public item
- prefer runnable examples; mark examples that must not run with the narrowest
  applicable rustdoc attribute

The migration crates are internal and fixture-oriented, but their public
surface is still treated as a maintained API. Each crate enables the
`missing_docs`, `clippy::missing_errors_doc`, and
`clippy::missing_panics_doc` lints, so warning-denying documentation and Clippy
builds reject incomplete public contracts. The crates currently forbid unsafe
code; a future exception requires a migration record and a documented `# Safety`
contract.

## Verification

Run Markdown lint for the maintained documents:

```bash
npx markdownlint-cli2@0.23.1
```

`.markdownlint-cli2.jsonc` includes maintained project Markdown and living
migration contracts. It excludes virtual environments, generated rustdoc,
mirrored third-party/upstream model cards, and append-only numbered
records/reviews. Line length is not enforced because copyable commands may be
intentionally long, and code spans may document significant surrounding spaces.
The local link checker still covers tracked and untracked historical evidence.

Build Rust documentation and reject rustdoc warnings:

```bash
RUSTDOCFLAGS="-D warnings" cargo doc \
  --manifest-path rewrite-in-rust/rust/Cargo.toml \
  --workspace --all-features --no-deps
```

Run the explicit public API documentation audit when changing lint policy:

```bash
RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc \
  --manifest-path rewrite-in-rust/rust/Cargo.toml \
  --workspace --all-features --no-deps
```

Check local links after moving or renaming documents. External links require a
network-enabled link checker and should not be the only evidence for a runtime
claim.

```bash
uv run python scripts/check_markdown_links.py
```
