# slicer_segment_merge_core - data_algorithm_reviewer

Date: 2026-07-17
Decision: pass-with-followups

## Findings

- Severity: low
- Location: `rewrite-in-rust/rust/crates/v2m-core/src/slicer_segment.rs:10`
- Issue: Rust can represent ragged or channel-count-mismatched stereo payloads, and `concat_waveforms` extends/pushes channels instead of rejecting incompatible stereo shapes. Legacy Python delegates `_concat_waveforms` to `np.concatenate(..., axis=-1)`, which requires non-concat dimensions to match for normal 2-D arrays. The current fixtures cover valid mono/stereo shape semantics, but not malformed stereo channel counts.
- Evidence: Rust models stereo as `Vec<Vec<f64>>` and derives sample length from the first channel (`rewrite-in-rust/rust/crates/v2m-core/src/slicer_segment.rs:10`, `rewrite-in-rust/rust/crates/v2m-core/src/slicer_segment.rs:16`), while stereo concat mutates matching channels and pushes extra right-side channels (`rewrite-in-rust/rust/crates/v2m-core/src/slicer_segment.rs:53`). Python selects concat axis only, leaving shape compatibility to NumPy (`inference/API/slicer_api.py:68`). The fixture table covers same-channel stereo concat/merge/tiny cases (`rewrite-in-rust/fixtures/slicer_segment_merge_core.jsonl:2`, `rewrite-in-rust/fixtures/slicer_segment_merge_core.jsonl:10`, `rewrite-in-rust/fixtures/slicer_segment_merge_core.jsonl:21`).
- Required fix: Non-blocking before this fixture-bound state update. Before any runtime bridge or promotion, either enforce a rectangular, same-channel stereo invariant at the Rust input boundary, or add malformed-shape fixtures plus an explicit error-mapping decision. If the bridge will only accept validated channel-major stereo payloads, record that invariant in the promotion/bootstrap evidence.

## Scope Confirmation

- Unit and role reviewed: `slicer_segment_merge_core`, `data_algorithm_reviewer`.
- Manifest state: the unit is `reimplemented`, confirmed, and requires this data/algorithm review alongside dependency and behavior reviews (`rewrite-in-rust/manifest.yaml:982`, `rewrite-in-rust/manifest.yaml:984`, `rewrite-in-rust/manifest.yaml:989`).
- Confirmed boundary: the record and bootstrap limit this unit to deterministic `_concat_waveforms`, `_silence_like`, `_segment_duration_sec`, `_merged_duration_sec`, `_merge_segments`, `_merge_short_segments`, and `_merge_tiny_chunks`, excluding RMS/default slicing, heuristic/grid/pitch slicing, model execution, audio IO, multiprocessing, CLI parsing, filesystem behavior, and production bridge wiring (`rewrite-in-rust/records/0045-confirm-slicer-segment-merge-boundary.md:7`, `rewrite-in-rust/records/0045-confirm-slicer-segment-merge-boundary.md:20`, `rewrite-in-rust/bootstrap/slicer_segment_merge_core.md:5`, `rewrite-in-rust/bootstrap/slicer_segment_merge_core.md:23`).
- Writer/reviewer separation: confirmed for this review. I only inspected sources and wrote this review report; I did not edit production code, fixtures, bootstrap records, dependency records, or `rewrite-in-rust/manifest.yaml`.

## Data And Algorithm Assessment

- Valid waveform shape semantics match the confirmed boundary: mono uses sample-vector length, stereo uses channel-major last-axis sample length for segment duration, and `silence_like` preserves the channel count for zero and positive gaps (`inference/API/slicer_api.py:73`, `inference/API/slicer_api.py:82`, `rewrite-in-rust/rust/crates/v2m-core/src/slicer_segment.rs:16`, `rewrite-in-rust/rust/crates/v2m-core/src/slicer_segment.rs:68`).
- Gap duration and merge math preserve the legacy timeline policy for finite, positive sample rates: Python computes left end from `shape[-1] / sr`, clamps negative gaps to zero, and inserts `int(round(gap * sr))` samples; Rust mirrors this with `sample_len`, non-negative gap math, and a local half-even rounder (`inference/API/slicer_api.py:86`, `inference/API/slicer_api.py:92`, `rewrite-in-rust/rust/crates/v2m-core/src/slicer_segment.rs:89`, `rewrite-in-rust/rust/crates/v2m-core/src/slicer_segment.rs:96`, `rewrite-in-rust/rust/crates/v2m-core/src/slicer_segment.rs:193`).
- Short-segment recursion terminates for finite input because recursion is only entered when the previous pass shortened the list. Runtime can still be quadratic in repeated merge chains because every merge copies waveform payloads, but that matches the legacy repeated NumPy concatenation behavior and is acceptable for this narrow compatibility unit (`inference/API/slicer_api.py:146`, `rewrite-in-rust/rust/crates/v2m-core/src/slicer_segment.rs:147`).
- Tiny-segment behavior intentionally preserves the legacy stereo quirk that uses Python `len(waveform) / sr`, so stereo duration is channel-count based in this helper rather than sample-count based. Rust mirrors this with `outer_len` (`inference/API/slicer_api.py:163`, `rewrite-in-rust/rust/crates/v2m-core/src/slicer_segment.rs:24`, `rewrite-in-rust/rust/crates/v2m-core/src/slicer_segment.rs:157`).
- Fixture adequacy is good for the confirmed boundary: the JSONL table covers mono/stereo concat, zero and positive silence shapes, positive and overlapping gaps, half-even gap rounding, stereo segment merges, empty short/tiny inputs, forward skip/reverse/recursive short-merge behavior, leading/body/all/single tiny cases, and the stereo `len(waveform)` behavior (`rewrite-in-rust/fixtures/slicer_segment_merge_core.jsonl:1`, `rewrite-in-rust/fixtures/slicer_segment_merge_core.jsonl:9`, `rewrite-in-rust/fixtures/slicer_segment_merge_core.jsonl:15`, `rewrite-in-rust/fixtures/slicer_segment_merge_core.jsonl:21`).

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_slicer_segment_merge_core.py`: pass, exit 0 with no output.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml slicer_segment`: pass; 2 `slicer_segment` tests passed, 76 `v2m_core` tests and 5 `v2m_quant_bridge` tests were filtered out.
- `git diff --check`: pass, exit 0 with no output.
- `git diff --name-status` and `git diff --cached --name-status` before writing this report: no tracked code diff was present at review time.

## Residual Risk

- The Rust representation does not encode NumPy ndarray dtype, arbitrary-rank arrays, or rectangular-channel invariants. This is acceptable for the current synthetic mono/stereo fixture seam, but must be settled before runtime promotion.
- Non-finite offsets, non-positive sample rates, and extremely large gap durations are not fixture-covered. Legacy callers are expected to provide normal finite offsets and positive integer sample rates; promotion should either preserve that precondition or define validation/error behavior.
- No benchmark was required for this helper. The algorithmic complexity is no worse than legacy behavior for the fixture-bound unit, but repeated concatenation remains copy-heavy for long merge chains.

## Promotion Note

This data/algorithm role does not block coordinator state update. The coordinator can use this report as durable promotion evidence together with the existing dependency and behavior reviews, provided the stereo-shape invariant follow-up is tracked or accepted as a promotion-time precondition.
