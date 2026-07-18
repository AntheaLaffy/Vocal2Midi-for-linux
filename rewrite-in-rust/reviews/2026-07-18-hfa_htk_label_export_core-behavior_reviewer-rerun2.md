# hfa_htk_label_export_core - behavior_reviewer rerun2

Date: 2026-07-18
Decision: fail

Unit: `hfa_htk_label_export_core`
Role: `behavior_reviewer`
Rerun after: `rewrite-in-rust/records/0090-fix-hfa-htk-current-directory-paths.md`

## Findings

- Severity: medium
- Location: `rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:188`
- Issue: HTK `.lab` basename planning still does not fully match Python `Path.with_suffix(".lab").name` for public `wav_path` text. Python accepts trailing-dot and parent-directory basenames, producing `song..lab` for `wav_path="song."` and `...lab` for `wav_path=".."`. Rust derives the label name through `Path::file_name()` plus `PathBuf::set_extension("lab")`, which produces `song.lab` for `song.` and treats `..` as an empty-name error.
- Evidence: Python source calls `wav_path.with_suffix(".lab").name` for both phone and word paths (`inference/HubertFA/tools/export_tool.py:51`, `inference/HubertFA/tools/export_tool.py:54`). A monkeypatched `Exporter.save_htk` probe wrote `out/HTK/Phones/song..lab` / `out/HTK/Words/song..lab` for `wav_path="song."`, and `out/HTK/Phones/...lab` / `out/HTK/Words/...lab` for `wav_path=".."`. A Rust std-path probe matching the implementation showed `PathBuf::from("song.").set_extension("lab")` becomes `song.lab`, while `Path::new("..").file_name()` is `None`. The implementation uses those exact operations at `rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:188` through `rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:199`. The 17-row fixture set has current-directory path cases but no trailing-dot or `..` basename case.
- Required fix: Replace label-name derivation with a Python-compatible `with_suffix(".lab").name` projection for these lexical basenames, and add fixture rows for `wav_path="song."` and `wav_path=".."` before rerunning this behavior role.

## Rerun Result For Previous Findings

Record `0089` fixes remain covered. Huge finite HTK times are locked by `huge_finite_time_uses_python_big_int` in `rewrite-in-rust/fixtures/hfa_htk_label_export_core.jsonl:9`, and Rust now renders finite scaled `f64` values through `python_int_string_from_f64` instead of a fixed-width cast (`rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:212` through `rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:260`).

The empty-name `wav_path` failure is covered by `empty_wav_path_name_error_before_writes` in `rewrite-in-rust/fixtures/hfa_htk_label_export_core.jsonl:16`; Rust now returns `ValueError: PosixPath('.') has an empty name` before appending directory or file plans (`rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:188` through `rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:196`).

Record `0090` fixes are also covered. The fixture rows `current_dir_output_folder_dot_normalizes_path_text`, `current_dir_output_folder_dot_slash_normalizes_path_text`, and `dot_prefixed_wav_parent_normalizes_path_text` at `rewrite-in-rust/fixtures/hfa_htk_label_export_core.jsonl:13` through `rewrite-in-rust/fixtures/hfa_htk_label_export_core.jsonl:15` all expect `HTK/...` paths without a leading `./`. Rust applies the same current-directory component projection in `planned_paths` (`rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:167` through `rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:185`).

Ordering and cumulative buffers pass the existing fixture matrix. The `multiple_predictions_are_cumulative` row shows later prediction files include previous prediction labels, matching Python's `w_out` and `ph_out` lifetime at `inference/HubertFA/tools/export_tool.py:37`.

Rollback remains intact: production Python still calls `Exporter.save_htk` / `Exporter.export` / `InferenceBase.export` (`inference/HubertFA/tools/export_tool.py:35`, `inference/HubertFA/tools/export_tool.py:65`, `inference/HubertFA/tools/infer_base.py:240`), while Rust exposes only an independent `v2m-core::hfa_htk_export` module (`rewrite-in-rust/rust/crates/v2m-core/src/lib.rs:19`).

## Checks

- `uv run python --version`: passed, Python 3.12.13.
- `uv run python rewrite-in-rust/bootstrap/check_hfa_htk_label_export_core.py`: passed, validated 17 fixtures.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_htk_label_export_core`: passed, 1 focused Rust fixture-parity test; 115 `v2m-core` tests and 5 quant bridge tests filtered out.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_htk_export::tests::hfa_htk_label_export_core_fixture_parity -- --exact`: passed, exact focused fixture-parity test.
- `uv run python - <<'PY' ...`: direct Python exporter probe confirmed huge finite integer text, empty-path `ValueError`, and all three current-directory path cases from record `0090`.
- `uv run python - <<'PY' ...`: direct Python path/exporter probe confirmed `song.` maps to `song..lab` and `..` maps to `...lab`.
- `rustc -o /tmp/v2m_path_suffix_probe - <<'RS' ...`: direct Rust std-path probe confirmed the current implementation primitives map `song.` to `song.lab` and treat `..` as no file name.
- `jq -r ... rewrite-in-rust/fixtures/hfa_htk_label_export_core.jsonl`: inspected the 17 fixture IDs and regression rows for huge finite times, empty-name path errors, and current-directory path normalization.
- `rg -n "hfa_htk_export|plan_htk_label_export|HfaHtk|save_htk|Exporter\\(|output_format|export\\(" rewrite-in-rust/rust inference application web_server.py web_task_manager.py scripts tests -g '!target'`: inspected routing references; Python remains the runtime owner and Rust HTK code is only exposed inside the independent Rust crate.

## Residual Risk

This review covered the selected HTK public behavior seam: prediction order, output paths, cumulative buffers, timestamp conversion, raw text, partial plans on errors, and rollback. It did not re-review dependency adequacy, error/tracing structure beyond behavior-visible errors, TextGrid serialization, export dispatch, or promotion bridge design.

Additional path lexical corners may remain unproven until the Rust label-name projection is changed from Rust `set_extension` semantics to Python `Path.with_suffix(".lab").name` semantics and fixtures are expanded around that projection.

## Promotion Note

This role is not ready for coordinator state update. Keep `hfa_htk_label_export_core` at `reimplemented`; rollback remains keeping Python `Exporter.save_htk` as the runtime owner, including cumulative-buffer behavior.
