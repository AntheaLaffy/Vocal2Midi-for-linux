# asr_romaji_batch_metadata_contract - behavior_reviewer

Date: 2026-07-18
Decision: pass-with-followups

## Findings

- Severity: medium
- Location: rewrite-in-rust/rust/crates/v2m-core/src/asr_romaji_batch_metadata.rs:188
- Issue: `prepare_batch_from_waveforms` records every requested path, but a path
  missing from the synthetic waveform map is silently treated as an empty
  waveform through `unwrap_or_default()`.
- Evidence: Python `prepare_batch` loads each path in order and propagates
  `load_audio` failures after the load call is attempted
  (`inference/romaji_asr/common.py:92`). The fixture harness' fake `load_audio`
  indexes `case["waveforms"][str(path)]`, so the same missing synthetic input
  would raise `KeyError` after recording the call
  (`rewrite-in-rust/bootstrap/check_asr_romaji_batch_metadata_contract.py:60`).
  The current golden file covers zero-length waveforms and load ordering, but
  every requested path has a waveform entry
  (`rewrite-in-rust/fixtures/asr_romaji_batch_metadata_contract.jsonl:15`).
- Required fix: before any runtime bridge or promotion that exposes
  `prepare_batch_from_waveforms` to non-fixture callers, add a golden case for a
  missing synthetic waveform key and make Rust propagate an error instead of
  substituting an empty waveform. This does not block the current fixture-bound
  reimplementation because runtime ownership remains legacy and the selected
  fixture inputs are complete.

## Checks

- `sed -n '1,260p' /home/fuurin/.claude/skills/vocal2midi-rs-review-gate/SKILL.md`: read the installed review skill fully.
- `sed -n '1,220p' README.md`: confirmed rewrite success condition is runnable, testable, and rollbackable.
- `sed -n '1,260p' resources.md`: confirmed `inference/romaji_asr/common.py` is the ASR helper source and model execution remains legacy-owned.
- `sed -n '1,260p' notes.md`: confirmed Stage 1 excludes model inference chains and allows fixture-bound backend helpers.
- `sed -n '1,220p' reviews/README.md`: confirmed report naming, structure, and role expectations.
- `nl -ba manifest.yaml | sed -n '1870,1915p'`: confirmed unit is `reimplemented`, current owner is `legacy`, required review includes `stage_behavior_reviewer`, and rollback keeps Python helpers.
- `nl -ba dependencies/asr_romaji_batch_metadata_contract.yaml | sed -n '1,140p'`: confirmed selected boundary is metadata/dtype/padding over fake metadata and synthetic waveforms; ONNX/audio/resampling remain out of scope.
- `nl -ba bootstrap/asr_romaji_batch_metadata_contract.md | sed -n '1,180p'`: confirmed fixture strategy covers fake session metadata, monkeypatched `load_audio`, ordering, errors, dtype casts, truncation, padding, masks, and used lengths.
- `nl -ba records/0121-bootstrap-asr-romaji-batch-metadata-contract.md records/0122-implement-asr-romaji-batch-metadata-contract.md | sed -n '1,220p'`: confirmed writer record claims Python-compatible error ordering and legacy rollback.
- `nl -ba ../inference/romaji_asr/common.py | sed -n '50,125p'`: confirmed legacy behavior for shape extraction, case-sensitive dtype mapping, empty-path error, fixed batch-size mismatch before loads, load order, target sample selection, feed ordering, optional mask, and missing `input_values` KeyError after loading.
- `nl -ba fixtures/asr_romaji_batch_metadata_contract.jsonl`: confirmed 23 golden cases cover fixed/dynamic/bool/one-dimensional shapes, dtype mapping, success paths, dtype casts, no-mask behavior, zero-length waveform, fixed-zero fallback, empty audio paths, batch mismatch, and missing `input_values`.
- `nl -ba bootstrap/check_asr_romaji_batch_metadata_contract.py | sed -n '1,180p'`: confirmed Python golden checker exercises current uv Python behavior with fake session metadata and monkeypatched synthetic waveform loading.
- `nl -ba rust/crates/v2m-core/src/asr_romaji_batch_metadata.rs | sed -n '1,460p'`: confirmed Rust implementation mirrors bool-as-int dims, dtype substring ordering, error order for covered cases, ndarray batch construction, feed projection, and JSON fixture assertions.
- `uv run python rewrite-in-rust/bootstrap/check_asr_romaji_batch_metadata_contract.py`: passed, `asr_romaji_batch_metadata_contract fixtures ok: 23 cases`.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml asr_romaji_batch_metadata_contract -- --nocapture`: passed, 1 fixture test passed with 125 filtered out in `v2m-core`; bridge crate had 0 matching tests.
- `uv run python -m py_compile inference/romaji_asr/common.py`: passed.
- `git diff --check -- rewrite-in-rust/rust/crates/v2m-core/src/asr_romaji_batch_metadata.rs rewrite-in-rust/fixtures/asr_romaji_batch_metadata_contract.jsonl rewrite-in-rust/bootstrap/check_asr_romaji_batch_metadata_contract.py rewrite-in-rust/dependencies/asr_romaji_batch_metadata_contract.yaml rewrite-in-rust/manifest.yaml rewrite-in-rust/records/0121-bootstrap-asr-romaji-batch-metadata-contract.md rewrite-in-rust/records/0122-implement-asr-romaji-batch-metadata-contract.md`: passed.
- `cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check`: passed.
- `uv run python - <<'PY' ... yaml.safe_load(...) ... PY`: parsed `rewrite-in-rust/manifest.yaml` and `rewrite-in-rust/dependencies/asr_romaji_batch_metadata_contract.yaml`.
- `uv run python scripts/audit_vendored_sources.py`: passed, `135 Python packages, 41 native-extension packages, 269 foreign runtime native binaries, 0 third_party binary artifacts`.

## Residual Risk

The review proves parity for the selected public seam and fixture-backed inputs.
It does not prove real ONNX Runtime metadata quirks, real `soundfile`/SciPy audio
loading or resampling, filesystem errors, negative fixed sample dimensions, NaN
or infinity sample casts, NumPy stride/layout internals, or model-session
execution. Those are explicitly outside this unit's boundary.

## Promotion Note

This behavior role does not block coordinator state update for the current
fixture-bound `reimplemented` unit. Keep the follow-up around missing synthetic
waveform keys attached to any later bridge or runtime promotion work.
