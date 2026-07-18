# Rust Rewrite Review Reports

Review reports are durable promotion evidence. A reviewer writes one report for
one unit and one role. Reports are append-only historical evidence: fix a stale
or failed conclusion with a dated rerun instead of editing the old decision.

## Naming

```text
rewrite-in-rust/reviews/YYYY-MM-DD-<unit-id>-<role>.md
```

## Roles

- `behavior_reviewer`
- `dependency_bootstrap_reviewer`
- `error_tracing_reviewer`
- `data_algorithm_reviewer`
- `rust_style_reviewer`
- `architecture_reviewer`
- `product_ergonomics_reviewer`

## Report Format

```md
# <unit-id> - <role>

Date: YYYY-MM-DD
Decision: pass | pass-with-followups | fail

## Findings

- Severity: critical | high | medium | low
- Location: path:line
- Issue: what is wrong
- Evidence: command, fixture, source, or diff proof
- Required fix: concrete next action

## Checks

- `command`: result

## Residual Risk

What remains unproven after this review.

## Promotion Note

Whether this role blocks promotion.
```

Reports may include "No findings" only after checking the role's full scope.

For `dependency_bootstrap_reviewer`, the report must also state whether the
manifest unit boundary is confirmed, should be split, should be merged, should be
deferred, or should be replaced.

## Evidence Rules

- Record the exact command and result, not only "tests pass."
- Link findings to repository-root-relative paths and line numbers when stable.
- Separate observed evidence from reviewer inference.
- State skipped checks and residual risk explicitly.
- Never promote a unit solely because a report contains no findings; the
  manifest must name every required role and its evidence.
- A reviewer must not patch production code in the same review role.

## Documentation Review

Rust style review includes crate/module rustdoc, public `Result` error sections,
panic and safety contracts, intra-doc links, examples where practical, and the
commands in [`docs/documentation.md`](../../docs/documentation.md).
