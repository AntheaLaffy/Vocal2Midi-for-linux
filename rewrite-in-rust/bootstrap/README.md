# Bootstrap Records

This directory contains living seam and fixture-harness contracts for migration
implementers and reviewers. It does not own runtime behavior.

Create one bootstrap record per migration unit only when a seam, fixture harness,
or repeated-call proof is needed.

Use:

```text
rewrite-in-rust/bootstrap/<unit-id>.md
```

The default seam is an independent Rust library plus fixtures. Do not add PyO3,
CLI/subprocess, HTTP, or runtime router bootstrap work unless the manifest or a
rewrite record explicitly chooses that architecture.

Bootstrap records may also prove that no bridge should exist yet. In that case,
state the fixture harness, repeated-call behavior, and rollback route that keep
the unit independently verifiable.

## Required Content

- public compatibility boundary and legacy source references
- included and excluded behavior
- fixture paths and input/output shape
- effect, error, panic, and cancellation behavior
- exact verification commands
- runtime owner and rollback route
- dependency record and manifest links

Update a bootstrap contract in place when its living seam changes, and add a
numbered record under `../records/` explaining the decision. Do not rewrite an
old review report to match the new contract.

## Verification

Run the checker named by the unit's `verification` list in `../manifest.yaml`,
then run the matching Rust test filter. Commands must work from the repository
root and must not require model downloads or live inference unless the contract
explicitly owns that effect.
