# download_models_effectful_fetch_contract - behavior_reviewer

Date: 2026-07-16
Decision: pass

## Findings

No behavior findings.

- Location: rewrite-in-rust/manifest.yaml:734
- Issue: The unit under review is explicitly `download_models_effectful_fetch_contract`, with legacy Python as the current owner and Rust as the target owner. The public policy is limited to mocked GitHub API lookup, stream progress, GitHub download mapping, Qwen CLI resolution/install/download flow, and cleanup of partial artifacts.
- Evidence: `rewrite-in-rust/manifest.yaml:742` defines the behavior policy; `rewrite-in-rust/manifest.yaml:749` lists the fixture coverage; `rewrite-in-rust/manifest.yaml:753` keeps `download_models.py` as the rollback/runtime owner.
- Required fix: None.

- Location: download_models.py:186
- Issue: The legacy Python behavior for this unit is represented by the expected helper set: GitHub API cache/fallback, stream progress, GitHub model download result mapping, CLI return/install/resolve helpers, Qwen cleanup, provider-specific Qwen downloads, and `download_qwen` source dispatch.
- Evidence: Legacy source spans `download_models.py:186` through `download_models.py:211` for asset-size cache/fallback, `download_models.py:214` through `download_models.py:244` for stream progress, `download_models.py:323` through `download_models.py:384` for GitHub download mapping, `download_models.py:387` through `download_models.py:428` for CLI/install/resolve helpers, and `download_models.py:431` through `download_models.py:543` for Qwen cleanup/download/source behavior.
- Required fix: None.

- Location: rewrite-in-rust/fixtures/download_models_effectful_fetch_contract.jsonl:1
- Issue: The durable fixture table covers the declared behavior surface without requiring live network, package installation, external CLIs, archive extraction, or model-weight inspection.
- Evidence: Fixture lines `rewrite-in-rust/fixtures/download_models_effectful_fetch_contract.jsonl:1` through `rewrite-in-rust/fixtures/download_models_effectful_fetch_contract.jsonl:26` cover GitHub API cache/fallback, stream progress, GitHub success/failure mapping, `_run_cli`, `_pip_install`, `_resolve_cli`, Qwen cleanup, provider CLI flows, and `download_qwen` source strategy. The boundary docs also state the mocked-effect limit at `rewrite-in-rust/bootstrap/download_models_effectful_fetch_contract.md:42` through `rewrite-in-rust/bootstrap/download_models_effectful_fetch_contract.md:44`.
- Required fix: None.

- Location: rewrite-in-rust/bootstrap/check_download_models_effectful_fetch_contract.py:110
- Issue: The Python checker proves the fixture expectations against legacy Python while replacing live effects with fakes and temp-directory fixtures.
- Evidence: The checker patches/records GitHub API behavior at `rewrite-in-rust/bootstrap/check_download_models_effectful_fetch_contract.py:110` through `rewrite-in-rust/bootstrap/check_download_models_effectful_fetch_contract.py:139`, stream behavior at `rewrite-in-rust/bootstrap/check_download_models_effectful_fetch_contract.py:142` through `rewrite-in-rust/bootstrap/check_download_models_effectful_fetch_contract.py:172`, GitHub download behavior at `rewrite-in-rust/bootstrap/check_download_models_effectful_fetch_contract.py:175` through `rewrite-in-rust/bootstrap/check_download_models_effectful_fetch_contract.py:246`, CLI/install/resolve behavior at `rewrite-in-rust/bootstrap/check_download_models_effectful_fetch_contract.py:249` through `rewrite-in-rust/bootstrap/check_download_models_effectful_fetch_contract.py:340`, and Qwen cleanup/download/strategy behavior at `rewrite-in-rust/bootstrap/check_download_models_effectful_fetch_contract.py:343` through `rewrite-in-rust/bootstrap/check_download_models_effectful_fetch_contract.py:502`.
- Required fix: None.

