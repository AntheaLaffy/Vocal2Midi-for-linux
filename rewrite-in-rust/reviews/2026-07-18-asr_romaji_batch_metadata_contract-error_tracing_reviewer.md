# asr_romaji_batch_metadata_contract - error_tracing_reviewer

Date: 2026-07-18
Decision: fail

## Findings

- Severity: medium
- Location: rust/crates/v2m-core/src/asr_romaji_batch_metadata.rs:199
- Issue: negative fixed sample dimensions are silently converted into a successful zero-width batch instead of preserving the legacy error path.
- Evidence: Python legacy code keeps a non-zero negative `target_num_samples` from `get_fixed_num_samples(session) or max(lengths)` and then lets `np.zeros((batch_size, target_num_samples), ...)` raise `ValueError: negative dimensions are not allowed` after audio loading (`../inference/romaji_asr/common.py:92`, `../inference/romaji_asr/common.py:94`, `../inference/romaji_asr/common.py:97`). The Rust implementation computes `fixed_num_samples.max(0) as usize` (`rust/crates/v2m-core/src/asr_romaji_batch_metadata.rs:199` to `rust/crates/v2m-core/src/asr_romaji_batch_metadata.rs:203`), which suppresses that error and loses the diagnostic distinction between an explicit invalid fixed dimension and an intentional zero-sample fallback. A direct Python probe with fake metadata `shape=[1, -1]` produced `ValueError negative dimensions are not allowed [{'path': 'a', 'sample_rate': 16000}]`; the current fixture set has no negative-dimension case, and the Rust fixture test still passes.
- Required fix: Add a Python golden fixture for a negative fixed sample dimension, including the post-load `load_audio_calls` ordering and exact `ValueError` message, then update Rust to return a structured `BatchMetadataError` with `error_type="ValueError"`, `message="negative dimensions are not allowed"`, and the already-recorded load calls. If the coordinator decides negative ONNX dimensions are outside this unit, record that exclusion explicitly in the dependency/bootstrap record and keep the public Rust API from silently treating negative fixed dimensions as valid zero-width arrays.

## Checks

- `sed -n '1,260p' /home/fuurin/.claude/skills/vocal2midi-rs-review-gate/SKILL.md`: reviewed the installed review-gate instructions.
- `sed -n '1,220p' README.md`: reviewed rewrite mission and ownership rules.
- `sed -n '1,260p' resources.md`: reviewed source-of-truth and current ASR boundary index.
- `sed -n '1,260p' notes.md`: reviewed project constraints, including legacy ownership and dependency alignment.
- `sed -n '1,220p' reviews/README.md`: reviewed report format and decision vocabulary.
- `rg -n "asr_romaji_batch_metadata_contract|asr_romaji_batch_metadata|romaji_batch" manifest.yaml records dependencies bootstrap fixtures rust/crates/v2m-core/src rust/crates/v2m-core/Cargo.toml`: located unit artifacts and touched files.
- `git diff -- manifest.yaml dependencies/asr_romaji_batch_metadata_contract.yaml bootstrap/asr_romaji_batch_metadata_contract.md records/0121-bootstrap-asr-romaji-batch-metadata-contract.md records/0122-implement-asr-romaji-batch-metadata-contract.md rust/crates/v2m-core/Cargo.toml rust/crates/v2m-core/src/lib.rs rust/crates/v2m-core/src/asr_romaji_batch_metadata.rs fixtures/asr_romaji_batch_metadata_contract.jsonl`: inspected unit diff context; many files are untracked, so untracked file contents were read directly.
- `nl -ba rust/crates/v2m-core/src/asr_romaji_batch_metadata.rs | sed -n '1,520p'`: inspected structured error type, error ordering, load-call context, and fixture encoder.
- `nl -ba fixtures/asr_romaji_batch_metadata_contract.jsonl | sed -n '1,260p'`: inspected Python golden success and error cases.
- `nl -ba dependencies/asr_romaji_batch_metadata_contract.yaml | sed -n '1,220p'`: inspected dependency and ownership boundary.
- `nl -ba bootstrap/asr_romaji_batch_metadata_contract.md | sed -n '1,200p'`: inspected fixture strategy and writer boundary.
- `nl -ba records/0121-bootstrap-asr-romaji-batch-metadata-contract.md records/0122-implement-asr-romaji-batch-metadata-contract.md | sed -n '1,260p'`: inspected bootstrap and implementation records.
- `nl -ba ../inference/romaji_asr/common.py | sed -n '82,108p'`: inspected legacy error ordering around load, target length, and NumPy allocation.
- `cargo test --manifest-path rust/Cargo.toml asr_romaji_batch_metadata_contract -- --nocapture`: passed; 1 Rust fixture test passed, 23 JSONL cases matched.
- `uv run python rewrite-in-rust/bootstrap/check_asr_romaji_batch_metadata_contract.py`: passed; `asr_romaji_batch_metadata_contract fixtures ok: 23 cases`.
- `uv run python -m py_compile inference/romaji_asr/common.py`: passed.
- `rg -n "BatchMetadataError|value_error|key_error|error_type|load_audio_calls|prepare_batch_from_waveforms" rust/crates/v2m-core/src/asr_romaji_batch_metadata.rs fixtures/asr_romaji_batch_metadata_contract.jsonl bootstrap/check_asr_romaji_batch_metadata_contract.py ../inference/romaji_asr/common.py`: confirmed current structured error coverage and fixture-projected error fields.
- `uv run python - <<'PY' ... PY`: direct fake-session probe for `shape=[1, -1]` produced `ValueError negative dimensions are not allowed [{'path': 'a', 'sample_rate': 16000}]`.

## Residual Risk

The reviewed unit is still outside the production Python runtime, so no production logs or bridge redaction behavior exist yet. The current error object is structured and does not emit logs, but future bridge work must decide whether `load_audio_calls.path` can contain user-sensitive filesystem paths before surfacing it outside test or diagnostic channels. Real `load_audio` IO failures remain legacy-owned and were not reviewed here.

## Promotion Note

This role blocks promotion until the negative fixed sample dimension error behavior is either fixture-backed and implemented in Rust or explicitly excluded by the unit record with a non-silent Rust API behavior.
