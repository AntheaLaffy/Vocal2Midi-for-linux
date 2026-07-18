# Dependency Records

This directory contains machine-readable capability decisions for migration
coordinators, implementers, and dependency reviewers.

Create one dependency record per migration unit when dependency or capability
coverage decisions matter.

Use:

```text
rewrite-in-rust/dependencies/<unit-id>.yaml
```

Minimum shape:

```yaml
unit: unit_id
status: planned | active | done | blocked
capabilities: {}
seam:
  kind: library
  default_owner: legacy
  bridge_dependencies: []
fixtures:
  required: []
inventory_impact:
  decision: confirmed | split | merged | renamed | deferred | replaced
  reason: ""
hand_written_replacements: []
legacy_kept: []
verification: []
```

The record must explain why Rust covers a capability or why legacy Python keeps
it.

If dependency expansion changes the planned module list, update
`rewrite-in-rust/manifest.yaml` and add a rewrite record explaining the re-cut.

## Decision Rules

1. Use a maintained Rust crate directly when fixtures prove the required
   behavior.
2. Reuse a crate for a stable lower layer and add a narrow compatibility adapter
   when Python source explains the semantic gap.
3. Hand-write only uncovered capabilities unless a recorded tradeoff justifies
   a complete narrow replacement.
4. Keep legacy behavior when the public boundary cannot be verified safely.

Each source expansion must name the public call path that makes it relevant.
Lockfile transitivity alone does not justify expanding into second-layer or
deeper dependencies.

## Verification

Validate YAML parsing and the repository source inventory from the repository
root:

```bash
uv run python scripts/audit_vendored_sources.py
uv run python -c \
  "import glob, yaml; [yaml.safe_load(open(path)) for path in glob.glob('rewrite-in-rust/dependencies/*.yaml')]"
```
