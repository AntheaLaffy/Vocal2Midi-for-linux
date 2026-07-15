# quantization_phrase_dp_core - behavior_reviewer

Date: 2026-07-15
Decision: pass

## Findings

No behavior-parity findings.

The reviewed unit is explicitly `quantization_phrase_dp_core` and this report covers only `behavior_reviewer`. The manifest keeps the unit at `reimplemented`, lists behavior, data-algorithm, and Rust-style reviews as required, and keeps rollback on legacy Python `_quantize_notes_dp_asym` and `_quantize_notes_phrase_hybrid` at `rewrite-in-rust/manifest.yaml:233`. The boundary record confirms the unit is the phrase-DP/asymmetric helper set only and excludes Bayesian quantization, public dispatch, GUI/Web settings, and runtime promotion at `rewrite-in-rust/records/0009-confirm-phrase-dp-boundary.md:23`.

The Python compatibility source sorts notes by onset, resolves non-positive `quantization_step` to the internal grid, builds raw pairs with `raw_end = max(onset + 1, offset)`, splits phrase segments, decodes center options, repairs overlaps, and mutates only onset/offset at `inference/quant/quantization.py:354`. The `_quantize_notes_dp_asym` wrapper delegates to the same phrase implementation, and public `dp` dispatch still calls that wrapper at `inference/quant/quantization.py:789`.

The Rust implementation mirrors the observable phrase-DP behavior: empty no-op, stable onset sort, internal-grid fallback, same raw-pair construction, segment split, per-center decode, segment-level option DP, overlap repair, and onset/offset mutation at `rewrite-in-rust/rust/crates/v2m-core/src/quant.rs:280`. The local asymmetric cost, segment split, center adjustment, and decode helpers preserve the same scalar cost terms and first-min scan behavior at `rewrite-in-rust/rust/crates/v2m-core/src/quant.rs:538`.

Fixture coverage matches the documented behavior surface. Helper fixtures cover local cost terms, segment splitting, center adjustments, and decode cases at `rewrite-in-rust/fixtures/quantization_phrase_dp_helpers.tsv:2`. Golden note fixtures cover empty no-op, internal-grid fallback, sorting/metadata preservation, scaled centers, segmentation, tie lyrics, overlap repair, center bonuses, and multi-segment switching at `rewrite-in-rust/fixtures/quantization_phrase_dp_core.tsv:2`. The Python checker calls the legacy phrase helper directly at `rewrite-in-rust/bootstrap/check_quantization_phrase_dp_core.py:181`, and Rust tests consume the same helper and core fixture tables at `rewrite-in-rust/rust/crates/v2m-core/src/quant.rs:1169` and `rewrite-in-rust/rust/crates/v2m-core/src/quant.rs:1246`.

Rollback remains intact. The Rust crate is explicitly not wired into the Python runtime at `rewrite-in-rust/rust/crates/v2m-core/src/lib.rs:1`, and a production import scan found only the legacy Python phrase definitions and dispatch references.

## Checks

- `UV_CACHE_DIR=/tmp/v2m-uv-cache uv run python rewrite-in-rust/bootstrap/check_quantization_phrase_dp_core.py`: passed.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml phrase_dp`: passed; 2 tests passed.
- `rg -n "v2m_core|rewrite-in-rust/rust|quantize_notes_phrase_dp|quantize_notes_dp_asym|_quantize_notes_dp_asym|_quantize_notes_phrase_hybrid|from .*rust|import .*rust" application inference gui scripts web_server.py web_task_manager.py --glob '!**/__pycache__/**'`: inspected; only legacy Python phrase quantization definitions and dispatch matched.

## Residual Risk

This review does not cover data-structure or numeric-complexity quality beyond behavior parity; those belong to the required `data_algorithm_reviewer` role. It also does not cover Rust API shape, ownership, visibility, or warning hygiene; those belong to `rust_style_reviewer`.

The current Rust unit is fixture-bound to finite note timings and positive finite tempo when quantization runs. Invalid numeric input mapping, arbitrary Python dynamic object behavior, and bridge-level lyric coercion remain promotion-time work, as documented at `rewrite-in-rust/bootstrap/quantization_phrase_dp_core.md:140` and `rewrite-in-rust/dependencies/quantization_phrase_dp_core.yaml:54`.

## Promotion Note

This behavior review does not block coordinator state update for `quantization_phrase_dp_core`. The coordinator can mark the unit verified after all required roles pass. Runtime promotion is still out of scope; production ownership must remain legacy Python until a separate promotion unit verifies a bridge and caller behavior.
