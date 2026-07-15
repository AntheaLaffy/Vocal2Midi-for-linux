# quantization_bayesian_core - behavior_reviewer

Date: 2026-07-15
Decision: pass

## Findings

No behavior-parity findings.

The reviewed unit is explicitly `quantization_bayesian_core` and this report covers only `behavior_reviewer`. The manifest keeps the unit at `reimplemented`, requires behavior, data-algorithm, and Rust-style reviews, and keeps rollback on legacy Python `_quantize_notes_bayesian` at `rewrite-in-rust/manifest.yaml:260`. The boundary record confirms this unit is the Bayes-specific helper set and `_quantize_notes_bayesian` only, with public dispatch, GUI/Web/application settings, runtime promotion, and bridge design out of scope at `rewrite-in-rust/records/0010-confirm-bayesian-boundary.md:18` and `rewrite-in-rust/records/0010-confirm-bayesian-boundary.md:30`.

Behavior parity evidence is fixture-backed. The bootstrap defines the compatibility surface for no-op behavior, sorting, raw pair construction, gap annotation, Bayes candidate filtering, phase-center estimation, priors, segmented DP decode, final overlap repair, and metadata preservation at `rewrite-in-rust/bootstrap/quantization_bayesian_core.md:21`. The Python checker executes the same legacy helper and core functions against the helper and core TSV fixtures at `rewrite-in-rust/bootstrap/check_quantization_bayesian_core.py:17`, `rewrite-in-rust/bootstrap/check_quantization_bayesian_core.py:239`, and `rewrite-in-rust/bootstrap/check_quantization_bayesian_core.py:324`. The Rust tests include the same fixture tables and compare helper/core behavior at `rewrite-in-rust/rust/crates/v2m-core/src/quant.rs:1371`, `rewrite-in-rust/rust/crates/v2m-core/src/quant.rs:1968`, and `rewrite-in-rust/rust/crates/v2m-core/src/quant.rs:2124`.

The golden fixtures cover the required public behavior edges: empty no-op, disabled-step no-op with order preserved, sorting with metadata preservation, short-note fine-grid candidates, repeated motif priors, phase pullback, large-gap segmentation, overlap repair with minimum extension, and non-default tempo/step at `rewrite-in-rust/fixtures/quantization_bayesian_core.tsv:2`. Helper fixtures cover shift limits, fallback candidate selection, phase center, segment splitting, penalties, priors, local cost, and decode predecessor behavior at `rewrite-in-rust/fixtures/quantization_bayesian_helpers.tsv:2`.

Runtime ownership remains legacy Python. The Rust crate documents that it is not wired into Python production runtime at `rewrite-in-rust/rust/crates/v2m-core/src/lib.rs:1`, the Bayes Rust function is marked pre-promotion at `rewrite-in-rust/rust/crates/v2m-core/src/quant.rs:473`, and repository search found production dispatch still calling only `inference.quant.quantization._quantize_notes_bayesian` from `inference/quant/quantization.py:798`.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_quantization_bayesian_core.py`: passed
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml bayesian`: passed, 2 tests
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml`: passed, 37 tests
- `rg -n "from rewrite|import rewrite|v2m_core|v2m-core|quantize_notes_bayesian|_quantize_notes_bayesian" application inference gui web_server.py web_task_manager.py scripts -S`: no Rust bridge or production Rust import found; only legacy Python definition and dispatch appeared

## Residual Risk

This is a fixture-bound behavior review. It does not prove Python-compatible handling for arbitrary invalid numeric inputs such as NaN, infinities, overflow-sized ticks, or non-positive tempo. The dependency record intentionally keeps invalid numeric input error mapping out of this unit and assigns that to promotion work at `rewrite-in-rust/dependencies/quantization_bayesian_core.yaml:56`.

This review does not cover data/algorithm quality, Rust style, performance, public `quantize_notes` dispatch, GUI/Web/application defaults, or runtime promotion. Those remain separate roles or units.

## Promotion Note

This behavior role does not block promotion evidence. The coordinator can promote only after all required roles for `quantization_bayesian_core` pass and without marking the manifest from this report alone.
