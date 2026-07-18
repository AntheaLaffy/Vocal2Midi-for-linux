# hfa_htk_label_export_core - data_algorithm_reviewer rerun2

Date: 2026-07-18
Decision: pass

Unit: `hfa_htk_label_export_core`
Role: `data_algorithm_reviewer`
Rerun after: `rewrite-in-rust/records/0091-fix-hfa-htk-pathlib-suffix-names.md`

## Findings

No findings.

The numeric behavior remains compatible with the selected legacy contract after
record 0091. Python renders HTK times with `int(float(time) * 10000000)` for
words and phonemes at `inference/HubertFA/tools/export_tool.py:42` through
`inference/HubertFA/tools/export_tool.py:48`. Rust still multiplies as `f64`
first, preserves the Python NaN and infinity conversion errors, and renders
finite scaled values with an unbounded decimal string routine at
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:232` through
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:300`. The
`huge_finite_time_uses_python_big_int` fixture locks positive and negative
finite values beyond fixed-width integer range at
`rewrite-in-rust/fixtures/hfa_htk_label_export_core.jsonl:9`, and an independent
CPython probe in this review matched the Rust decimal algorithm model against
`int(float(value) * 10000000)` for 121 cases.

The cumulative buffer data structure still mirrors the legacy observable state.
Python initializes `w_out` and `ph_out` once before the prediction loop at
`inference/HubertFA/tools/export_tool.py:37`, appends labels during iteration,
and writes the full accumulated buffers for each prediction at
`inference/HubertFA/tools/export_tool.py:60` through
`inference/HubertFA/tools/export_tool.py:63`. Rust keeps one cumulative
`String` for words and one for phonemes at
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:93` through
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:94`, appends label
lines at `rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:107` and
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:120` through
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:121`, and clones
the cumulative contents into ordered planned files at
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:148` through
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:157`. Fixture rows
4, 5, 11, and 20 cover cumulative later files, duplicate basename overwrite
order, partial plans after a later conversion error, and repeated stateless
calls.

The record 0091 path-name change does not introduce a data-structure or
algorithm risk for this role. It replaces Rust extension semantics with a
Python `Path.with_suffix(".lab").name` lexical projection at
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:188` through
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:220`. That code is
linear in the path component count and final-name length, does not affect the
numeric renderer, and does not alter the cumulative `String` storage model.
Fixture rows 16 through 18 lock trailing-dot, `..`, all-dot, hidden, and
multi-suffix names after record 0091.

Algorithmic complexity is acceptable for the in-memory planned-side-effect
seam. Label rendering is linear in predictions, words, phonemes, and label text
rendered. The custom decimal routine is bounded by binary64 exponent and digit
limits for each timestamp, not by unbounded input size. Planned file content
storage is proportional to the cumulative bytes that Python would write; this
can duplicate prefixes across many predictions, but that is the observable
compatibility output of this seam rather than an internal streaming design.

## Checks

- `PYTHONDONTWRITEBYTECODE=1 uv run python rewrite-in-rust/bootstrap/check_hfa_htk_label_export_core.py`: passed; validated 20 Python 3.12 fixtures generated from real `Exporter.save_htk`.
- `CARGO_TARGET_DIR=/tmp/v2m-hfa-htk-data-rerun2-target cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_htk_export::tests::hfa_htk_label_export_core_fixture_parity -- --exact`: passed; 1 focused Rust fixture-parity test, with 115 `v2m-core` tests and 5 quant bridge tests filtered out.
- `PYTHONDONTWRITEBYTECODE=1 uv run python - <<'PY' ...`: fixture inventory probe passed; confirmed 20 rows and the post-0091 suffix cases at rows 16, 17, and 18.
- `PYTHONDONTWRITEBYTECODE=1 uv run python - <<'PY' ...`: independent numeric probe passed; 121/121 cases matched CPython `int(float(value) * 10000000)`, including negative truncation, subnormal zeroing, huge finite values, random finite bit patterns, NaN, infinities, and max-float overflow-to-infinity.

## Residual Risk

The finite-float decimal renderer is custom code. The fixtures and independent
probe cover the compatibility classes that previously failed, including huge
finite values, but this is not a formal proof over every finite `f64` bit
pattern.

The in-memory plan intentionally clones cumulative output strings into each
planned file. That matches the selected planner seam and legacy observable
writes, but a future effectful or streaming promotion should not use these
cloned strings as a scalability model without re-reviewing memory behavior.

## Promotion Note

This `data_algorithm_reviewer` rerun2 is ready for coordinator state update for
this role. It does not block promotion on data/algorithm grounds. The
coordinator still owns the final unit state update and must combine this with
the other required review-role evidence after record 0091.
