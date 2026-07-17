---
name: vocal2midi-rs-review-gate
description: Review one Vocal2Midi Rust migration unit quality gate without writing production code. Use for behavior parity review, dependency/seam review, error tracing review, data or algorithm review, Rust style audit, architecture review, product ergonomics review, or promotion readiness.
---

# Vocal2Midi Rust Review Gate

Run one independent review role for one migration unit and write a durable report
under `rewrite-in-rust/reviews/`.

## Required Context

Read these first:

- `rewrite-in-rust/README.md`
- `rewrite-in-rust/manifest.yaml`
- `rewrite-in-rust/resources.md`
- `rewrite-in-rust/notes.md`
- `rewrite-in-rust/reviews/README.md`
- relevant records under `rewrite-in-rust/records/`
- relevant dependency/bootstrap records
- diff and files touched by the unit
- tests and fixtures relevant to the unit

Completion criterion: review findings cite code, fixtures, docs, or commands.

## Choose Exactly One Role

- `behavior_reviewer`: Python/Rust parity, public inputs, outputs, ordering,
  errors, fixtures, and rollback.
- `dependency_bootstrap_reviewer`: capability coverage, kept-legacy decisions,
  seam choice, provisional inventory changes, crate reuse plus compatibility
  adapter choices, hand-written replacement choices, and missing crate/fixture
  risk.
- `error_tracing_reviewer`: structured errors, context, redaction, logs, and
  diagnosability.
- `data_algorithm_reviewer`: data structures, numeric behavior, complexity,
  benchmarks, and algorithmic assumptions.
- `rust_style_reviewer`: Rust module shape, ownership, visibility, tests,
  warnings, and maintainability.
- `architecture_reviewer`: owner boundaries, control-plane purity, bridge shape,
  and promotion risk.
- `product_ergonomics_reviewer`: CLI/Web/GUI workflow impact, user-visible
  messages, recovery, and operational ergonomics.

If the user asks for all reviews, run behavior first and state that remaining
roles must be separate passes or separate agents.

Completion criterion: one role and one unit are explicit.

## Review Workflow

1. Confirm unit id and review role.
2. Confirm the unit stayed inside its minimum boundary, or that dependency
   expansion justifies the re-cut boundary.
3. Confirm writer/reviewer separation.
4. For dependency reviews, judge coverage by capability, not package-name or
   top-level API parity. A Rust crate can be acceptable when it owns a stable
   lower layer and the dependency record names Python-specific gaps, reference
   source, compatibility adapter plan, and fixture evidence.
5. Inspect only the scope needed for the chosen role.
6. Run non-mutating checks where useful.
7. Report findings first, ordered by severity, with file/line references.
8. Write `rewrite-in-rust/reviews/YYYY-MM-DD-<unit-id>-<role>.md`.
9. Use decision `pass`, `pass-with-followups`, or `fail`.
10. Do not mark the manifest `verified`; the coordinator updates state after
   required reviews pass.

Completion criterion: the report can be used as durable promotion evidence.

## Boundaries

- Do not edit production code.
- Do not combine multiple review roles in one report.
- Do not rely on the writer's explanation when files or tests can answer.
- Do not approve a new bridge architecture without a matching record and
  rollback route.
- Do not approve a unit boundary merely because it appeared in the initial
  manifest; check dependency expansion evidence when that is in scope.
- Do not fail a dependency decision merely because no Rust crate is a perfect
  Python package drop-in. Fail it only when the selected crate-owned layer,
  compatibility adapter, Python-source reference, or fixtures are insufficient
  for the claimed behavior.
- If no issue is found, say so and document residual risk.

## Completion Response

Summarize decision, report path, highest-severity findings, checks run, and
whether the unit is ready for coordinator state update.
