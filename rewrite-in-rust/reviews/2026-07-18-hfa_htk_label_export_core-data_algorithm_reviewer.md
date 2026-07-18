# hfa_htk_label_export_core - data_algorithm_reviewer

Date: 2026-07-18
Decision: fail

## Findings

- Severity: high
- Location: rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:174
- Issue: finite HTK times whose scaled value exceeds `i128` are not rendered like Python. The legacy algorithm is `int(float(time) * 10000000)`, and Python `int` keeps arbitrary-precision finite results. Rust currently multiplies as `f64` and renders `(scaled as i128)`, which saturates for finite values outside the `i128` range instead of returning Python's full integer text.
- Evidence: `inference/HubertFA/tools/export_tool.py:42` and `inference/HubertFA/tools/export_tool.py:46` define the Python conversion. `uv run python -c "import math; vals=[1e32,-1e32]; print([(v, math.isfinite(v*10000000), int(v*10000000)) for v in vals]); print('i128_max', 2**127-1); print('i128_min', -(2**127))"` printed finite Python results `1000000000000000090824893823431825424384` and `-1000000000000000090824893823431825424384`, both beyond `i128`. The existing "large" fixture only reaches `1234567891234560` at `rewrite-in-rust/fixtures/hfa_htk_label_export_core.jsonl:8`, so it cannot catch this saturation path.
- Required fix: render finite `float * 10000000` results with Python-compatible arbitrary-precision truncation, or explicitly narrow the accepted public timestamp domain with a manifest/bootstrap update and legacy-backed rejection behavior. Add positive and negative finite fixtures above the `i128` boundary before rerunning this review role.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_hfa_htk_label_export_core.py`: passed, validated 12 fixtures.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_htk_label_export_core`: passed, one focused Rust fixture parity test.
- `uv run python -c "import math; vals=[1e32,-1e32]; print([(v, math.isfinite(v*10000000), int(v*10000000)) for v in vals]); print('i128_max', 2**127-1); print('i128_min', -(2**127))"`: showed Python finite arbitrary-precision outputs beyond `i128`.
- Source inspection: `inference/HubertFA/tools/export_tool.py:35`, `rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:87`, `rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:125`, and fixture lines 4-12.

## Residual Risk

No blocking data-structure issue was found for cumulative string buffers, ordered directory/file side-effect vectors, duplicate basename overwrite modeling, output-folder versus wav-parent path construction, empty output-folder handling, repeated exporter calls, or partial-plan cloning on conversion errors. Those choices are covered by the 12 existing fixtures, but the numeric fixture matrix does not yet cover finite scaled values outside fixed-width integer bounds.

The in-memory plan necessarily stores every planned file's full content, so memory is proportional to total planned write bytes. That is reasonable for this compatibility seam because Python also writes the cumulative buffers to each file, and the Rust API is explicitly a side-effect plan rather than a streaming writer.

## Promotion Note

This role blocks coordinator state update for `hfa_htk_label_export_core`. The unit should remain `reimplemented` until the finite large-number conversion behavior is fixed or formally scoped out with legacy-backed evidence, fixtures, and a rerun of this data/algorithm review.
