# Adopt Rust Documentation Standard

Date: 2026-07-18

## Context

The repository already had user, maintainer, migration, and rustdoc material,
but current-state guides still described the first quantization batch while the
manifest had grown to 66 verified units. Rust modules had crate/module comments,
yet the public API emitted 966 `missing_docs` warnings and did not enforce
Clippy's public error/panic documentation lints.

## Decision

- Treat root READMEs, contributor/security entrypoints, `docs/`, Rust workspace
  guidance, and living migration contracts as maintained current-state docs.
- Treat numbered records and dated review reports as append-only historical
  evidence. Validate their local links without rewriting old conclusions.
- Treat mirrored third-party sources and the upstream Qwen model card as
  upstream material, outside project Markdown formatting ownership.
- Follow rustdoc conventions for crate/module contracts, all public items,
  intra-doc links, examples, `# Errors`, `# Panics`, and `# Safety`.
- Enable `missing_docs`, `clippy::missing_errors_doc`, and
  `clippy::missing_panics_doc` in both workspace crates.
- Forbid unsafe code in the current migration crates. Any future exception
  requires its own record, safety contract, and review.
- Store Cargo README, repository, and crate descriptions in workspace/package
  metadata even though the crates remain `publish = false`.
- Use `.markdownlint-cli2.jsonc` for maintained Markdown and
  `scripts/check_markdown_links.py` for every tracked or untracked project
  Markdown file.

## Documentation Impact

- Added `CONTRIBUTING.md`, `SECURITY.md`, and `docs/documentation.md`.
- Synchronized the English, Simplified Chinese, and Japanese README document
  maps and Rust commands.
- Updated architecture, contribution, platform, Web security, Rust workspace,
  migration resource, glossary, evidence-directory, third-party, and romaji
  model-bundle documentation.
- Documented every public Rust item and the error/panic contracts reported by
  the strict Rust documentation lints.
- Preserved existing numbered records and dated reviews unchanged.

## Runtime and Rollback

No runtime route or migration owner changed. Legacy Python remains the current
owner for all 66 manifest units, and the quantization JSON bridge remains
explicitly opt-in. Documentation rollback is a normal revert of this policy;
runtime rollback is unaffected.

## Verification

- `cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check`:
  pass.
- `cargo clippy --manifest-path rewrite-in-rust/rust/Cargo.toml --workspace
  --all-targets --all-features -- -D warnings`: pass.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml --workspace
  --all-features`: pass, 131 unit tests and 2 doctests.
- `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --manifest-path
  rewrite-in-rust/rust/Cargo.toml --workspace --all-features --no-deps`: pass.
- `npx markdownlint-cli2@0.23.1`: pass, 93 maintained Markdown files.
- `uv run python scripts/check_markdown_links.py`: pass, 516 Markdown files.
- `cargo metadata --manifest-path rewrite-in-rust/rust/Cargo.toml --no-deps
  --format-version 1`: both packages expose description, README, repository,
  license, edition, and MSRV metadata.

## Residual Risk

The deterministic link gate validates repository-local targets. It does not
prove that every external website is reachable at all times. External links are
kept descriptive and point to official or upstream project locations.
