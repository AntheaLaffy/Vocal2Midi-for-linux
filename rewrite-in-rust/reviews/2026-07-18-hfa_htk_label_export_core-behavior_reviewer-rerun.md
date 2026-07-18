# hfa_htk_label_export_core - behavior_reviewer rerun

Date: 2026-07-18
Decision: fail

## Findings

- Severity: medium
- Location: rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:167
- Issue: Dot-prefixed current-directory paths do not preserve Python's planned path text. Python `pathlib` drops the leading `./` when `output_folder` is `"."` or `"./"`, and when `wav_path` is `"./song.wav"` in wav-parent mode. The Rust planner keeps the lexical `./` in the returned `PathBuf`s, so the public planned directory/file paths differ even though the filesystem target is equivalent.
- Evidence: Python stores truthy `output_folder` with `pathlib.Path(output_folder)` at `inference/HubertFA/tools/export_tool.py:9` and builds HTK paths with `/ "HTK" / ...` at `inference/HubertFA/tools/export_tool.py:50` through `inference/HubertFA/tools/export_tool.py:55`. The legacy monkeypatched exporter probe returned `HTK/Phones/song.lab` and `HTK/Words/song.lab` for `output_folder="."`, `output_folder="./"`, and `wav_path="./song.wav"`. The Rust root is copied directly from `output_folder` or `wav_path.parent()` at `rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:167` through `rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:170`, and the matching Rust std-path probe emitted `./HTK/Phones/song.lab` and `./HTK/Words/song.lab` for the same inputs. The current 14 fixtures do not include these path forms.
- Required fix: Match `pathlib.Path(...).as_posix()` path text for current-directory roots in the planned path contract, or explicitly narrow the Rust seam's accepted path forms. Add fixtures for `output_folder="."`, `output_folder="./"`, and wav-parent mode with `wav_path="./song.wav"` before rerunning this role.

## Rerun Result

Record `0089` fixed the two previously reported behavior failures:

- Huge finite HTK times now pass fixture parity. The new `huge_finite_time_uses_python_big_int` case at `rewrite-in-rust/fixtures/hfa_htk_label_export_core.jsonl:9` covers positive and negative values beyond `i128`, and Rust renders finite scaled `f64` values without an `i128` cast at `rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:200` through `rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:248`.
- Empty-name `wav_path` now passes fixture parity. The new `empty_wav_path_name_error_before_writes` case at `rewrite-in-rust/fixtures/hfa_htk_label_export_core.jsonl:13` expects `ValueError: PosixPath('.') has an empty name` with an empty partial plan, and Rust now returns that error from `lab_file_name` at `rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:177` through `rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:185`.

## Checks

- `uv run python --version`: passed, Python 3.12.13.
- `uv run python rewrite-in-rust/bootstrap/check_hfa_htk_label_export_core.py`: passed, validated 14 fixtures.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_htk_label_export_core`: passed, 1 focused Rust fixture-parity test.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_htk_export::tests::hfa_htk_label_export_core_fixture_parity -- --exact`: passed.
- `uv run python - <<'PY' ...`: targeted probe confirmed Python renders `1e40`, `-1e40`, `1e39`, and `1.234567890123456e30` as arbitrary-precision integer text after HTK scaling, and raises `ValueError: PosixPath('.') has an empty name` for `Path("").with_suffix(".lab")`.
- `uv run python - <<'PY' ...`: targeted legacy exporter probe confirmed `output_folder="."`, `output_folder="./"`, and `wav_path="./song.wav"` produce planned paths without leading `./`.
- `rustc -o /tmp/v2m_hfa_path_probe - <<'RS' ...`: targeted Rust std-path probe mirroring `planned_paths` confirmed the current Rust path construction produces leading `./` for those same inputs.
- `rg -n "pub mod hfa_htk_export|hfa_htk_export|plan_htk_label_export|HfaHtkPrediction|Exporter\\(|save_htk|output_format|export\\(" rewrite-in-rust/rust inference application web_server.py web_task_manager.py scripts tests -g '!target'`: confirmed the Rust module is only exposed in `v2m-core`; Python `Exporter.save_htk` remains the production runtime owner.

## Residual Risk

Fixture coverage now proves prediction order, output-folder None/empty/string behavior, cumulative `w_out`/`ph_out`, duplicate basename overwrite order, nested paths, raw Unicode/quotes/newlines, huge finite timestamp text, NaN/infinity errors with partial plans, empty-name `wav_path`, and repeated exporter calls for the covered cases. It still does not cover current-directory lexical normalization, trailing-dot file names, or `..` basename behavior from Python `Path.with_suffix(".lab").name`.

This behavior review did not re-review dependency adequacy, error/tracing structure outside the public error projection, data/algorithm internals beyond behavior-visible timestamp rendering, TextGrid serialization, or export dispatch.

## Promotion Note

This behavior rerun is not ready for coordinator state update for this role. Keep `hfa_htk_label_export_core` at `reimplemented`; rollback remains keeping Python `Exporter.save_htk` as the runtime owner, including its cumulative-buffer behavior.
