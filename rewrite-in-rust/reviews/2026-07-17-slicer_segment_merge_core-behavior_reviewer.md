# slicer_segment_merge_core - behavior_reviewer

Date: 2026-07-17
Decision: pass

## Findings

No behavior findings for this review.

## Scope Reviewed

- Unit and role reviewed: `slicer_segment_merge_core`, `behavior_reviewer`.
- Writer/reviewer separation: confirmed. This review inspected the implementation and wrote only this report; it did not edit production code, fixtures, bootstrap files, or the manifest.
- Manifest state: `rewrite-in-rust/manifest.yaml:982` keeps the unit as `slicer_segment_merge_core`, `rewrite-in-rust/manifest.yaml:984` marks it `reimplemented`, and `rewrite-in-rust/manifest.yaml:1003` keeps rollback on the Python merge helpers.
- Boundary: the confirmed unit covers `_concat_waveforms`, `_silence_like`, `_segment_duration_sec`, `_merged_duration_sec`, `_merge_segments`, `_merge_short_segments`, and `_merge_tiny_chunks`, while excluding RMS/default slicing, heuristic/grid/pitch slicing, model execution, audio IO, multiprocessing, CLI parsing, and filesystem behavior (`rewrite-in-rust/bootstrap/slicer_segment_merge_core.md:5`, `rewrite-in-rust/bootstrap/slicer_segment_merge_core.md:23`, `rewrite-in-rust/records/0045-confirm-slicer-segment-merge-boundary.md:7`, `rewrite-in-rust/records/0045-confirm-slicer-segment-merge-boundary.md:20`).

## Parity Notes

- Python chooses concat axis `-1` for multidimensional arrays and `0` for mono arrays at `inference/API/slicer_api.py:68`; Rust mirrors that split with `Waveform::Mono` and `Waveform::Stereo` concatenation at `rewrite-in-rust/rust/crates/v2m-core/src/slicer_segment.rs:45`.
- Python `_silence_like` returns zero-length slices for non-positive sample counts and zero-filled arrays preserving the last-axis sample length for positive counts (`inference/API/slicer_api.py:73`); Rust mirrors those shapes at `rewrite-in-rust/rust/crates/v2m-core/src/slicer_segment.rs:68`.
- Python duration and merge math use `waveform.shape[-1] / sr`, non-negative timeline gaps, and Python half-even `round` for inserted silence samples (`inference/API/slicer_api.py:82`, `inference/API/slicer_api.py:86`, `inference/API/slicer_api.py:92`); Rust uses `sample_len`, non-negative gaps, and a local half-even rounder at `rewrite-in-rust/rust/crates/v2m-core/src/slicer_segment.rs:84`, `rewrite-in-rust/rust/crates/v2m-core/src/slicer_segment.rs:89`, `rewrite-in-rust/rust/crates/v2m-core/src/slicer_segment.rs:96`, and `rewrite-in-rust/rust/crates/v2m-core/src/slicer_segment.rs:193`.
- Python short-segment behavior is a caller-order greedy scan, reverse tail merge, and recursive retry only after progress (`inference/API/slicer_api.py:103`, `inference/API/slicer_api.py:131`, `inference/API/slicer_api.py:146`); Rust preserves the same control flow at `rewrite-in-rust/rust/crates/v2m-core/src/slicer_segment.rs:107`, `rewrite-in-rust/rust/crates/v2m-core/src/slicer_segment.rs:132`, and `rewrite-in-rust/rust/crates/v2m-core/src/slicer_segment.rs:147`.
- Python tiny-segment behavior uses legacy `len(waveform) / sr`, which means stereo duration uses channel count rather than sample count (`inference/API/slicer_api.py:155`, `inference/API/slicer_api.py:163`); Rust intentionally mirrors this with `outer_len` at `rewrite-in-rust/rust/crates/v2m-core/src/slicer_segment.rs:24` and `rewrite-in-rust/rust/crates/v2m-core/src/slicer_segment.rs:157`.
- The fixture table covers mono/stereo concat, silence shapes, positive and overlapping gaps, half-even gap rounding, short merge skip/reverse/recursive paths, empty inputs, tiny leading/body/all/single cases, and the stereo `len(waveform)` quirk (`rewrite-in-rust/fixtures/slicer_segment_merge_core.jsonl:1`, `rewrite-in-rust/fixtures/slicer_segment_merge_core.jsonl:9`, `rewrite-in-rust/fixtures/slicer_segment_merge_core.jsonl:15`, `rewrite-in-rust/fixtures/slicer_segment_merge_core.jsonl:21`). The Python checker exercises the same table against the legacy helpers at `rewrite-in-rust/bootstrap/check_slicer_segment_merge_core.py:54`, and the Rust test consumes the same JSONL at `rewrite-in-rust/rust/crates/v2m-core/src/slicer_segment.rs:213` and `rewrite-in-rust/rust/crates/v2m-core/src/slicer_segment.rs:277`.
- Runtime rollback remains intact. The new Rust module is exposed from `v2m-core` at `rewrite-in-rust/rust/crates/v2m-core/src/lib.rs:21`, but production slicing still calls the Python helpers at `inference/API/slicer_api.py:433`, `inference/API/slicer_api.py:442`, `inference/API/slicer_api.py:633`, and `inference/API/slicer_api.py:642`.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_slicer_segment_merge_core.py`: pass, exit 0 with no output.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml slicer_segment`: pass; 2 `slicer_segment` tests passed, 76 `v2m_core` tests and 5 `v2m_quant_bridge` tests were filtered out.
- `git diff --check`: pass, exit 0 with no output.
- `rg -n "slicer_segment|concat_waveforms|merge_short_segments|merge_tiny_chunks|merge_segments" --glob '!rewrite-in-rust/rust/target/**'`: inspected; Rust hits are limited to rewrite artifacts and `v2m-core`, while production runtime hits remain the legacy Python helper definitions and callers.

## Residual Risk

This role proves behavior for the confirmed synthetic mono/channel-major stereo fixture boundary. It does not prove real audio loading, RMS/default slicing, heuristic/grid/pitch slicing, RMVPE/ASR execution, multiprocessing, SoundFile/FFmpeg/filesystem behavior, or a Python/Rust bridge.

Invalid sample rates, mismatched waveform dimensionality, ragged stereo payloads, and diagnostic `print` text are not covered by the fixture contract. Before any runtime promotion, the bridge or adapter should either preserve the current Python preconditions and diagnostics explicitly or add focused promotion fixtures for those cases.

## Promotion Note

This behavior role does not block promotion evidence for `slicer_segment_merge_core`. The coordinator should not mark the unit `verified` from this report alone; the manifest also lists a required `data_algorithm_reviewer` gate for this unit.
