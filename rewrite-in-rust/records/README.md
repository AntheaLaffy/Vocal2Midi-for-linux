# Migration Decision Records

This directory contains append-only decisions and reusable lessons for the
Vocal2Midi Rust rewrite. It is for coordinators, implementers, and reviewers who
need to understand why the current manifest has its present shape.

## Naming

Use the next four-digit sequence number and a short kebab-case title:

```text
rewrite-in-rust/records/<NNNN>-short-decision-title.md
```

Never renumber existing records. A later decision supersedes an earlier one by
linking to it and explaining what changed.

## Required Content

- context and public behavior boundary
- evidence inspected
- decision and rejected alternatives
- manifest, dependency, fixture, or source impact
- verification commands and results
- runtime owner and rollback effect
- unresolved follow-up or residual risk

Records explain decisions; they do not replace `manifest.yaml`, living bootstrap
contracts, dependency YAML, or independent review reports.
