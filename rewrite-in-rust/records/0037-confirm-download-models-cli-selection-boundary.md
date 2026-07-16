# 0037 - Confirm Download Models CLI Selection Boundary

## Context

`download_models_cli_selection_contract` is the third slice from the former
`download_models_asset_safety` unit. The catalog and archive behavior are now
separate, leaving the command-line selection and `main` orchestration rules in
`download_models.py`:

- `parse_args` accepted choices and defaults;
- repeated `--only` handling;
- `--force`, `--qwen-source`, `--no-qwen`, and `--list` flags;
- list-mode source mapping and early exit;
- GitHub model iteration over catalog order;
- Qwen inclusion and skipping rules;
- failure aggregation and final exit code using injected download outcomes.

The behavior is deterministic when `download_github_model`, `download_qwen`,
and `list_planned` are replaced with fakes.

## Decision

Confirm `download_models_cli_selection_contract` as an independent Rust library
seam with JSONL fixtures. The Python checker patches effectful collaborators,
records fake calls, captures stdout/stderr, and uses a temp `EXPERIMENTS_DIR`.
The Rust unit mirrors the same parser state and main-plan outcomes.

The fixture intentionally preserves legacy quirks:

- `--only` can be repeated, but duplicate GitHub names still download once
  because `main` iterates catalog rows;
- selected GitHub downloads follow catalog order, not `--only` order;
- `--only qwen --no-qwen` performs no downloads and still succeeds;
- `--qwen-source skip` is only a list-display source label unless Qwen is
  otherwise skipped; in normal download mode `main` still calls
  `download_qwen("skip", force)`;
- list mode returns before creating `EXPERIMENTS_DIR` or calling downloads;
- failures are reported in GitHub catalog order, then Qwen.

## Consequences

This unit can validate command-line planning without invoking network access,
archive extraction, package installation, external CLIs, or model-weight
inspection. The behavior inside `download_github_model`, `download_qwen`, and
`list_planned` remains owned by the catalog/archive/effectful units.

## Reversal

Rollback is keeping `download_models.py` as the runtime owner. No production
bridge is introduced by this record.
