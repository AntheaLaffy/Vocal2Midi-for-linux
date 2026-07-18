# asr_romaji_batch_metadata_contract - error_tracing_reviewer rerun

Date: 2026-07-18
Decision: pass

## Findings

No findings.

- Severity: none
- Location: `rust/crates/v2m-core/src/asr_romaji_batch_metadata.rs:97`
- Issue: Structured error projection is adequate for this unit boundary.
- Evidence: `BatchMetadataError` carries `error_type`, `message`, and `load_audio_calls`; `ValueError` and `KeyError` constructors preserve the fixture-projected legacy exception class and already-recorded synthetic load calls.
- Required fix: none

- Severity: none
- Location: `rust/crates/v2m-core/src/asr_romaji_batch_metadata.rs:205`
- Issue: The prior negative fixed sample dimension review finding is fixed.
- Evidence: Rust now rejects negative `get_fixed_num_samples` after synthetic waveform loading with `ValueError` message `negative dimensions are not allowed`; fixture case `prepare_negative_fixed_num_samples_error_after_load` at `fixtures/asr_romaji_batch_metadata_contract.jsonl:24` pins the same message and load-call ordering.
- Required fix: none

- Severity: none
- Location: `rust/crates/v2m-core/src/asr_romaji_batch_metadata.rs:194`
- Issue: Missing synthetic waveform keys are no longer silently treated as empty waveforms.
- Evidence: Rust records the attempted load call and returns a `KeyError` using Python-style key formatting; fixture case `prepare_missing_waveform_key_error_after_load` at `fixtures/asr_romaji_batch_metadata_contract.jsonl:25` pins `KeyError`, message `'missing'`, and load-call context.
- Required fix: none

- Severity: none
- Location: `rust/crates/v2m-core/src/asr_romaji_batch_metadata.rs:273`
- Issue: `attention_mask` float16 projection is covered.
- Evidence: `cast_attention_mask` maps `NumpyDType::Float16` through `half::f16`; fixture case `prepare_attention_mask_float16_type` at `fixtures/asr_romaji_batch_metadata_contract.jsonl:26` pins dtype `float16`, shape `[1, 3]`, values `[[1.0, 1.0, 0.0]]`, and load-call context.
- Required fix: none

## Checks

- `sed -n '1,260p' /home/fuurin/.claude/skills/vocal2midi-rs-review-gate/SKILL.md`: read the required review-gate skill before reviewing.
- `nl -ba rust/crates/v2m-core/src/asr_romaji_batch_metadata.rs | sed -n '1,470p'`: inspected structured error type, negative dimension path, missing waveform key path, dtype casts, fixture harness, and absence of logging side effects.
- `nl -ba fixtures/asr_romaji_batch_metadata_contract.jsonl | sed -n '1,40p'`: inspected all 26 golden cases, including the three requested follow-up cases.
- `nl -ba bootstrap/asr_romaji_batch_metadata_contract.md | sed -n '1,220p'`: confirmed ownership boundary keeps audio IO, ONNX Runtime, providers, and model execution legacy-owned.
- `nl -ba records/0121-bootstrap-asr-romaji-batch-metadata-contract.md | sed -n '1,220p'`: confirmed bootstrap record documents 26-case coverage and synthetic-waveform boundary.
- `nl -ba records/0122-implement-asr-romaji-batch-metadata-contract.md | sed -n '1,240p'`: confirmed implementation record documents Python-compatible error projection for missing waveform keys and negative fixed sample dimensions.
- `nl -ba ../inference/romaji_asr/common.py | sed -n '1,155p'`: inspected legacy `prepare_batch` ordering and message source.
- `cargo test --manifest-path rust/Cargo.toml asr_romaji_batch_metadata_contract -- --nocapture`: passed; 1 Rust fixture test passed, 26 JSONL cases matched.
- `uv run python rewrite-in-rust/bootstrap/check_asr_romaji_batch_metadata_contract.py`: passed; `asr_romaji_batch_metadata_contract fixtures ok: 26 cases`.
- `cargo fmt --manifest-path rust/Cargo.toml --all -- --check`: passed.
- `uv run python -m py_compile inference/romaji_asr/common.py`: passed.
- `uv run python scripts/audit_vendored_sources.py`: passed; source audit reported 135 Python packages, 41 native-extension packages, 269 foreign runtime native binaries, and 0 third-party binary artifacts.
- `uv run python - <<'PY' ... PY`: direct legacy probes confirmed `shape=[1, -1]` raises `ValueError negative dimensions are not allowed` after one load call, missing synthetic key raises `KeyError 'missing'` after one load call, and `attention_mask` `tensor(float16)` returns dtype `float16` with expected values.
- `uv run python - <<'PY' ... yaml.safe_load(...) ... PY`: `rewrite-in-rust/manifest.yaml` and `rewrite-in-rust/dependencies/asr_romaji_batch_metadata_contract.yaml` loaded successfully.
- `git diff --check -- manifest.yaml dependencies/asr_romaji_batch_metadata_contract.yaml bootstrap/asr_romaji_batch_metadata_contract.md bootstrap/check_asr_romaji_batch_metadata_contract.py records/0121-bootstrap-asr-romaji-batch-metadata-contract.md records/0122-implement-asr-romaji-batch-metadata-contract.md rust/crates/v2m-core/src/asr_romaji_batch_metadata.rs rust/crates/v2m-core/src/lib.rs rust/crates/v2m-core/Cargo.toml`: passed for tracked relevant paths.

## Residual Risk

This unit remains a fixture-bound Rust helper outside the production Python runtime. It does not prove real ONNX Runtime metadata objects, real audio decode failures, path redaction policy for user-facing logs, or every Python `KeyError.__str__` quoting edge case for unusual path strings. Those are acceptable residual risks for the current synthetic-waveform ownership boundary because runtime ownership remains with `inference.romaji_asr.common`.

## Promotion Note

This `error_tracing_reviewer` rerun does not block promotion. The prior fail finding is resolved, and the requested negative dimension, missing synthetic waveform key, and `attention_mask` float16 coverage are backed by Python golden fixtures and the Rust fixture test.
