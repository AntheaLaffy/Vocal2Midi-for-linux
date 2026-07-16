# download_models_effectful_fetch_contract - dependency_bootstrap_reviewer

Date: 2026-07-16
Decision: pass

## Findings

No blocking findings.

## Dependency And Boundary Review

The manifest unit boundary is confirmed. The unit should not be split, merged,
deferred, or replaced.

- `rewrite-in-rust/manifest.yaml:734` marks
  `download_models_effectful_fetch_contract` reimplemented and confirmed, and
  `rewrite-in-rust/manifest.yaml:749` limits verification to mocked urllib,
  subprocess, CLI-return, and temp-file fixtures with no live network, package
  install, or model download.
- `rewrite-in-rust/records/0035-split-download-models-asset-safety.md:23`
  split the old mixed asset-safety unit into archive layout, catalog, CLI
  selection, and effectful-fetch units. `rewrite-in-rust/records/0038-confirm-download-models-effectful-fetch-boundary.md:24`
  confirms this unit as an independent Rust library seam with JSONL fixtures,
  and `:44` keeps GitHub, package installation, external CLI execution, archive
  extraction, and model-weight inspection out of verification.
- `rewrite-in-rust/dependencies/download_models_effectful_fetch_contract.yaml:4`
  through `:19` maps each capability to a mocked Rust state model:
  GitHub asset-size API fallback/cache, stream progress, GitHub download
  result mapping, and Qwen CLI/source-selection control flow. `:42` through
  `:50` keeps real network transfer, package installation, external CLI
  execution, archive extraction, model-weight inspection, catalog display,
  archive layout, and CLI selection under separate or legacy ownership.
- `rewrite-in-rust/bootstrap/download_models_effectful_fetch_contract.md:23`
  through `:40` names the compatibility surface, while `:61` through `:63`
  correctly avoids adding a Rust network client, archive crate, package
  installer, external CLI dependency, or model-runtime dependency for this
  fixture-backed unit.
- `rewrite-in-rust/bootstrap/check_download_models_effectful_fetch_contract.py:110`
  patches `urllib.request.urlopen` for GitHub API behavior, `:142` patches it
  for stream bytes into a temp file, `:175` patches `stream_download`,
  `extract_zip`, marker checks, and asset-size lookup for `download_github_model`,
  and `:366` patches CLI resolution, package installation, subprocess calls,
  cleanup, and Qwen weight checks. These are the right seams for the declared
  kept-legacy decisions.
- `rewrite-in-rust/fixtures/download_models_effectful_fetch_contract.jsonl:1`
  through `:26` covers the declared capability groups: GitHub size API/cache,
  stream progress, GitHub download success and failure mapping, `_run_cli`,
  `uv`/`pip` install command selection, CLI resolution, Qwen cleanup, provider
  CLI flows, and `download_qwen` source strategy.
- `rewrite-in-rust/rust/crates/v2m-core/src/download_models_effectful.rs:1`
  through `:6` documents the no-live-effects model, `:29` through `:627`
  implements the fixture-driven state machines, and `:633` consumes the same
  JSONL fixture table in Rust tests. The module has no real process, network,
  archive, package-install, or model-runtime dependency.
- `rewrite-in-rust/rust/crates/v2m-core/Cargo.toml:13` still only depends on
  `serde_json`, so this unit does not introduce missing network/archive/process
  crate risk. The import of catalog helpers at
  `rewrite-in-rust/rust/crates/v2m-core/src/download_models_effectful.rs:12`
  is acceptable because static catalog and formatting behavior is already split
  and fixture-backed under `download_models_asset_catalog_contract`; this unit
  does not retake catalog display or filesystem marker ownership.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_download_models_effectful_fetch_contract.py`: passed.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml download_models_effectful`: passed, 1 test passed.
- `rg -n "reqwest|ureq|curl|tokio|Command::|std::process|subprocess|urlopen|urllib|zip|Zip|install|modelscope|huggingface|NamedTemporaryFile|TemporaryDirectory|extract_zip|stream_download|shutil\\.which|pip" rewrite-in-rust/rust/Cargo.toml rewrite-in-rust/rust/crates/v2m-core/Cargo.toml rewrite-in-rust/rust/crates/v2m-core/src/download_models_effectful.rs rewrite-in-rust/bootstrap/check_download_models_effectful_fetch_contract.py rewrite-in-rust/fixtures/download_models_effectful_fetch_contract.jsonl`: inspected; matches are fake effect boundaries, fixture literals, documentation comments, or expected CLI argument strings. No Rust live network/process/archive/package APIs are present.
- `git diff --check -- rewrite-in-rust/manifest.yaml rewrite-in-rust/rust/crates/v2m-core/src/lib.rs`: passed.
- `rg -n "[ \\t]$" rewrite-in-rust/records/0038-confirm-download-models-effectful-fetch-boundary.md rewrite-in-rust/dependencies/download_models_effectful_fetch_contract.yaml rewrite-in-rust/bootstrap/download_models_effectful_fetch_contract.md rewrite-in-rust/fixtures/download_models_effectful_fetch_contract.jsonl rewrite-in-rust/bootstrap/check_download_models_effectful_fetch_contract.py rewrite-in-rust/rust/crates/v2m-core/src/download_models_effectful.rs rewrite-in-rust/rust/crates/v2m-core/src/lib.rs rewrite-in-rust/manifest.yaml`: passed with no matches.

## Residual Risk

This dependency-bootstrap pass does not prove exhaustive behavior or
diagnostic coverage for every network/process edge. The fixture table covers
the capability classes needed to justify the seam, but behavior/error/product
reviews should still decide whether to add more permutations such as non-404
HTTP errors, `TimeoutError`, empty streams, and symmetric ModelScope/Hugging
Face install/run/weight-failure cases before promotion. Real IO bridge design
also remains intentionally unapproved by this unit.

## Promotion Note

This dependency-bootstrap gate passes. The boundary is honest and confirmed:
legacy Python remains the runtime owner, no production bridge is introduced,
and the coordinator can use this role as non-blocking promotion evidence after
the other required review roles pass.
