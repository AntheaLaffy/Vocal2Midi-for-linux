# batch_cli_reslice_json_core - dependency_bootstrap_reviewer

Date: 2026-07-16
Decision: pass-with-followups

## Findings

- Severity: medium
- Location: rewrite-in-rust/manifest.yaml:853, rewrite-in-rust/dependencies/batch_cli_reslice_json_core.yaml:33, rewrite-in-rust/bootstrap/batch_cli_reslice_json_core.md:26, rewrite-in-rust/fixtures/batch_cli_reslice_json_core.jsonl:4
- Issue: The dependency/bootstrap artifacts require dict and list JSON payload coverage, but the current durable fixtures do not include a successful non-empty top-level list payload that writes chunks/labs. The passing write-plan case uses a dict payload with `chunks`; current list payloads only exercise error paths.
- Evidence: `uv run python -c "...fixture audit..."` reported list payload cases only for `source_exists=false`, missing `index`, and bad `offset`: `[('slice_audio_from_json_error_cases', 'timestamps.json', 0, 'error', None, False), ('slice_audio_from_json_error_cases', 'missing_index.json', 1, 'error', None, True), ('slice_audio_from_json_error_cases', 'bad_offset.json', 1, 'error', None, True)]`.
- Required fix: Add a valid top-level list-payload fixture that reaches `slice_audio_from_json` write planning, including at least one WAV path and lab/no-lab decision, then rerun the Python checker and Rust fixture test.

- Severity: low
- Location: rewrite-in-rust/bootstrap/check_batch_cli_reslice_json_core.py:21, scripts/slice_asr_cli.py:29, scripts/slice_asr_cli.py:37, rewrite-in-rust/dependencies/batch_cli_reslice_json_core.yaml:48
- Issue: The checker keeps real audio/model behavior out of calls, but it still imports the full legacy CLI module before monkeypatching `load_audio` and `sf.write`. That means bootstrap verification still requires the excluded heavy import graph (`librosa`, `soundfile`, and inference modules) to be importable in the uv environment.
- Evidence: `scripts/slice_asr_cli.py` imports `librosa` and `soundfile` at module import time and also imports ASR/RMVPE/slicer/device modules. The checker then patches `cli.load_audio` and `cli.sf.write` only after that import. Current checks pass, so this is an environment/bootstrap fragility rather than evidence of real decode/encode calls.
- Required fix: Document the uv import precondition in the bootstrap record, or later isolate the checker import path if this unit must run in a minimal dependency environment.

## Boundary Decision

The manifest unit boundary is confirmed. It should not be split, merged, deferred, or replaced for this role. The records correctly keep real audio loading/resampling, FFmpeg behavior, SoundFile/libsndfile encoding, ASR/RMVPE/slicer/model runtime, full CLI parser UX, and production routing legacy-owned while moving only deterministic JSON/text/path/sample-range/write-plan behavior into fixture-backed Rust.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_batch_cli_reslice_json_core.py`: passed.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml batch_cli_reslice_json`: passed, 1 fixture test.
- `uv run python -c "import yaml; yaml.safe_load(...manifest...); yaml.safe_load(...dependency record...)"`: passed.
- `uv run python scripts/audit_vendored_sources.py`: passed; source audit reported 135 Python packages, 41 native-extension packages, 269 foreign runtime native binaries, and 0 third_party binary artifacts.
- Dependency evidence inspected with `rg`: `librosa`, `numpy`, and `soundfile` are declared in project dependencies and have vendored source entries; `libsndfile-1.2.2` is mapped in `third_party/native_sources/manifest.json`.
- Fixture audit for list-payload coverage: passed as a probe, and produced the medium finding above.

## Residual Risk

The hand-written Rust model intentionally does not prove real codec IO or NumPy array semantics beyond synthetic fixture shapes. That is consistent with the confirmed seam, but the successful list-payload fixture gap should be closed before coordinator promotion relies on this review as complete dependency/bootstrap evidence.

## Promotion Note

This role does not block continued writer or behavior-review work, but it should remain `pass-with-followups` until the missing successful list-payload fixture is added or explicitly accepted by the coordinator. Do not mark the manifest verified based on this report alone.
