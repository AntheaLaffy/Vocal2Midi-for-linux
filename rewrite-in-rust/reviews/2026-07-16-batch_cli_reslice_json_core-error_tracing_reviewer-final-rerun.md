# batch_cli_reslice_json_core - error_tracing_reviewer final rerun

Date: 2026-07-16
Decision: pass

## Findings

No error-tracing findings remain in the final rerun scope.

Previous findings:

- Closed: missing `offset` and `duration` now preserve legacy `KeyError`. Legacy indexes `record["offset"]` and `record["duration"]` before numeric coercion at `scripts/slice_asr_cli.py:374`-`376`; fixtures assert `"'offset'"` and `"'duration'"` at `rewrite-in-rust/fixtures/batch_cli_reslice_json_core.jsonl:7`; Rust separates missing-key lookup from numeric coercion through `required_f64` at `rewrite-in-rust/rust/crates/v2m-core/src/batch_cli_reslice_json.rs:355`-`358`.
- Closed: `slice_audio_from_json` record `index` now models Python `int(...)` coercion. Legacy calls `int(record["index"])` at `scripts/slice_asr_cli.py:374`; fixtures cover numeric string, float truncation, and bad string cases at `rewrite-in-rust/fixtures/batch_cli_reslice_json_core.jsonl:5` and `rewrite-in-rust/fixtures/batch_cli_reslice_json_core.jsonl:7`; Rust uses `required_python_int` / `python_int` at `rewrite-in-rust/rust/crates/v2m-core/src/batch_cli_reslice_json.rs:231` and `rewrite-in-rust/rust/crates/v2m-core/src/batch_cli_reslice_json.rs:350`-`385`.
- Closed: malformed JSON now asserts the exact legacy `JSONDecodeError` message. Legacy propagates `json.loads(...)` at `scripts/slice_asr_cli.py:359`; the fixture asserts `Expecting property name enclosed in double quotes: line 1 column 2 (char 1)` at `rewrite-in-rust/fixtures/batch_cli_reslice_json_core.jsonl:7`; Rust returns that full message at `rewrite-in-rust/rust/crates/v2m-core/src/batch_cli_reslice_json.rs:184`-`188`.
- Closed: malformed `save_timestamps_json` string and float `chunk_indices` now preserve legacy `TypeError`. Legacy uses the chunk index directly for `chunks[chunk_index]` at `scripts/slice_asr_cli.py:319`-`320`; fixtures assert `list indices must be integers or slices, not str` and `not float` at `rewrite-in-rust/fixtures/batch_cli_reslice_json_core.jsonl:3`; Rust rejects non-integer list-index values through `required_list_index_value` at `rewrite-in-rust/rust/crates/v2m-core/src/batch_cli_reslice_json.rs:106`-`110` and `rewrite-in-rust/rust/crates/v2m-core/src/batch_cli_reslice_json.rs:396`-`400`.
- Closed: Rust fixture assertion diagnostics now include case and path context. The Rust `assert_subset` helper threads `case_id` and nested JSON paths at `rewrite-in-rust/rust/crates/v2m-core/src/batch_cli_reslice_json.rs:548`-`582`, and grouped subcases are labeled with `case_id[index]` at `rewrite-in-rust/rust/crates/v2m-core/src/batch_cli_reslice_json.rs:612`-`620`.

## Checks

- `UV_CACHE_DIR=/tmp/v2m-uv-cache PYTHONDONTWRITEBYTECODE=1 uv run python rewrite-in-rust/bootstrap/check_batch_cli_reslice_json_core.py`: passed.
- `CARGO_TARGET_DIR=/tmp/v2m-reslice-final-rerun-target cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml batch_cli_reslice_json`: passed.
- `CARGO_TARGET_DIR=/tmp/v2m-reslice-final-rerun-target cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check`: passed.
- `UV_CACHE_DIR=/tmp/v2m-uv-cache PYTHONDONTWRITEBYTECODE=1 uv run python - <<'PY' ...`: parsed manifest/dependency YAML and confirmed fixtures include missing `offset`/`duration` `KeyError`, bad-index `ValueError`, exact malformed-JSON message, numeric-string/float re-slice indices, and string/float timestamp chunk-index `TypeError`.
- `UV_CACHE_DIR=/tmp/v2m-uv-cache PYTHONDONTWRITEBYTECODE=1 uv run python - <<'PY' ...`: focused legacy probe confirmed string/float `chunk_indices` raise `TypeError`, missing `offset`/`duration` raise `KeyError`, numeric-string and float re-slice indices are accepted, bad string index raises `ValueError`, and malformed JSON includes line/column/char context.
- `rg -n "librosa|soundfile|sf\\.write|load_audio|run_slicer|process_one_file|batch_transcribe_asr|load_qwen_model|RmvpeTranscriber|clear_qwen_model_cache" rewrite-in-rust/bootstrap/check_batch_cli_reslice_json_core.py rewrite-in-rust/rust/crates/v2m-core/src/batch_cli_reslice_json.rs rewrite-in-rust/fixtures/batch_cli_reslice_json_core.jsonl`: found only checker monkeypatches for `load_audio` / `cli.sf.write`; no Rust runtime call to excluded audio/model IO.

## Residual Risk

The unit remains fixture-bound. Real `librosa`, SoundFile/libsndfile, FFmpeg, ASR, RMVPE, slicer, ONNX Runtime, Qwen, full CLI parser UX, and production routing errors remain legacy-owned per `rewrite-in-rust/bootstrap/batch_cli_reslice_json_core.md:45`-`47` and `rewrite-in-rust/dependencies/batch_cli_reslice_json_core.yaml:48`-`56`.

The malformed-JSON assertion proves the current `{bad` fixture exactly. Rust models fixture payloads instead of becoming a general JSON parser for every possible malformed string; future malformed-JSON fixtures should add their own expected messages.

## Promotion Note

This error-tracing role no longer blocks coordinator state update for `batch_cli_reslice_json_core`. The manifest currently remains `reimplemented` at `rewrite-in-rust/manifest.yaml:839`-`860`; this review did not mark it verified.
