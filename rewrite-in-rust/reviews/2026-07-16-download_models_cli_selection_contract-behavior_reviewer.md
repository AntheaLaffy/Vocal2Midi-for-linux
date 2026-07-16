# download_models_cli_selection_contract - behavior_reviewer

Date: 2026-07-16
Decision: pass

## Findings

No blocking behavior findings.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_download_models_cli_selection_contract.py`: pass.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml download_models_cli`: pass; `download_models_cli::tests::download_models_cli_selection_fixtures_match` passed.
- `git diff --check -- download_models.py rewrite-in-rust/fixtures/download_models_cli_selection_contract.jsonl rewrite-in-rust/bootstrap/check_download_models_cli_selection_contract.py rewrite-in-rust/rust/crates/v2m-core/src/download_models_cli.rs rewrite-in-rust/manifest.yaml`: pass.
- Focused source comparison: pass. Legacy `download_models.py:568`-`602` defines the parser defaults and choices mirrored by fixture lines 1-4 and Rust `download_models_cli.rs:60`-`110`. Legacy `download_models.py:608`-`638` defines list early exit, mkdir timing, GitHub catalog-order iteration, Qwen inclusion, failure aggregation, exit codes, and output shape mirrored by fixture lines 5-15 and Rust `download_models_cli.rs:121`-`199`.

## Behavior Notes

- Parse defaults, repeated `--only`, `--force`, `--qwen-source`, `--no-qwen`, and `--list` match the legacy parser for the declared fixture surface.
- Invalid `--only` and `--qwen-source` choices preserve parser exit status 2 and the checked error fragments.
- Selected GitHub downloads follow catalog order from `GITHUB_MODELS`, not user-provided `--only` order; duplicate selected GitHub names call the catalog row once.
- `--force` is propagated to both fake GitHub and fake Qwen download calls.
- `--no-qwen` suppresses Qwen in normal mode, including `--only qwen --no-qwen`, while still creating the experiments directory before reporting success.
- `--list` calls the fake `list_planned` boundary and exits before creating the experiments directory or calling downloads; `--no-qwen` maps the list source to `skip`.
- `--qwen-source skip` is still passed to fake `download_qwen` in normal download mode, matching legacy `main`.
- Failure aggregation is GitHub catalog order followed by Qwen, with exit code 1 and the checked stdout/stderr line shape when colors are disabled.

## Residual Risk

This gate validates the recorded compatibility surface, not a full `argparse` clone. Legacy `argparse` also accepts conveniences such as `--only=game`, `--qwen-source=skip`, abbreviated long options like `--no-q`, and `-h`/`--help`; those are not represented in the current fixtures or Rust parser. That is acceptable while legacy Python remains runtime owner, but a future promotion that replaces direct CLI parsing should either preserve those behaviors or explicitly narrow the public contract.

The checker uses fake `download_github_model`, `download_qwen`, and `list_planned` collaborators by design, so this review does not cover network fetches, archive extraction, external CLI invocation, or catalog display formatting.

## Promotion Note

The behavior gate passes for `download_models_cli_selection_contract`. This role does not block coordinator state update, subject to the separate required dependency-bootstrap and product-ergonomics reviews.
