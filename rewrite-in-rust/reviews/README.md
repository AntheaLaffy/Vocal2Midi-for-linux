# Rust Rewrite Review Reports

Review reports are durable promotion evidence. A reviewer writes one report for
one unit and one role.

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
