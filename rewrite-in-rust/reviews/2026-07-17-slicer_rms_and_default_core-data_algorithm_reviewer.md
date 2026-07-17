# slicer_rms_and_default_core - data_algorithm_reviewer

Date: 2026-07-17
Decision: fail

## Findings

- Severity: high
- Location: rewrite-in-rust/rust/crates/v2m-core/src/slicer_default.rs:265
- Issue: The Rust trailing-silence branch can panic for valid legacy inputs when the trailing silent run reaches the end of `rms_list` before `max_sil_kept` is exhausted. Python clamps the exclusive slice end naturally in `rms_list[silence_start: silence_end + 1]` after `silence_end = min(total_frames, silence_start + max_sil_kept)` (`inference/slicer/slicer2.py:129`-`133`). Rust computes the same clamped value, but then uses it as an inclusive end in `&rms_list[start..=silence_end]` (`rewrite-in-rust/rust/crates/v2m-core/src/slicer_default.rs:261`-`267`). If `silence_end == total_frames`, the inclusive range indexes one past the last frame.
- Evidence: The current trailing fixture only covers `start + max_sil_kept < total_frames` (`rewrite-in-rust/fixtures/slicer_rms_and_default_core.jsonl:13`), so both the Python and Rust fixture checks pass. A Python probe for a valid uncovered case returns normally: `uv run python -c "import numpy as np; from inference.slicer.slicer2 import Slicer; s=Slicer(sr=10, threshold=-20, min_length=400, min_interval=200, hop_size=100, max_sil_kept=1000); chunks=s.slice(np.asarray([1,1,1,1,0,0,0,0], dtype=np.float32)); print([(c['offset'], c['waveform'].tolist()) for c in chunks])"` printed `[(0.0, [1.0, 1.0, 1.0, 1.0, 0.0])]`. The Rust range in the same branch would include index `total_frames`.
- Required fix: Change the Rust trailing branch to use an exclusive slice end clamped to `rms_list.len()`, for example `let slice_end = (start + self.max_sil_kept + 1).min(total_frames); let pos = argmin(&rms_list[start..slice_end]) + start;`, and add a fixture where trailing silence length is less than or equal to `max_sil_kept`.

- Severity: low
- Location: rewrite-in-rust/fixtures/slicer_rms_and_default_core.jsonl:14
- Issue: The stereo fixture does not actually prove stereo averaging semantics. The boundary claims mono/stereo averaging and channel slicing (`rewrite-in-rust/bootstrap/slicer_rms_and_default_core.md:15`), Python computes `waveform.mean(axis=0)` for multi-dimensional input (`inference/slicer/slicer2.py:74`-`76`), and Rust averages all channels (`rewrite-in-rust/rust/crates/v2m-core/src/slicer_default.rs:355`-`374`). The only stereo fixture uses two identical channels, so a first-channel-only implementation would still pass the fixture table.
- Evidence: `slice_stereo_averages_for_rms_and_slices_channels` has identical channel payloads in both rows at `rewrite-in-rust/fixtures/slicer_rms_and_default_core.jsonl:14`. The Rust fixture parser and assertion only replay that table (`rewrite-in-rust/rust/crates/v2m-core/src/slicer_default.rs:537`-`582`).
- Required fix: Add a non-identical channel-major stereo fixture where averaging changes the RMS silence decision, while still proving the returned chunks slice every channel along the sample axis.

## Scope

Unit reviewed: `slicer_rms_and_default_core`.

Role reviewed: `data_algorithm_reviewer` only.

The confirmed boundary is pure RMS/default silence slicing over synthetic mono and channel-major stereo waveforms (`rewrite-in-rust/records/0046-confirm-slicer-rms-default-boundary.md:7`-`20`). I did not review behavior, dependency/bootstrap, Rust style, architecture, error tracing, or product ergonomics roles.

Writer/reviewer separation is preserved in this pass. I edited only this review report.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_slicer_rms_and_default_core.py`: passed with no output.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml slicer_default`: passed; 3 tests passed and 78 filtered out in `v2m-core`, plus 0 bridge tests.
- `git diff --check`: passed with no output.
- `uv run python -c "import numpy as np; from inference.slicer.slicer2 import Slicer; s=Slicer(sr=10, threshold=-20, min_length=400, min_interval=200, hop_size=100, max_sil_kept=1000); chunks=s.slice(np.asarray([1,1,1,1,0,0,0,0], dtype=np.float32)); print([(c['offset'], c['waveform'].tolist()) for c in chunks])"`: passed and printed the expected legacy output for the missing trailing-silence case.

## Residual Risk

The Rust implementation uses `f64` payloads and RMS math (`rewrite-in-rust/rust/crates/v2m-core/src/slicer_default.rs:99`-`123`), while the Python fixture harness coerces waveform inputs to `np.float32` (`rewrite-in-rust/bootstrap/check_slicer_rms_and_default_core.py:22`-`23`). Existing fixtures keep RMS values far from the threshold, so near-threshold `< self.threshold` behavior remains unpinned.

Malformed bridge payload policy remains promotion work. Rust rejects empty/ragged stereo payloads (`rewrite-in-rust/rust/crates/v2m-core/src/slicer_default.rs:355`-`364`), while legacy Python normally receives rectangular NumPy arrays. The bootstrap record already requires promotion-time waveform payload validation and error mapping (`rewrite-in-rust/bootstrap/slicer_rms_and_default_core.md:107`-`109`).

## Promotion Note

This role blocks promotion and should not be used to mark the unit verified. The coordinator can use this gate as failure evidence for the trailing-silence algorithm defect, then request a rerun after the Rust branch and fixture coverage are fixed.
