# hfa_htk_label_export_core - behavior_reviewer

Date: 2026-07-18
Decision: fail

## Findings

- Severity: high
- Location: rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:174
- Issue: Finite timestamp values whose scaled HTK time is outside `i128` range do not preserve Python's `int(float(time) * 10000000)` behavior.
- Evidence: Python `Exporter.save_htk` converts each start/end through `int(float(... ) * HTK_PAD_VAL)` at `inference/HubertFA/tools/export_tool.py:41` through `inference/HubertFA/tools/export_tool.py:48`. Python integers are arbitrary precision for finite floats, so `uv run python - <<'PY' ...` showed `1e35 * 10000000` renders as `1000000000000000044885712678075916785549312` and `-1e35 * 10000000` renders as `-1000000000000000044885712678075916785549312`. Rust computes the same scaled float but then emits `(scaled as i128).to_string()` at `rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:188`, which is bounded and cannot represent those finite Python outputs. The existing 12 fixtures include a "large" value, but it remains well inside `i128`.
- Required fix: Render finite scaled `f64` values with Python-compatible float-to-int decimal behavior for the full finite public range, or explicitly narrow the Rust public seam and fixture contract if such values are intentionally unsupported. Add positive and negative fixtures above the `i128` range before rerunning this review.

- Severity: medium
- Location: rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:168
- Issue: Empty-name `wav_path` values diverge from Python path planning and error behavior.
- Evidence: Python builds paths with `wav_path.with_suffix(".lab").name` at `inference/HubertFA/tools/export_tool.py:51` and `inference/HubertFA/tools/export_tool.py:54`. A targeted probe showed `Path("").with_suffix(".lab")` raises `ValueError: PosixPath('.') has an empty name` before any mkdir/write side effects. Rust instead uses `wav_path.file_name().map(PathBuf::from).unwrap_or_default()` and `set_extension("lab")` at `rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:168` through `rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:171`, so the planner has no equivalent path-planning failure path and can emit an HTK directory/file plan for an input Python rejects.
- Required fix: Mirror the Python `Path.with_suffix(".lab").name` empty-name failure, including partial side effects, or document and enforce a narrower valid-path precondition for the Rust seam. Add fixture coverage for this path edge if it remains public.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_hfa_htk_label_export_core.py`: passed, validated 12 fixtures.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_htk_label_export_core`: passed, 1 Rust fixture-parity test.
- `uv run python - <<'PY' ...`: targeted Python probe confirmed finite huge values render as arbitrary-precision integers, while values whose scaled float is infinity raise `OverflowError`.
- `uv run python - <<'PY' ...`: targeted Python path probe confirmed normal basename replacement cases and the empty-name `ValueError`.
- `rg -n "pub mod hfa_htk_export|plan_htk_label_export|hfa_htk_export|Exporter\\(|save_htk|output_format|export\\(" rewrite-in-rust/rust inference application web_server.py web_task_manager.py scripts tests -g '!target'`: inspected routing references. Rust exposes the module only inside `v2m-core`; Python `Exporter.save_htk` remains the production runtime path.
- Source inspection covered `rewrite-in-rust/manifest.yaml`, `rewrite-in-rust/dependencies/hfa_htk_label_export_core.yaml`, `rewrite-in-rust/bootstrap/hfa_htk_label_export_core.md`, `rewrite-in-rust/records/0074-split-hfa-g2p-config-export-lifecycle.md`, `rewrite-in-rust/records/0088-implement-hfa-htk-label-export-core.md`, `inference/HubertFA/tools/export_tool.py`, `rewrite-in-rust/bootstrap/check_hfa_htk_label_export_core.py`, `rewrite-in-rust/fixtures/hfa_htk_label_export_core.jsonl`, and `rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs`.

## Residual Risk

The existing fixtures prove the normal HFA prediction path for prediction order, output-folder None/empty/string behavior, basename replacement, cumulative `w_out`/`ph_out`, duplicate basenames, raw Unicode/quotes/newlines, partial conversion failures, and repeated calls. This review did not cover dependency adequacy, structured diagnostics, or TextGrid/export dispatch behavior.

## Promotion Note

This behavior review blocks coordinator state update for `hfa_htk_label_export_core`. The unit should remain `reimplemented` until the behavior findings are fixed, covered by fixtures, and rerun behavior evidence passes.
