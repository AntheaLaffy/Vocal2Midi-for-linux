# quantization_simple_grid_core - behavior_reviewer

Date: 2026-07-15
Decision: pass

## Findings

No findings.

Evidence:

- Legacy boundary is exactly `_quantize_notes_simple`: it returns without sorting for `quantization_step <= 0` or empty inputs, sorts by onset, captures original onset/offset values, uses Python half-even `round`, bumps non-increasing onsets, glues touching offsets, enforces one-step minimum duration, clips offsets to the next onset, and mutates only `onset`/`offset` on the sorted notes. Source: `inference/quant/quantization.py:687`.
- Rust mirrors that boundary in the independent library function: early return at `rewrite-in-rust/rust/crates/v2m-core/src/quant.rs:30`, stable onset sort at `rewrite-in-rust/rust/crates/v2m-core/src/quant.rs:35`, original timing capture at `rewrite-in-rust/rust/crates/v2m-core/src/quant.rs:41`, onset rounding/bumping at `rewrite-in-rust/rust/crates/v2m-core/src/quant.rs:45`, offset glue/minimum/clipping at `rewrite-in-rust/rust/crates/v2m-core/src/quant.rs:59`, and final onset/offset mutation at `rewrite-in-rust/rust/crates/v2m-core/src/quant.rs:79`.
- Python/Rust output shape, ordering, and pitch/lyric preservation are checked against the same fixture table. Fixture cases cover empty no-op, step-zero order preservation, half-even grid rounding, sorting plus monotonic onset bumping, touching-note glue, minimum duration, and offset clipping at `rewrite-in-rust/fixtures/quantization_simple_grid_core.tsv:2`.
- The Python harness calls only `_quantize_notes_simple` and compares mutated note lists to the fixture table at `rewrite-in-rust/bootstrap/check_quantization_simple_grid_core.py:83`. The Rust fixture test reads the same table and compares onset, offset, pitch, and lyric at `rewrite-in-rust/rust/crates/v2m-core/src/quant.rs:256`.
- Rollback remains intact: manifest ownership is still legacy at `rewrite-in-rust/manifest.yaml:172`, and the bootstrap rollback route keeps production imports on `inference.quant.quantization._quantize_notes_simple` at `rewrite-in-rust/bootstrap/quantization_simple_grid_core.md:98`.

## Checks

- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml simple_grid`: passed, 2 tests.
- `uv run python rewrite-in-rust/bootstrap/check_quantization_simple_grid_core.py`: passed.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml quant`: passed, 6 tests.

## Residual Risk

Fixture coverage is intentionally narrow to the simple-grid core. A future runtime promotion still needs bridge-level review for Python object mapping and any metadata beyond the current `NoteInfo` shape, because this Rust unit has no Python bridge and models only onset, offset, pitch, and lyric (`rewrite-in-rust/rust/crates/v2m-core/src/quant.rs:8`; `inference/io/note_io.py:11`).

The table does not separately fixture negative `quantization_step`, equal-onset stable ordering, or non-positive tempo. The code path for `quantization_step <= 0` matches the Python guard, and the production pipeline validates positive tempo before quantization (`inference/pipeline/auto_lyric_hybrid.py:61`).

## Promotion Note

This behavior review does not block promotion evidence for `quantization_simple_grid_core`. The coordinator can record this role as passed, but should not mark runtime promotion; production ownership remains legacy, and the manifest lists a separate `data_algorithm_reviewer` role for this unit (`rewrite-in-rust/manifest.yaml:181`).
