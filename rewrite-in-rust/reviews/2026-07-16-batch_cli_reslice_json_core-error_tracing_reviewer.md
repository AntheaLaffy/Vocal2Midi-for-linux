# batch_cli_reslice_json_core - error_tracing_reviewer

Date: 2026-07-16
Decision: fail

## Findings

- Severity: medium
- Location: rewrite-in-rust/rust/crates/v2m-core/src/batch_cli_reslice_json.rs:230
- Issue: Missing `offset` or `duration` record fields are mapped to `ValueError` in Rust instead of the legacy `KeyError`.
- Evidence: `scripts/slice_asr_cli.py:374`-`376` indexes `record["offset"]` and `record["duration"]` before float coercion, so missing keys propagate as `KeyError`. A focused legacy probe returned `missing_offset ('KeyError', "'offset'")` and `missing_duration ('KeyError', "'duration'")`. Rust calls `required_f64` for both fields, and `required_f64` maps missing values through `value_error` at `rewrite-in-rust/rust/crates/v2m-core/src/batch_cli_reslice_json.rs:348`-`353`. The fixture only covers missing `index` and bad `offset` at `rewrite-in-rust/fixtures/batch_cli_reslice_json_core.jsonl:6`.
- Required fix: Add missing-`offset` and missing-`duration` fixtures, then split Rust field access from numeric coercion so missing fields return `KeyError` and malformed values return the legacy coercion error.

- Severity: medium
- Location: rewrite-in-rust/rust/crates/v2m-core/src/batch_cli_reslice_json.rs:229
- Issue: `index` coercion is not error-compatible with Python, which obscures both accepted input and propagated error type/message.
- Evidence: Legacy code uses `int(record["index"])` at `scripts/slice_asr_cli.py:374`. The focused probe returned `bad_index ('ValueError', "invalid literal for int() with base 10: 'bad'")` and `numeric_string_index ('OK', '')`. Rust uses `required_i64`, which only accepts JSON integers and otherwise returns `KeyError` from `rewrite-in-rust/rust/crates/v2m-core/src/batch_cli_reslice_json.rs:341`-`346`. Current fixtures do not cover numeric-string or malformed-string index values.
- Required fix: Add index coercion fixtures and implement Python-compatible `int(...)` behavior for record indices, including the legacy `ValueError` message for bad strings.

- Severity: medium
- Location: rewrite-in-rust/rust/crates/v2m-core/src/batch_cli_reslice_json.rs:182
- Issue: Malformed JSON diagnostics are underasserted and the Rust model drops line/column/character context from the propagated `JSONDecodeError`.
- Evidence: Legacy `json.loads` at `scripts/slice_asr_cli.py:359` reports `Expecting property name enclosed in double quotes: line 1 column 2 (char 1)` for the fixture's `{bad` payload. Rust hard-codes only `Expecting property name enclosed in double quotes` at `rewrite-in-rust/rust/crates/v2m-core/src/batch_cli_reslice_json.rs:182`-`186`, and the fixture asserts only `error_type` for the malformed JSON case at `rewrite-in-rust/fixtures/batch_cli_reslice_json_core.jsonl:6`.
- Required fix: Either preserve and assert the exact legacy `JSONDecodeError` string for malformed payload fixtures or explicitly record a deliberate normalized-error contract before promotion.

- Severity: low
- Location: rewrite-in-rust/rust/crates/v2m-core/src/batch_cli_reslice_json.rs:465
- Issue: Rust fixture assertion failures lack case-id and JSON-path context, which makes future error-parity regressions harder to diagnose.
- Evidence: The Python checker reports case id and assertion path at `rewrite-in-rust/bootstrap/check_batch_cli_reslice_json_core.py:24`-`44`; the Rust `assert_subset` helper at `rewrite-in-rust/rust/crates/v2m-core/src/batch_cli_reslice_json.rs:465`-`481` unwraps nested values and asserts equality without carrying `case_id` or a field path.
- Required fix: Thread the fixture `case_id` and nested path through the Rust assertion helper so failing parity cases identify the affected operation and field.

## Checks

- `PYTHONDONTWRITEBYTECODE=1 uv run python rewrite-in-rust/bootstrap/check_batch_cli_reslice_json_core.py`: passed.
- `CARGO_TARGET_DIR=/tmp/v2m-reslice-review-target cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml batch_cli_reslice_json`: passed.
- `PYTHONDONTWRITEBYTECODE=1 uv run python - <<'PY' ...`: focused legacy probe confirmed missing `offset`/`duration` are `KeyError`, bad string `index` is `ValueError`, numeric-string `index` is accepted, and malformed JSON includes line/column/char context.
- `PYTHONDONTWRITEBYTECODE=1 uv run python - <<'PY' ... yaml.safe_load(...)`: parsed `rewrite-in-rust/manifest.yaml` and `rewrite-in-rust/dependencies/batch_cli_reslice_json_core.yaml`.
- `git diff --check -- rewrite-in-rust/reviews/2026-07-16-batch_cli_reslice_json_core-error_tracing_reviewer.md`: passed.

## Residual Risk

The FileNotFoundError path context, empty-list and empty-audio skip stdout, lab sidecar decisions, and temp-path redaction are fixture-modeled and appear aligned with the declared boundary. Real `librosa`, `soundfile`/libsndfile, FFmpeg, and model-runtime IO errors remain legacy-owned per `rewrite-in-rust/bootstrap/batch_cli_reslice_json_core.md:45`-`47` and `rewrite-in-rust/dependencies/batch_cli_reslice_json_core.yaml:48`-`56`.

The remaining risk is concentrated in malformed record and payload errors that are not yet fixture-covered. Because the unit explicitly claims missing-key, numeric-coercion, and malformed-JSON propagation, the current checks are too narrow to prove the error-tracing surface.

## Promotion Note

This role blocks promotion. Do not mark `batch_cli_reslice_json_core` verified until the in-scope error cases above are covered by fixtures and the Rust model preserves the legacy error types/messages or a recorded normalized-error contract replaces exact propagation.
