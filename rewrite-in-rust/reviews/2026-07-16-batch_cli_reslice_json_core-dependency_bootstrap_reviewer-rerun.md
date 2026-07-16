# batch_cli_reslice_json_core - dependency_bootstrap_reviewer rerun

Date: 2026-07-16
Decision: pass

## Findings

No findings.

Previous medium finding status: closed. `rewrite-in-rust/fixtures/batch_cli_reslice_json_core.jsonl:5` now contains `slice_audio_from_json_list_payload_success`, a successful non-empty top-level list payload with four records, four expected WAV write plans, and three expected lab sidecars. The focused fixture probe returned `[('slice_audio_from_json_list_payload_success', 'list_payload.json', 4, 4, 4, 3)]`.

## Boundary Decision

The manifest unit boundary remains confirmed. It should not be split, merged, deferred, or replaced for this role.

The kept-legacy boundary is still coherent:

- `rewrite-in-rust/manifest.yaml:856` requires monkeypatched `load_audio` and `sf.write`, synthetic arrays only, and no ASR/RMVPE inference, real audio decode, or real waveform encoding.
- `rewrite-in-rust/bootstrap/batch_cli_reslice_json_core.md:45` keeps real audio decoding, resampling, FFmpeg lookup, SoundFile/libsndfile encoding, actual WAV bytes, model execution, full CLI parser UX, and production routing out of this unit.
- `rewrite-in-rust/records/0040-confirm-batch-cli-reslice-json-boundary.md:42` keeps real audio decoding/resampling, WAV/PCM encoding, ASR/RMVPE/slicer/ONNX/Qwen runtime behavior, and full CLI progress/help parity legacy-owned.
- `rewrite-in-rust/rust/crates/v2m-core/src/batch_cli_reslice_json.rs:1` describes the Rust module as deterministic JSON timestamp and synthetic re-slicing helpers while Python remains runtime owner for decoding, resampling, SoundFile/libsndfile encoding, model execution, and CLI routing.

The hand-written replacement choice is still appropriate. The Rust module models JSON payload parsing, Python-like numeric coercion, path/name formatting, sample-range planning, and fake write/lab plans from fixtures; it does not add an audio-codec crate or production bridge. `rewrite-in-rust/rust/crates/v2m-core/Cargo.toml:12` lists only the existing core dependencies (`encoding_rs`, `md-5`, and `serde_json`), not a `librosa`, SoundFile, libsndfile, FFmpeg, or NumPy replacement.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_batch_cli_reslice_json_core.py`: passed.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml batch_cli_reslice_json`: passed; 1 fixture test, 61 filtered.
- `uv run python -c "import yaml; yaml.safe_load(open('rewrite-in-rust/manifest.yaml', encoding='utf-8')); yaml.safe_load(open('rewrite-in-rust/dependencies/batch_cli_reslice_json_core.yaml', encoding='utf-8'))"`: passed.
- `rg -n "librosa\\.load|soundfile\\.write|sf\\.write\\(|load_audio\\(|run_slicer\\(|process_one_file\\(|batch_transcribe_asr|load_qwen_model|RmvpeTranscriber|clear_qwen_model_cache|ffmpeg" rewrite-in-rust/bootstrap/check_batch_cli_reslice_json_core.py rewrite-in-rust/rust/crates/v2m-core/src/batch_cli_reslice_json.rs rewrite-in-rust/fixtures/batch_cli_reslice_json_core.jsonl`: found only the fake `load_audio` definition in the checker, with no real runtime/audio calls in the fixture/Rust surface.
- Focused list-payload fixture probe: passed and confirmed one successful non-empty top-level list payload with 4 writes and 3 labs.
- Dependency evidence inspection: `pyproject.toml` and `requirements.txt` declare `librosa`, `numpy`, and `soundfile`; `third_party/sources/manifest.json` maps `librosa-0.11.0`, `numpy-1.26.4`, and `soundfile-0.14.0`; `third_party/native_sources/manifest.json` maps `libsndfile-1.2.2`.
- `uv run python scripts/audit_vendored_sources.py`: passed; source audit reported 135 Python packages, 41 native-extension packages, 269 foreign runtime native binaries, and 0 third_party binary artifacts.

## Residual Risk

The Python checker still imports `scripts.slice_asr_cli` in the uv environment before monkeypatching `load_audio` and `sf.write`, so the excluded dependency graph must remain importable for the checker. That is documented in the bootstrap dependency expansion and is acceptable for this fixture-bound seam because the checker does not execute real decode, encode, slicer, ASR, RMVPE, ONNX, Qwen, or FFmpeg paths.

## Promotion Note

This dependency/bootstrap review role is ready for coordinator use. This report does not mark the manifest verified; promotion still depends on the coordinator and the remaining required review evidence.
