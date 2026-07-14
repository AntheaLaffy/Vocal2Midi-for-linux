# 0001 - Initialize Rust Rewrite Workspace

## Context

Vocal2Midi is currently an ONNX-first Python application with GUI, Web, CLI, and
library-style inference layers. The repository already documents the intended
dependency direction:

```text
gui -> application -> inference
web -> application -> inference
```

The rewrite should not disrupt current user workflows. It must proceed through
small units that can be independently verified and rolled back.

## Decision

Create `rewrite-in-rust/` as the durable control plane for the Rust rewrite.

The workspace will track:

- migration status in `manifest.yaml`
- resources in `resources.md`
- working constraints in `notes.md`
- durable decisions in `records/`
- shared terms in `reference/glossary.md`
- independent Rust implementation work under `rust/`

The first Rust workspace is not connected to Python production imports. It only
proves that Rust code can be built and tested independently.

## Consequences

- Existing Python behavior remains the compatibility source.
- Rust code can evolve without changing GUI, Web, CLI, or inference runtime
  behavior until a unit is explicitly promoted.
- Each future migration unit must define behavior fixtures before its manifest
  status can move beyond `reimplemented`.
- Review cost is controlled by batching behavior reviews at the stage level.

## Reversal

If this control-plane layout becomes wrong, keep the existing Python owner and
replace only the `rewrite-in-rust/` workspace documents. No production rollback
is needed because initialization does not route runtime calls to Rust.
