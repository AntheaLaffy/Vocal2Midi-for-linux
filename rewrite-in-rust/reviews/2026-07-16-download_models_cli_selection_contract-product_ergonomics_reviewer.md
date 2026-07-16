# download_models_cli_selection_contract - product_ergonomics_reviewer

Date: 2026-07-16
Decision: pass-with-followups

## Findings

No blocking findings.

- Severity: low
- Location: rewrite-in-rust/rust/crates/v2m-core/src/download_models_cli.rs:72
- Issue: The Rust parser preserves the declared selection-planning forms, but it
  is narrower than Python `argparse` for promotion as a user-facing CLI.
  `download_models.py` builds an `argparse.ArgumentParser` at
  `download_models.py:570`, so current Python accepts long-option equals forms
  such as `--only=game` and `--qwen-source=huggingface`, and `--help` exits 0
  with usage text. The Rust parser matches only literal `--only` and
  `--qwen-source` tokens at
  `rewrite-in-rust/rust/crates/v2m-core/src/download_models_cli.rs:72` and
  `rewrite-in-rust/rust/crates/v2m-core/src/download_models_cli.rs:85`, and
  otherwise reports `unrecognized arguments` at
  `rewrite-in-rust/rust/crates/v2m-core/src/download_models_cli.rs:101`.
- Evidence: `uv run python - <<'PY' ... dm.parse_args(["--only=game"]) ... PY`
  returns `Namespace(only=['game'], ...)`, `dm.parse_args(["--qwen-source=huggingface"])`
  returns `qwen_source='huggingface'`, and `dm.parse_args(["--help"])` exits 0
  with usage. No matching fixture rows exist in
  `rewrite-in-rust/fixtures/download_models_cli_selection_contract.jsonl:1`.
- Required fix: Before using this Rust parser as the production CLI parser, add
  fixtures and implementation coverage for `--flag=value`, help output, and
  missing-option-value diagnostics, or explicitly document that a future bridge
  still delegates full parsing/help rendering to Python.

## Evidence Checked

- `--qwen-source skip` in download mode is documented as a preserved oddity in
  `rewrite-in-rust/records/0037-confirm-download-models-cli-selection-boundary.md:34`.
  The legacy main path still calls `download_qwen(args.qwen_source, args.force)`
  at `download_models.py:625`, and the fixture asserts `qwen_calls:
  [{"source":"skip","force":false}]` plus the final failure/tips shape at
  `rewrite-in-rust/fixtures/download_models_cli_selection_contract.jsonl:12`.
- `--only qwen --no-qwen` remains a successful no-download plan: legacy creates
  `EXPERIMENTS_DIR` before selection completion at `download_models.py:612`,
  disables Qwen through `do_qwen` at `download_models.py:615`, and the fixture
  preserves that odd success output at
  `rewrite-in-rust/fixtures/download_models_cli_selection_contract.jsonl:11`.
- List mode exits before directory creation: legacy returns from list mode at
  `download_models.py:608` through `download_models.py:610`, before the mkdir at
  `download_models.py:612`. The checker captures `mkdir_called` at
  `rewrite-in-rust/bootstrap/check_download_models_cli_selection_contract.py:122`,
  and fixtures cover both normal list source propagation and `--no-qwen` mapping
  to `skip` at
  `rewrite-in-rust/fixtures/download_models_cli_selection_contract.jsonl:14`
  through
  `rewrite-in-rust/fixtures/download_models_cli_selection_contract.jsonl:15`.
- Repeated `--only` remains append-based, while GitHub execution follows catalog
  order once per catalog row. Legacy uses `action="append"` at
  `download_models.py:575` and catalog membership iteration at
  `download_models.py:619`. Fixtures cover catalog-order execution and duplicate
  suppression at
  `rewrite-in-rust/fixtures/download_models_cli_selection_contract.jsonl:6`
  through
  `rewrite-in-rust/fixtures/download_models_cli_selection_contract.jsonl:7`.
- Failure aggregation and operator tips preserve the main-level output shape:
  legacy aggregates failures at `download_models.py:617` through
  `download_models.py:636`, and Rust mirrors the blank-line, stderr, and tips
  rows at
  `rewrite-in-rust/rust/crates/v2m-core/src/download_models_cli.rs:186`
  through
  `rewrite-in-rust/rust/crates/v2m-core/src/download_models_cli.rs:199`.
- The manifest keeps this unit scoped to fake collaborators and no live model
  side effects at `rewrite-in-rust/manifest.yaml:714` through
  `rewrite-in-rust/manifest.yaml:732`.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_download_models_cli_selection_contract.py`: passed.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml download_models_cli`: passed; 1 matching Rust fixture test passed.
- Focused reads of `download_models.py`, the CLI fixture JSONL, Python checker,
  Rust implementation, bootstrap doc, record, and manifest block: completed.
- `git diff --check -- download_models.py rewrite-in-rust/fixtures/download_models_cli_selection_contract.jsonl rewrite-in-rust/bootstrap/check_download_models_cli_selection_contract.py rewrite-in-rust/rust/crates/v2m-core/src/download_models_cli.rs rewrite-in-rust/bootstrap/download_models_cli_selection_contract.md rewrite-in-rust/manifest.yaml`: passed.
- Additional whitespace scan for untracked new unit artifacts with
  `git diff --check --no-index /dev/null <file>`: passed with no output.

## Residual Risk

This review proves the declared fixture-bound selection behavior, not a
production CLI bridge. Full argparse help rendering, equals-form parsing, and
missing-value diagnostics need explicit treatment before Rust owns the
user-facing CLI. Collaborator-level messages from `download_qwen`,
`download_github_model`, and `list_planned` remain owned by the catalog,
archive, and effectful-fetch units.

## Promotion Note

The product ergonomics gate passes with a promotion follow-up. The follow-up is
not blocking while Python remains runtime owner, but it should be closed or
explicitly documented before a direct Rust CLI parser promotion.
