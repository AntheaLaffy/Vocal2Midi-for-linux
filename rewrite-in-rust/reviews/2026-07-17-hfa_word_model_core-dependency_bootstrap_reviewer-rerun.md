# hfa_word_model_core - dependency_bootstrap_reviewer rerun

Date: 2026-07-17
Decision: pass

Unit: `hfa_word_model_core`
Role: `dependency_bootstrap_reviewer`

## Findings

No dependency/bootstrap findings.

Evidence:

- The prior command-filter finding is closed. All current authoritative command entries use `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_word` (`rewrite-in-rust/dependencies/hfa_word_model_core.yaml:51`, `rewrite-in-rust/bootstrap/hfa_word_model_core.md:78`, `rewrite-in-rust/records/0067-implement-hfa-word-model-core.md:41`, `rewrite-in-rust/manifest.yaml:1326`). The filter now lists and executes both the shared fixture-table test and the exponent-format regression test (`rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:503`, `rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:546`). The earlier review remains an intentionally historical record of the resolved mismatch.
- Boundary decision remains confirmed. `hfa_word_model_core` is the sole owner of canonical Rust `Phoneme` and `Word` construction/local mutations. `hfa_wordlist_collection_ap_core` must introduce the sole `WordList` and log storage over those types, and `hfa_wordlist_finalize_core` must extend that collection rather than duplicate or translate it (`rewrite-in-rust/records/0066-split-hfa-word-lifecycle.md:46`, `rewrite-in-rust/bootstrap/hfa_word_model_core.md:36`, `rewrite-in-rust/bootstrap/hfa_wordlist_collection_ap_core.md:36`, `rewrite-in-rust/bootstrap/hfa_wordlist_finalize_core.md:39`, `rewrite-in-rust/manifest.yaml:1329`, `rewrite-in-rust/manifest.yaml:1353`). The current Rust module still defines only `Phoneme` and `Word`, not a premature `WordList` (`rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:66`, `rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:99`).
- The expanded 14-case fixture remains inside the selected first-unit capability boundary. The additions lock Python's empty-phoneme `move_start` chained-comparison behavior: negative, negative-infinity, and NaN values short-circuit to warning delivery before indexing, while nonnegative starts and every empty-phoneme `move_end` reach the legacy `IndexError` surface (`rewrite-in-rust/fixtures/hfa_word_model_core.jsonl:9`, `rewrite-in-rust/fixtures/hfa_word_model_core.jsonl:10`, `rewrite-in-rust/fixtures/hfa_word_model_core.jsonl:14`, `inference/HubertFA/tools/align_word.py:91`, `rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:200`). These are local Word mutation semantics; they do not pull WordList, decoder math, aggregation, or model execution into the unit.
- Special-float fixture coverage remains strict and cross-language consumable. `$float` markers represent NaN, positive/negative infinity, and negative zero for constructors and boundary mutations without nonstandard JSON; both harnesses parse the same table and `jq -e` accepts every record (`rewrite-in-rust/fixtures/hfa_word_model_core.jsonl:2`, `rewrite-in-rust/fixtures/hfa_word_model_core.jsonl:4`, `rewrite-in-rust/fixtures/hfa_word_model_core.jsonl:8`, `rewrite-in-rust/bootstrap/check_hfa_word_model_core.py:21`, `rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:294`).
- No dependency, bridge, or runtime creep is present. The dependency seam remains an independent library with `default_owner: legacy` and no bridge dependencies (`rewrite-in-rust/dependencies/hfa_word_model_core.yaml:24`). `hfa_word.rs` uses only `std` outside its `#[cfg(test)]` JSON fixture harness, Cargo manifests and lockfile have no current diff, and production callers still import Python `Phoneme`/`Word`/`WordList`. NumPy decoding, librosa/audio IO, aggregation, export, ONNX/model execution, warning presentation, and production routing remain explicitly legacy-owned (`rewrite-in-rust/dependencies/hfa_word_model_core.yaml:42`, `rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:1`, `inference/HubertFA/tools/decoder.py:5`, `inference/HubertFA/tools/infer_base.py:10`).

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_hfa_word_model_core.py`: passed all 14 current fixture cases.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_word -- --nocapture`: passed; both selected tests passed.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_word -- --list`: passed; listed both relevant tests.
- `jq -e . rewrite-in-rust/fixtures/hfa_word_model_core.jsonl`: passed for all strict JSONL records.
- `cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check`: passed.
- `cargo clippy --manifest-path rewrite-in-rust/rust/Cargo.toml --all-targets --all-features -- -D warnings`: passed.
- `git diff --check`: passed.
- `git diff -- rewrite-in-rust/rust/Cargo.toml rewrite-in-rust/rust/Cargo.lock rewrite-in-rust/rust/crates/v2m-core/Cargo.toml`: empty; no dependency or lockfile change.

## Residual Risk

This dependency/bootstrap rerun does not substitute for independent behavior or data/algorithm review. The later WordList units remain planned and must prove canonical reuse when implemented. Production promotion still requires an explicit Python/Rust payload, warning/error mapping, owner switch, and rollback evidence.

## Promotion Note

This role no longer blocks coordinator state update for `hfa_word_model_core`. The coordinator may record `dependency_bootstrap_reviewer` as passed, but this report alone does not justify marking the unit verified or promoted.
