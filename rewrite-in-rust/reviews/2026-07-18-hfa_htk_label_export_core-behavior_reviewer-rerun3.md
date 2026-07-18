# hfa_htk_label_export_core - behavior_reviewer rerun3

Date: 2026-07-18
Decision: pass

Unit: `hfa_htk_label_export_core`
Role: `behavior_reviewer`
Rerun after: `rewrite-in-rust/records/0091-fix-hfa-htk-pathlib-suffix-names.md`

## Findings

No findings.

## Behavior Evidence

The Python compatibility source remains `Exporter.save_htk`: it initializes
`w_out` and `ph_out` before the prediction loop, converts timestamps with
`int(float(...) * HTK_PAD_VAL)`, builds both phone and word paths through
`wav_path.with_suffix(".lab").name`, then creates phone/word directories and
writes phone before word content (`inference/HubertFA/tools/export_tool.py:35`,
`inference/HubertFA/tools/export_tool.py:37`,
`inference/HubertFA/tools/export_tool.py:41`,
`inference/HubertFA/tools/export_tool.py:50`,
`inference/HubertFA/tools/export_tool.py:57`). `InferenceBase.export` still
routes production callers to Python `Exporter(...).export(...)`
(`inference/HubertFA/tools/infer_base.py:240`).

The Rust planner preserves the same public behavior in-memory. It keeps
cumulative word/phoneme buffers for the full call, renders each prediction in
order, records phone/word directory and file plans in Python order, maps empty
output-folder strings back to wav-parent mode, and returns partial plans on
Python-visible conversion/path errors
(`rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:87`,
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:91`,
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:92`,
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:96`,
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:99`,
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:125`,
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:132`,
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:148`).

Record 0091's suffix fix is present in the implementation: label names now use
a Python-style final-name projection and suffix replacement instead of
`PathBuf::set_extension`; `..` is accepted as a final name, while empty names
still return `ValueError: PosixPath('.') has an empty name`
(`rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:188`,
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:201`,
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:213`). Current
directory path text is normalized by dropping `.` components
(`rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:163`,
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:182`).
Finite scaled `f64` values are rendered as Python integer text without fixed
`i128` saturation, while NaN and infinity return the Python exception type and
message (`rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:232`,
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:249`).

The 20-row Python-generated fixture matrix covers the previously failed
behavior independently: huge finite HTK integers
(`rewrite-in-rust/fixtures/hfa_htk_label_export_core.jsonl:9`), dot and
dot-slash current-directory path text
(`rewrite-in-rust/fixtures/hfa_htk_label_export_core.jsonl:13`,
`rewrite-in-rust/fixtures/hfa_htk_label_export_core.jsonl:14`),
dot-prefixed wav-parent path text
(`rewrite-in-rust/fixtures/hfa_htk_label_export_core.jsonl:15`), trailing-dot
and parent-directory label names
(`rewrite-in-rust/fixtures/hfa_htk_label_export_core.jsonl:16`,
`rewrite-in-rust/fixtures/hfa_htk_label_export_core.jsonl:17`), the compact
pathlib suffix matrix
(`rewrite-in-rust/fixtures/hfa_htk_label_export_core.jsonl:18`), and empty-name
`wav_path` errors before side effects
(`rewrite-in-rust/fixtures/hfa_htk_label_export_core.jsonl:19`). The matrix also
continues to cover cumulative buffers, duplicate basename overwrite order, raw
Unicode/quotes/newlines, empty words/phones, partial plans after conversion
errors, and repeated exporter calls
(`rewrite-in-rust/fixtures/hfa_htk_label_export_core.jsonl:4`,
`rewrite-in-rust/fixtures/hfa_htk_label_export_core.jsonl:5`,
`rewrite-in-rust/fixtures/hfa_htk_label_export_core.jsonl:6`,
`rewrite-in-rust/fixtures/hfa_htk_label_export_core.jsonl:7`,
`rewrite-in-rust/fixtures/hfa_htk_label_export_core.jsonl:11`,
`rewrite-in-rust/fixtures/hfa_htk_label_export_core.jsonl:20`).

Rollback remains intact. The manifest keeps the unit `reimplemented`,
`current_owner: legacy`, and explicitly says to keep `Exporter.save_htk` as the
runtime owner including cumulative-buffer behavior
(`rewrite-in-rust/manifest.yaml:1566`, `rewrite-in-rust/manifest.yaml:1570`,
`rewrite-in-rust/manifest.yaml:1593`). The Rust crate is still an independently
testable surface and not wired into the Python runtime
(`rewrite-in-rust/rust/crates/v2m-core/src/lib.rs:1`,
`rewrite-in-rust/rust/crates/v2m-core/src/lib.rs:19`).

## Checks

- `PYTHONDONTWRITEBYTECODE=1 uv run python --version`: passed, Python 3.12.13.
- `PYTHONDONTWRITEBYTECODE=1 uv run python rewrite-in-rust/bootstrap/check_hfa_htk_label_export_core.py`: passed, validated 20 Python-generated fixtures against the real monkeypatched `Exporter.save_htk`.
- `CARGO_TARGET_DIR=/tmp/v2m-hfa-htk-rerun3-target cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml --locked hfa_htk_label_export_core`: passed, 1 focused Rust fixture-parity test; 115 `v2m-core` tests and 5 quant bridge tests filtered out.
- `CARGO_TARGET_DIR=/tmp/v2m-hfa-htk-rerun3-target cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml --locked hfa_htk_export::tests::hfa_htk_label_export_core_fixture_parity -- --exact`: passed, exact focused fixture-parity test.
- `PYTHONDONTWRITEBYTECODE=1 uv run python - <<'PY' ...`: targeted legacy exporter probe confirmed the previous behavior failures now have Python truth data matching the fixture expectations: no leading `./` for current-directory paths, `song.` to `song..lab`, `..` to `...lab`, huge finite integer text, and empty-name `""`, `"."`, and `"./"` errors before side effects.
- `rg -n "pub mod hfa_htk_export|hfa_htk_export|plan_htk_label_export|HfaHtk|save_htk|Exporter\\(|output_format|export\\(" rewrite-in-rust/rust inference application web_server.py web_task_manager.py scripts tests -g '!target'`: inspected routing references; Python remains the production runtime owner and Rust HTK code is only exposed inside `v2m-core`.

## Residual Risk

This review covers only behavior parity for the selected HTK planned-export
seam. It does not re-review dependency/bootstrap adequacy, data/algorithm
internals beyond behavior-visible timestamp rendering, error/tracing beyond the
public error projection, TextGrid serialization, export dispatch, bridge design,
or product ergonomics. Real filesystem IO errors, status printing, and
execution of the planned effects remain intentionally legacy-owned. Non-UTF-8
OS path display is not fixture-proven.

## Promotion Note

This `behavior_reviewer` rerun passes and is ready for coordinator state update
for this role. It does not mark the manifest `verified`; the coordinator still
needs to combine this result with the other required role results. Rollback is
still keeping Python `Exporter.save_htk` as runtime owner.
