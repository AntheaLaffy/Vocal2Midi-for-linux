# download_models_effectful_fetch_contract - product_ergonomics_reviewer

Date: 2026-07-16
Decision: pass-with-followups

## Findings

No blocking findings.

- Severity: low
- Location: rewrite-in-rust/fixtures/download_models_effectful_fetch_contract.jsonl:18
- Issue: The Qwen provider install workflow is proven in split pieces, but not
  as one operator-facing transcript. In production, `download_qwen_modelscope`
  calls `_pip_install` when the `modelscope` CLI is missing
  (`download_models.py:455`), and `_pip_install` prints the selected installer
  command (`download_models.py:405`). The provider fixture for install failure
  expects the missing-CLI warning and final manual-install failure, but omits
  the nested `• Installing with uv: -U modelscope` line. The Rust provider model
  likewise records `pip_calls` and emits the install-failure text without
  including `_pip_install` stdout (`rewrite-in-rust/rust/crates/v2m-core/src/download_models_effectful.rs:405`).
- Evidence: A non-mutating Python probe with `_resolve_cli` forced missing,
  `_have_uv` true, and `_run_cli` returning `1` produced stdout
  `["! modelscope CLI not found. Installing modelscope...", "• Installing with uv: -U modelscope"]`
  and stderr
  `["✗ Failed to install modelscope. Run manually: uv pip install -U modelscope  (or: pip install -U modelscope)"]`.
  The standalone installer fixture covers the install line at
  `rewrite-in-rust/fixtures/download_models_effectful_fetch_contract.jsonl:14`,
  but the provider fixture at line 18 and Rust state machine do not prove the
  combined sequence a CLI operator would see.
- Required fix: Before promoting Rust to own a direct provider-download CLI
  workflow, add integrated Qwen provider install fixtures for uv and pip,
  success and failure, or explicitly document that a future bridge delegates
  installer progress output to a separate reporter before provider output.

## Evidence Checked

- The unit boundary is confirmed as mocked effectful download control flow with
  legacy Python still the runtime owner:
  `rewrite-in-rust/manifest.yaml:734`,
  `rewrite-in-rust/dependencies/download_models_effectful_fetch_contract.yaml:1`,
  and `rewrite-in-rust/bootstrap/download_models_effectful_fetch_contract.md:5`.
- Source selection preserves the operator-facing Mainland China default and
  Hugging Face fallback: production tries ModelScope first at
  `download_models.py:535`, fixtures cover ModelScope success and fallback at
  `rewrite-in-rust/fixtures/download_models_effectful_fetch_contract.jsonl:23`
  through line 24, and Rust mirrors that flow at
  `rewrite-in-rust/rust/crates/v2m-core/src/download_models_effectful.rs:591`.
- Progress display is covered for known and unknown content length by
  `download_models.py:214`, fixture lines 4 through 5, and Rust lines 95
  through 126.
- Recovery text for skip/retry, size mismatch, corrupt zip, 404 release note,
  marker-missing extraction, Qwen manual install, CLI run failure, missing
  weights, and unknown Qwen source is represented in fixture lines 6 through
  12 and 18 through 26.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_download_models_effectful_fetch_contract.py`: passed.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml download_models_effectful`: passed; 1 matching Rust fixture test passed.
- `cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check`: passed.
- `git diff --check -- rewrite-in-rust/manifest.yaml rewrite-in-rust/rust/crates/v2m-core/src/lib.rs download_models.py`: passed with no whitespace diagnostics.
- `git diff --check --no-index /dev/null <unit new file>` for the fixture,
  Python checker, and Rust implementation: no whitespace diagnostics; exit code
  `1` is expected because `/dev/null` differs from each new file.
- Focused message/effect scans with `rg` over the source, fixture, checker,
  Rust implementation, dependency record, bootstrap doc, and record 0038:
  reviewed; matches were expected mocked boundaries, fixture text, or
  operator-facing messages.

## Residual Risk

This review proves the fixture-bound operator messages and source-selection
state model, not a live downloader. Real GitHub traffic, package installation,
external ModelScope/Hugging Face CLIs, archive extraction, and model-weight
inspection remain legacy-owned. The main-level final tips are covered by the
separate CLI-selection contract, while this unit covers the lower-level helper
messages that feed those tips.

## Promotion Note

This product ergonomics role does not block coordinator state update for the
current legacy-owned mocked seam. Keep the install-transcript follow-up attached
to any future promotion that makes Rust own a direct user-facing Qwen provider
download path.
