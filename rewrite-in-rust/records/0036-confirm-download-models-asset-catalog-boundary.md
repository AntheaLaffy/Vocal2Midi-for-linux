# 0036 - Confirm Download Models Asset Catalog Boundary

## Context

`download_models_asset_catalog_contract` is the second slice from the former
`download_models_asset_safety` unit. The archive-layout behavior is already
isolated, leaving a pure catalog/display surface in `download_models.py`:

- static `GithubModel` entries and `GITHUB_MODEL_BY_NAME` keys;
- `asset_url` formatting;
- `human_size` formatting;
- `target_has_model` marker checks;
- `qwen_has_weights` immediate-entry marker checks;
- `list_planned` display rows using injected asset sizes and temp filesystem
  state.

The behavior is deterministic when GitHub asset sizes and filesystem state are
injected. It does not require network access, package installation, archive
extraction, external CLIs, or model-weight inspection.

## Decision

Confirm `download_models_asset_catalog_contract` as an independent Rust library
seam with a JSONL fixture table. The Python checker patches only module-level
catalog inputs and temp paths, while Rust mirrors the same display and marker
rules directly.

The fixture intentionally preserves legacy edge behavior:

- `target_has_model` uses `Path.exists()`, so a directory with the marker name
  counts as present;
- `qwen_has_weights` scans only immediate children of the Qwen directory;
- an immediate directory ending in `.bin` counts as a weight marker because the
  legacy helper checks entry names, not file types;
- `asset_url` concatenates path segments without URL encoding;
- `list_planned("skip")` renders the Qwen source as `skipped`.

## Consequences

The Rust unit can validate catalog metadata and dry-run display behavior without
contacting GitHub or reading real model directories. CLI selection, real size
lookup, stream downloads, package installation, Qwen CLI execution, cleanup, and
final exit-code aggregation remain split into later units.

## Reversal

Rollback is keeping `download_models.py` as the runtime owner. No production
bridge is introduced by this record.
