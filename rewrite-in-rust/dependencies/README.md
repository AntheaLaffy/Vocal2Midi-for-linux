# Dependency Records

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
