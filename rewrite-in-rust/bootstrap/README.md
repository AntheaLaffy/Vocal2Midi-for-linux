# Bootstrap Records

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