- Location: rewrite-in-rust/rust/crates/v2m-core/src/download_models_effectful.rs:28
- Issue: The Rust implementation mirrors the same fixture-driven state machine and remains outside production runtime IO.
- Evidence: Rust models asset parsing/cache at `rewrite-in-rust/rust/crates/v2m-core/src/download_models_effectful.rs:28` through `rewrite-in-rust/rust/crates/v2m-core/src/download_models_effectful.rs:82`, stream progress at `rewrite-in-rust/rust/crates/v2m-core/src/download_models_effectful.rs:94` through `rewrite-in-rust/rust/crates/v2m-core/src/download_models_effectful.rs:135`, GitHub download mapping at `rewrite-in-rust/rust/crates/v2m-core/src/download_models_effectful.rs:148` through `rewrite-in-rust/rust/crates/v2m-core/src/download_models_effectful.rs:295`, CLI/install/resolve/cleanup at `rewrite-in-rust/rust/crates/v2m-core/src/download_models_effectful.rs:297` through `rewrite-in-rust/rust/crates/v2m-core/src/download_models_effectful.rs:370`, and Qwen provider/source behavior at `rewrite-in-rust/rust/crates/v2m-core/src/download_models_effectful.rs:384` through `rewrite-in-rust/rust/crates/v2m-core/src/download_models_effectful.rs:627`. The test consumes the shared fixture table at `rewrite-in-rust/rust/crates/v2m-core/src/download_models_effectful.rs:633` through `rewrite-in-rust/rust/crates/v2m-core/src/download_models_effectful.rs:679`.
- Required fix: None.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_download_models_effectful_fetch_contract.py`: passed.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml download_models_effectful`: passed; 1 matching Rust test passed, 59 filtered out in `v2m_core`, and 0 matching tests ran in `v2m_quant_bridge`.
- `git diff --check -- rewrite-in-rust/manifest.yaml rewrite-in-rust/rust/crates/v2m-core/src/lib.rs`: passed.
- `rg -n "std::process|Command::|tokio::process|subprocess|reqwest|ureq|curl|urlopen|urllib|pip|install|modelscope|huggingface|NamedTemporaryFile|TemporaryDirectory|zipfile|ZipArchive|extract_zip|File::open|create_dir|remove_file|remove_dir|TcpStream" rewrite-in-rust/rust/crates/v2m-core/src/download_models_effectful.rs rewrite-in-rust/bootstrap/check_download_models_effectful_fetch_contract.py rewrite-in-rust/dependencies/download_models_effectful_fetch_contract.yaml rewrite-in-rust/bootstrap/download_models_effectful_fetch_contract.md`: reviewed; hits are boundary docs, Python fake harness/patch points, and Rust string/state-model values. The Rust unit does not perform live process, network, archive, package-install, temp-file, or filesystem mutation effects.

## Residual Risk

This behavior review proves fixture-backed parity only. It does not prove live GitHub availability, real network streaming, real package installation, real ModelScope/Hugging Face CLI behavior, real archive extraction, or model-weight inspection; those remain legacy-owned and explicitly out of scope in `rewrite-in-rust/bootstrap/download_models_effectful_fetch_contract.md:42` through `rewrite-in-rust/bootstrap/download_models_effectful_fetch_contract.md:44` and `rewrite-in-rust/dependencies/download_models_effectful_fetch_contract.yaml:42` through `rewrite-in-rust/dependencies/download_models_effectful_fetch_contract.yaml:50`.

This review did not perform the separate dependency-bootstrap, error-tracing, or product-ergonomics roles required by `rewrite-in-rust/manifest.yaml:743` through `rewrite-in-rust/manifest.yaml:747`.

## Promotion Note

The behavior role does not block coordinator state update for `download_models_effectful_fetch_contract`. Runtime promotion is not implied: `download_models.py` remains the rollback/runtime owner, and record `rewrite-in-rust/records/0038-confirm-download-models-effectful-fetch-boundary.md:44` through `rewrite-in-rust/records/0038-confirm-download-models-effectful-fetch-boundary.md:53` requires a later explicit bridge decision before Rust can own real network or process IO.
