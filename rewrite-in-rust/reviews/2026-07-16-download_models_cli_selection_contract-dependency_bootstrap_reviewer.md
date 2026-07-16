# download_models_cli_selection_contract - dependency_bootstrap_reviewer

Date: 2026-07-16
Decision: pass

## Findings

No blocking findings.

## Dependency And Boundary Review

The manifest boundary is confirmed. The unit should not be split, merged,
deferred, or replaced.

- `rewrite-in-rust/manifest.yaml:714` marks the unit reimplemented and
  confirmed, with verification scoped to fake
  `download_github_model`/`download_qwen`/`list_planned` outcomes and no real
  streamed downloads, external CLIs, or production model directories.
- `rewrite-in-rust/records/0037-confirm-download-models-cli-selection-boundary.md:23`
  confirms a library seam for parser state and `main` planning, and
  `:42` keeps network, archive extraction, package installation, external
  CLIs, and model-weight inspection out of this unit.
- `rewrite-in-rust/dependencies/download_models_cli_selection_contract.yaml:18`
  requires JSONL parity fixtures, fake collaborators, and temp paths; `:32`
  documents split ownership for catalog, archive, effectful fetch, and model
  weight behavior.
- `rewrite-in-rust/bootstrap/download_models_cli_selection_contract.md:29`
  excludes catalog display rows, GitHub requests, streamed downloads, archive
  extraction, package installation, external ModelScope/Hugging Face CLIs, Qwen
  cleanup, and model weight/format execution; `:36` limits dependency expansion
  to stdlib parsing/capture/temp-path behavior.
- `rewrite-in-rust/bootstrap/check_download_models_cli_selection_contract.py:90`
  patches `EXPERIMENTS_DIR`, `download_github_model`, `download_qwen`,
  `list_planned`, and `_USE_COLOR`; `:96` confines filesystem effects to a
  temporary directory.
- `rewrite-in-rust/rust/crates/v2m-core/src/download_models_cli.rs:59`
  implements parser state; `:113` simulates `main` planning with fake outcomes
  and does not use network, process, archive, package, or model-weight APIs.
- `rewrite-in-rust/rust/crates/v2m-core/src/lib.rs:11` only exposes the Rust
  module inside the independent rewrite crate; no production bridge is added.

The Rust module depends on the already separated catalog module for model names
and ordering. That is consistent with this unit's CLI-selection responsibility
and does not pull catalog display, marker checks, network, archive extraction,
or model-weight behavior into this boundary.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_download_models_cli_selection_contract.py`: passed.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml download_models_cli`: passed, 1 test passed.
- Scoped effect scan over dependency/bootstrap/checker/Rust files: passed. The broad scan only found documented exclusions, fake collaborator names, temporary path handling, and the literal `modelscope`/`huggingface` qwen-source choices; the tighter code scan found no actual network, process, archive, package-install, or model-weight API use.
- `git diff --check -- rewrite-in-rust/manifest.yaml rewrite-in-rust/records/0037-confirm-download-models-cli-selection-boundary.md rewrite-in-rust/dependencies/download_models_cli_selection_contract.yaml rewrite-in-rust/bootstrap/download_models_cli_selection_contract.md rewrite-in-rust/fixtures/download_models_cli_selection_contract.jsonl rewrite-in-rust/bootstrap/check_download_models_cli_selection_contract.py rewrite-in-rust/rust/crates/v2m-core/src/download_models_cli.rs rewrite-in-rust/rust/crates/v2m-core/src/lib.rs`: passed.
- `rg -n "[ \t]$" ...` over the untracked scoped unit files: passed with no trailing-whitespace matches.

## Residual Risk

This dependency-bootstrap pass does not prove full argparse formatting parity
for every parser-generated edge case such as help text or missing option values;
that belongs to behavior/product review if those surfaces are promoted. The
promotion order should keep the catalog module available before this CLI module,
because CLI choices and GitHub ordering intentionally reuse catalog data.

## Promotion Note

This dependency-bootstrap gate passes. The boundary is honest and remains
confirmed, with legacy Python still the runtime owner and no production bridge
introduced by this unit.
