# slice_method_and_bounds_contract - dependency_bootstrap_reviewer

Date: 2026-07-17
Decision: pass-with-followups

## Findings

- Severity: low
- Location: rewrite-in-rust/bootstrap/slice_method_and_bounds_contract.md:79
- Issue: Future writer work must keep the two custom-bound error namespaces separate instead of collapsing them into the already-verified application validator or one shared message set.
- Evidence: The bootstrap surface calls for separate CLI and API bound resolvers at rewrite-in-rust/bootstrap/slice_method_and_bounds_contract.md:79; CLI errors use `--min-seconds` / `--max-seconds` at scripts/slice_asr_cli.py:139, while API errors use `min_len_sec` / `max_len_sec` at inference/API/slicer_api.py:691. `application.config.validate_slice_bounds` has a separate 0-60 second user-facing policy at application/config.py:17 and is already represented by the verified `slice_bounds_validation` unit at rewrite-in-rust/manifest.yaml:73.
- Required fix: In the future Rust writer pass, expose or test separate CLI/API custom-bound contracts and do not route this unit through `validate_slice_bounds`.

- Severity: low
- Location: rewrite-in-rust/bootstrap/check_slice_method_and_bounds_contract.py:18
- Issue: The legacy fixture checker imports the full owning modules, so the bootstrap proof still depends on the project uv environment having the heavy runtime packages installed even though the selected seam does not require Rust replacements for them.
- Evidence: The checker imports `inference.API.slicer_api` and `scripts.slice_asr_cli` at rewrite-in-rust/bootstrap/check_slice_method_and_bounds_contract.py:18. Those modules import `librosa`, `numpy`, `Slicer`, `soundfile`, ASR, RMVPE, and device utilities at inference/API/slicer_api.py:1 and scripts/slice_asr_cli.py:29. The dependency record explicitly keeps those capabilities legacy-owned at rewrite-in-rust/dependencies/slice_method_and_bounds_contract.yaml:38.
- Required fix: Keep the future Rust tests library-only over strings and optional floats; do not add audio, ONNX/RMVPE/ASR, SoundFile/libsndfile, FFmpeg, argparse, or filesystem dependencies to satisfy this unit.

No blocking dependency/bootstrap findings were found. The manifest unit boundary is confirmed and does not need to be split, merged, deferred, or replaced for this role.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_slice_method_and_bounds_contract.py`: pass
- `uv run python -c "import json, pathlib, yaml; yaml.safe_load(pathlib.Path('rewrite-in-rust/dependencies/slice_method_and_bounds_contract.yaml').read_text()); yaml.safe_load(pathlib.Path('rewrite-in-rust/manifest.yaml').read_text()); [json.loads(line) for line in pathlib.Path('rewrite-in-rust/fixtures/slice_method_and_bounds_contract.jsonl').read_text(encoding='utf8').splitlines() if line.strip() and not line.startswith('#')]; print('dependency yaml, manifest yaml, fixture jsonl parse ok')"`: pass
- `uv run python scripts/audit_vendored_sources.py`: pass

## Residual Risk

This review covers only dependency/bootstrap validity. Behavior parity, product ergonomics, and any future Rust API shape still need their own requested review passes before promotion. The fixture table is broad enough for bootstrap discovery, but future implementation should keep explicit coverage for mojibake repair, keyword fallback, unsupported-method messages, NaN, and infinity.

## Promotion Note

This role does not block writer work. It does not mark the unit verified; coordinator state should only change after the required review set for this unit passes.
