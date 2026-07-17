# hfa_word_model_core - dependency_bootstrap_reviewer

Date: 2026-07-17
Decision: pass-with-followups

Unit: `hfa_word_model_core`
Role: `dependency_bootstrap_reviewer`

## Findings

- Severity: low
- Location: `rewrite-in-rust/dependencies/hfa_word_model_core.yaml:51`
- Issue: Durable verification commands use two different test filters. The dependency record and bootstrap prescribe `hfa_word_model`, while the manifest and implementation record prescribe `hfa_word`. The narrower `hfa_word_model` filter selects only the shared fixture-table test and omits `python_float_error_formatting_normalizes_exponents`.
- Evidence: `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_word_model -- --list` listed 1 test. `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_word -- --nocapture` ran and passed both `hfa_word_model_follows_parity_fixture_table` and `python_float_error_formatting_normalizes_exponents`. See also `rewrite-in-rust/bootstrap/hfa_word_model_core.md:71`, `rewrite-in-rust/manifest.yaml:1326`, `rewrite-in-rust/records/0067-implement-hfa-word-model-core.md:35`, and `rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:497`.
- Required fix: Before coordinator closure, standardize the dependency/bootstrap command on the broader `hfa_word` filter, or rename the exponent-format test so the documented `hfa_word_model` filter selects both tests.

No dependency, seam, inventory, or canonical-ownership blocker was found.

Evidence:

- Boundary decision: confirmed. Record 0066 correctly replaced the former broad `hfa_word_interval_core` with three ordered units. `hfa_word_model_core` owns only canonical `Phoneme`/`Word` construction and local mutations; `hfa_wordlist_collection_ap_core` introduces the sole canonical `WordList`/log storage over those types; `hfa_wordlist_finalize_core` must extend that same collection (`rewrite-in-rust/records/0066-split-hfa-word-lifecycle.md:46`, `rewrite-in-rust/bootstrap/hfa_word_model_core.md:36`, `rewrite-in-rust/bootstrap/hfa_wordlist_collection_ap_core.md:36`, `rewrite-in-rust/bootstrap/hfa_wordlist_finalize_core.md:39`). The manifest encodes the same dependency direction and explicitly bans parallel representations (`rewrite-in-rust/manifest.yaml:1302`, `rewrite-in-rust/manifest.yaml:1329`, `rewrite-in-rust/manifest.yaml:1353`).
- Capability coverage is complete for the selected first-unit boundary. The dependency record maps constructors, duration, contained add, contiguous append/end growth, and boundary mutation to the legacy source and canonical Rust API (`rewrite-in-rust/dependencies/hfa_word_model_core.yaml:3`). These are the local operations used by decoder construction, aggregate reconstruction, and API short-word repair (`inference/HubertFA/tools/decoder.py:109`, `inference/HubertFA/tools/infer_base.py:217`, `inference/API/hfa_api.py:67`). WordList policies remain in the two later units rather than leaking into the first Rust implementation.
- Fixture strategy covers the non-obvious numeric surface with strict JSON. Twelve JSONL cases use `$float` markers for `NaN`, positive/negative infinity, and negative zero across Phoneme/Word constructors and boundary moves; they also cover reversed intervals, exact errors, initial phonemes, duration, log-list and warning sinks, zero-length mutated inputs, and empty-phoneme `IndexError` projection (`rewrite-in-rust/fixtures/hfa_word_model_core.jsonl:1`, `rewrite-in-rust/bootstrap/check_hfa_word_model_core.py:21`, `rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:286`). `jq -e` accepted every fixture line as strict JSON, and both Python and Rust fixture gates passed.
- Kept-legacy choices match the mission. NumPy decoder math, librosa/audio IO, multi-pass aggregation, TextGrid/export, ONNX Runtime/model execution, Python warning presentation, and production routing remain caller-owned (`rewrite-in-rust/dependencies/hfa_word_model_core.yaml:42`, `rewrite-in-rust/records/0066-split-hfa-word-lifecycle.md:98`). This preserves the Stage 1 exclusion of HFA model execution (`rewrite-in-rust/resources.md:144`, `rewrite-in-rust/notes.md:51`).
- No dependency, bridge, or runtime creep was introduced. `align_word.py` itself uses only `warnings` and `dataclasses`; the Rust module uses only `std` outside its `#[cfg(test)]` fixture harness. Cargo manifests and lockfile have no current diff, bridge dependencies remain empty, Python remains runtime owner, and repository callers continue importing the Python types (`rewrite-in-rust/dependencies/hfa_word_model_core.yaml:24`, `rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:1`, `rewrite-in-rust/rust/crates/v2m-core/src/lib.rs:17`).

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_hfa_word_model_core.py`: passed all 12 current fixture cases.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_word -- --nocapture`: passed; 2 selected tests passed.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_word_model -- --list`: passed; demonstrated that the narrower documented filter selects only 1 test.
- `jq -e . rewrite-in-rust/fixtures/hfa_word_model_core.jsonl`: passed for all JSONL records.
- `cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check`: passed.
- `cargo clippy --manifest-path rewrite-in-rust/rust/Cargo.toml --all-targets --all-features -- -D warnings`: passed.
- `git diff --check`: passed.
- `git diff -- rewrite-in-rust/rust/Cargo.toml rewrite-in-rust/rust/Cargo.lock rewrite-in-rust/rust/crates/v2m-core/Cargo.toml`: empty; no dependency manifest or lockfile change.

## Residual Risk

This review confirms dependency/bootstrap readiness and the selected boundary, not full behavior or algorithm parity. The two later WordList units are intentionally still planned; their records define canonical reuse, but their fixtures and implementations must prove that reuse before writer completion. A future production promotion still needs an explicit Python/Rust payload, warning/error mapping, owner switch, and rollback evidence.

## Promotion Note

The low-severity command-filter inconsistency does not invalidate the boundary or block independent behavior/data reviews. The coordinator should resolve it before closing `hfa_word_model_core`; this report alone does not justify marking the unit verified or promoted.
