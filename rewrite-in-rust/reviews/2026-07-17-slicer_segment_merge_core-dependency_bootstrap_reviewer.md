# slicer_segment_merge_core - dependency_bootstrap_reviewer

Date: 2026-07-17
Decision: pass
Manifest unit boundary: confirmed

## Findings

No findings for this dependency/bootstrap review.

## Boundary Review

- Unit and role reviewed: `slicer_segment_merge_core`, `dependency_bootstrap_reviewer`.
- Writer/reviewer separation: confirmed. No Rust `slicer_segment` implementation exists under `rewrite-in-rust/rust/crates/v2m-core/src/`, and the coordinator has not routed writer work for this unit.
- Manifest state: `rewrite-in-rust/manifest.yaml:982` through `rewrite-in-rust/manifest.yaml:1001` keeps the unit `planned`, `inventory_status: confirmed`, `current_owner: legacy`, with rollback to the Python merge helpers. This review does not mark the unit verified.
- Boundary decision: confirmed. The discovery record confirms the unit as pure segment waveform manipulation over synthetic mono/stereo arrays and segment dictionaries, excluding RMS/default slicing, heuristic/grid/pitch slicing, model execution, audio IO, multiprocessing, CLI parsing, and filesystem behavior (`rewrite-in-rust/records/0045-confirm-slicer-segment-merge-boundary.md:7`, `rewrite-in-rust/records/0045-confirm-slicer-segment-merge-boundary.md:20`).

## Evidence

- The selected source helpers are confined to waveform concatenation, silence padding, segment duration/gap math, short-segment merging, and tiny-chunk merging (`inference/API/slicer_api.py:68`, `inference/API/slicer_api.py:73`, `inference/API/slicer_api.py:82`, `inference/API/slicer_api.py:86`, `inference/API/slicer_api.py:92`, `inference/API/slicer_api.py:103`, `inference/API/slicer_api.py:155`).
- The bootstrap correctly identifies that the owning module imports heavier dependencies, while the selected helpers only require NumPy-like shape inspection, zero allocation, concatenation, Python list/dict handling, float division, comparisons, and Python `round` behavior (`rewrite-in-rust/bootstrap/slicer_segment_merge_core.md:30`, `rewrite-in-rust/bootstrap/slicer_segment_merge_core.md:33`).
- The proposed seam is an independent Rust library module in `v2m-core` with no bridge dependencies and no Python/audio/model/process/filesystem work (`rewrite-in-rust/bootstrap/slicer_segment_merge_core.md:61`, `rewrite-in-rust/bootstrap/slicer_segment_merge_core.md:76`; `rewrite-in-rust/dependencies/slicer_segment_merge_core.yaml:20`).
- The hand-written replacement choice is justified: only NumPy shape, slicing, concatenate, and zeros behavior is needed, so a small waveform representation is sufficient and a broad ndarray/audio dependency is optional (`rewrite-in-rust/dependencies/slicer_segment_merge_core.yaml:31`).
- The kept-legacy decisions are explicit for RMS/default slicing, heuristic/grid policies, pitch/RMVPE smart slicing, audio IO, ASR, SoundFile/libsndfile, FFmpeg, argparse, and runtime behavior (`rewrite-in-rust/dependencies/slicer_segment_merge_core.yaml:37`).
- Fixture coverage includes mono/stereo concat, silence shapes, half-even gap rounding, overlap and positive gaps, short merge paths, recursive retry behavior, tiny leading/body/all/single cases, and the stereo `len(waveform) / sr` tiny-duration quirk (`rewrite-in-rust/fixtures/slicer_segment_merge_core.jsonl:1`, `rewrite-in-rust/fixtures/slicer_segment_merge_core.jsonl:9`, `rewrite-in-rust/fixtures/slicer_segment_merge_core.jsonl:15`, `rewrite-in-rust/fixtures/slicer_segment_merge_core.jsonl:21`).
- Dependency evidence supports the split: project manifests include `numpy`, `librosa`, `scipy`, `soundfile`, and ONNX packages for the broader module/runtime (`pyproject.toml:13`, `pyproject.toml:17`, `pyproject.toml:30`, `pyproject.toml:32`; `requirements.txt:8`, `requirements.txt:10`, `requirements.txt:14`, `requirements.txt:16`, `requirements.txt:17`), while vendored source records exist for NumPy and adjacent audio/runtime packages (`third_party/sources/manifest.json:473`, `third_party/sources/manifest.json:375`, `third_party/sources/manifest.json:779`, `third_party/sources/manifest.json:828`) and native/runtime source coverage remains clean (`third_party/source_audit.json:24`).

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_slicer_segment_merge_core.py`: pass, exit 0 with no output.
- `uv run python scripts/audit_vendored_sources.py`: pass. Output: `Source audit passed: 135 Python packages, 41 native-extension packages, 269 foreign runtime native binaries, 0 third_party binary artifacts.`
- `git diff --check`: pass, exit 0 with no output.
- `rg -n 'slicer_segment|SegmentMerge|merge_tiny|merge_short' rewrite-in-rust/rust/crates/v2m-core/src`: no matches, confirming no Rust implementation exists for this unit yet.

## Residual Risk

- The dependency/bootstrap gate proves the legacy fixture table and source inventory, but Rust parity is still unproven because implementation has not started.
- The fixtures use synthetic numeric arrays and intentionally do not cover real audio loading, dtype preservation, RMS behavior, Slicer state, librosa frame/time conversion, SoundFile/libsndfile, FFmpeg, multiprocessing, or model execution; those remain outside this unit by boundary decision.
- The private merge helpers assume caller-provided segment order in production paths; behavior review should keep the fixture contract explicit if a future Rust surface chooses to expose direct helper calls beyond the current sorted synthetic cases.

## Promotion Note

This role does not block dependency/bootstrap promotion. The manifest boundary should remain `confirmed`, not split, merged, deferred, or replaced. The coordinator still owns manifest updates and should not mark the unit `verified` from this review alone; writer work plus the required behavior and data/algorithm review gates remain outstanding.
