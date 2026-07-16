# batch_cli_reslice_json_core - error_tracing_reviewer rerun

Date: 2026-07-16
Decision: pass-with-followups

## Findings

- Severity: medium
- Location: rewrite-in-rust/rust/crates/v2m-core/src/batch_cli_reslice_json.rs:106
- Issue: `save_timestamps_json` still has an uncovered malformed `chunk_indices` error surface: Rust coerces chunk indices with `python_int`, while legacy Python uses the value directly as a list index.
- Evidence: Legacy code assigns `chunk_index = chunk_indices[result_index]` and then indexes `chunks[chunk_index]` at `scripts/slice_asr_cli.py:319`-`320`. A focused legacy probe returned `TypeError: list indices must be integers or slices, not str` for `chunk_indices=["1"]` and `TypeError: list indices must be integers or slices, not float` for `chunk_indices=[1.2]`. Rust instead calls `python_int` at `rewrite-in-rust/rust/crates/v2m-core/src/batch_cli_reslice_json.rs:106`-`110`, which would accept string and float values. Current fixtures cover negative and out-of-range integer chunk indices at `rewrite-in-rust/fixtures/batch_cli_reslice_json_core.jsonl:3`, but not string or float chunk-index errors.
- Required fix: Add `save_timestamps_json` fixtures for string and float `chunk_indices`, then either preserve legacy `TypeError` behavior or record that this malformed internal input is deliberately out of scope.

- Severity: low
- Location: rewrite-in-rust/rust/crates/v2m-core/src/batch_cli_reslice_json.rs:523
- Issue: The previous low finding about Rust assertion diagnostics is still open.
- Evidence: The Python checker carries case id and nested JSON path in assertion errors at `rewrite-in-rust/bootstrap/check_batch_cli_reslice_json_core.py:24`-`44`. Rust `assert_subset` still unwraps nested values and asserts equality without case id or field path at `rewrite-in-rust/rust/crates/v2m-core/src/batch_cli_reslice_json.rs:523`-`539`.
- Required fix: Thread fixture `case_id` and nested path through the Rust assertion helper when maintainability work resumes. This is diagnostic quality, not behavior parity.

## Previous Fail Findings

- Closed: missing `offset` and `duration` now have explicit fixtures expecting `KeyError` at `rewrite-in-rust/fixtures/batch_cli_reslice_json_core.jsonl:7`, and Rust `required_f64` now separates missing-key handling from numeric coercion at `rewrite-in-rust/rust/crates/v2m-core/src/batch_cli_reslice_json.rs:355`-`358`.
- Closed: `slice_audio_from_json` record `index` now uses Python-compatible `int(...)` modeling through `required_python_int` / `python_int` at `rewrite-in-rust/rust/crates/v2m-core/src/batch_cli_reslice_json.rs:350`-`383`. Fixtures cover numeric string, float truncation, and bad string cases at `rewrite-in-rust/fixtures/batch_cli_reslice_json_core.jsonl:5` and `rewrite-in-rust/fixtures/batch_cli_reslice_json_core.jsonl:7`.
- Closed: malformed JSON now asserts the exact legacy message `Expecting property name enclosed in double quotes: line 1 column 2 (char 1)` in the fixture at `rewrite-in-rust/fixtures/batch_cli_reslice_json_core.jsonl:7`, and Rust returns that message at `rewrite-in-rust/rust/crates/v2m-core/src/batch_cli_reslice_json.rs:184`-`188`.
- Still open as low: Rust assertion diagnostics do not yet include case id or nested field path.

## Checks

- `PYTHONDONTWRITEBYTECODE=1 uv run python rewrite-in-rust/bootstrap/check_batch_cli_reslice_json_core.py`: passed.
- `CARGO_TARGET_DIR=/tmp/v2m-reslice-review-rerun-target cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml batch_cli_reslice_json`: passed.
- `PYTHONDONTWRITEBYTECODE=1 uv run python - <<'PY' ...`: inspected fixture cases for numeric-string/float/bad-string index, exact `JSONDecodeError`, missing `offset`, and missing `duration`.
- `PYTHONDONTWRITEBYTECODE=1 uv run python - <<'PY' ...`: focused legacy probe confirmed `save_timestamps_json` string/float `chunk_indices` raise `TypeError`, while bool and negative integer indices are accepted.
- `PYTHONDONTWRITEBYTECODE=1 uv run python - <<'PY' ... yaml.safe_load(...)`: parsed `rewrite-in-rust/manifest.yaml` and `rewrite-in-rust/dependencies/batch_cli_reslice_json_core.yaml`.
- `rg -n "librosa|soundfile|sf\\.write|load_audio|run_slicer|process_one_file|batch_transcribe_asr|load_qwen_model|RmvpeTranscriber|clear_qwen_model_cache" ...`: found only documented exclusions and checker monkeypatches; no Rust runtime call to excluded audio/model IO.

## Residual Risk

The unit remains fixture-bound. Real `librosa`, `soundfile`/libsndfile, FFmpeg, ASR, RMVPE, slicer, ONNX Runtime, and Qwen errors remain legacy-owned per `rewrite-in-rust/bootstrap/batch_cli_reslice_json_core.md:45`-`47` and `rewrite-in-rust/dependencies/batch_cli_reslice_json_core.yaml:48`-`56`.

The exact malformed JSON message is now proven for the current `{bad` fixture, but Rust still models that fixture case rather than running a general JSON parser. That is acceptable for the current fixture-bound gate as long as future malformed-JSON fixtures add their own expected messages.

## Promotion Note

The previous blocking error-tracing findings are closed. This role is suitable for coordinator advancement as `pass-with-followups` if malformed non-integer `save_timestamps_json` `chunk_indices` are accepted as a follow-up or internal-input edge case. Do not mark this report as a clean pass, and do not mark the manifest verified solely from this review if the coordinator treats that `chunk_indices` malformed-input surface as required before promotion.
