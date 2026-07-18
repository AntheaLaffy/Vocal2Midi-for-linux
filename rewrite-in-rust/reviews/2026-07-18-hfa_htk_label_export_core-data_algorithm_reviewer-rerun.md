# hfa_htk_label_export_core - data_algorithm_reviewer rerun

Date: 2026-07-18
Decision: pass

## Findings

No findings.

The record 0089 repair addresses the prior data/algorithm failure. Legacy HTK
time rendering is `int(float(time) * 10000000)` at
`inference/HubertFA/tools/export_tool.py:42` through line 48, and the manifest
requires preserving that rendering plus the observable cumulative `w_out` and
`ph_out` buffers at `rewrite-in-rust/manifest.yaml:1575`. The fixed Rust path
still performs the binary64 multiply first at
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:200`, preserves
Python-compatible NaN and infinity errors at lines 202 through 213, and now
renders finite scaled values by decoding the `f64` bits into an unbounded
decimal string at lines 217 through 268 rather than casting into a fixed-width
integer.

The added fixture row
`rewrite-in-rust/fixtures/hfa_htk_label_export_core.jsonl:9` locks positive and
negative finite timestamps whose scaled values exceed `i128`, including the
expected CPython arbitrary-precision decimal text. A separate CPython probe in
this review compared a bit-level implementation of the Rust decimal algorithm
against `int(float(value) * 10000000)` for negative truncation, subnormal
zeroing, non-finite errors, the prior `1e32` failure class, the fixture `1e40`
class, and `1e300`; all 21 cases matched.

The cumulative buffer data structure is compatible with the legacy contract.
Python initializes `w_out` and `ph_out` once before the prediction loop at
`inference/HubertFA/tools/export_tool.py:37`, appends through the loop, then
writes the full accumulated buffers for each prediction at lines 60 through 63.
Rust mirrors this with one `String` each at
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:93` and line 94,
appends labels at lines 107 and 120 through 121, and stores cloned cumulative
file contents at lines 148 through 157. Fixture rows 4, 5, 11, and 14 cover
cumulative later files, duplicate basename overwrite order, partial plans after
conversion errors, and repeated stateless exporter calls.

## Checks

- `uv run python --version`: Python 3.12.13.
- `uv run python rewrite-in-rust/bootstrap/check_hfa_htk_label_export_core.py`:
  passed; validated 14 legacy-generated fixtures.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_htk_label_export_core -- --nocapture`:
  passed; `hfa_htk_export::tests::hfa_htk_label_export_core_fixture_parity` ok,
  with 115 `v2m-core` tests and 5 bridge tests filtered out.
- `uv run python - <<'PY' ... PY`: independent bit-level decimal conversion
  probe passed 21/21 comparisons against CPython `int(float(value) * 10000000)`,
  including `1e32`, `-1e32`, `1e40`, `-1e40`, `1e300`, `-1e300`, `5e-324`,
  `-5e-324`, NaN, infinities, and max-float overflow-to-infinity behavior.

## Residual Risk

The Rust decimal renderer is intentionally hand-written instead of backed by a
big-integer crate. Its worst case is bounded by binary64 exponent and digit
limits, but it is still a custom numeric routine. The fixture and independent
probe cover the public compatibility classes that failed the first review; they
do not constitute a formal proof for every finite `f64` bit pattern.

The in-memory plan stores every planned file's full cumulative content, so memory
is proportional to total planned write bytes and can duplicate cumulative
prefixes across many predictions. That matches the selected planned-side-effect
seam and the Python observable writes, but a future streaming/effectful bridge
should avoid treating these cloned strings as a scalable internal storage model.

## Promotion Note

This data/algorithm rerun does not block promotion. The unit remains
legacy-owned at runtime, and the coordinator may use this report as the
`data_algorithm_reviewer` pass evidence for updating this role's state.
