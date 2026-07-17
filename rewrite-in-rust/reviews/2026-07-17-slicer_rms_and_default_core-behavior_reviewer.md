# slicer_rms_and_default_core - behavior_reviewer

Date: 2026-07-17
Decision: fail

## Findings

- Severity: high
- Location: rewrite-in-rust/rust/crates/v2m-core/src/slicer_default.rs:265
- Issue: Rust can panic instead of preserving Python trailing-silence behavior when the final silent run is at least `min_interval` frames but shorter than, or equal to, the remaining `max_sil_kept` window. Python clamps the search end with `min(total_frames, silence_start + max_sil_kept)` and then uses an exclusive NumPy slice that safely clips past EOF (`inference/slicer/slicer2.py:130`). Rust computes the same `silence_end`, but then indexes `rms_list[start..=silence_end]` (`rewrite-in-rust/rust/crates/v2m-core/src/slicer_default.rs:266`), so `silence_end == rms_list.len()` makes the inclusive range out of bounds. This is in the accepted boundary because the record covers `Slicer.slice` and trailing silence handling (`rewrite-in-rust/records/0046-confirm-slicer-rms-default-boundary.md:15`), and the manifest requires preserving leading/middle/trailing silence handling plus chunk waveform slicing (`rewrite-in-rust/manifest.yaml:1017`).
- Evidence: A legacy accepted input with `params={"sr":10,"threshold":-20,"min_length":400,"min_interval":200,"hop_size":100,"max_sil_kept":300}` and `waveform=[1,1,1,1,0,0]` returns one chunk in Python: `{'offset': 0.0, 'waveform': [1.0, 1.0, 1.0, 1.0, 0.0]}`. Its RMS list has length 7 and the trailing silence starts at frame 5, so Rust reaches `silence_end = min(7, 5 + 3) = 7` and then attempts `rms_list[5..=7]`.
- Required fix: Use Python-equivalent trailing search bounds, for example an exclusive end or an inclusive end clamped to `total_frames - 1`, then add the accepted trailing-shorter-than-`max_sil_kept` fixture and rerun the Python and Rust slicer checks.

- Severity: medium
- Location: rewrite-in-rust/fixtures/slicer_rms_and_default_core.jsonl:13
- Issue: The fixture table has one trailing-silence case, but it only exercises the branch where `silence_start + max_sil_kept < total_frames`. The Rust fixture replay therefore passes while missing the EOF-clipping branch above (`rewrite-in-rust/rust/crates/v2m-core/src/slicer_default.rs:538`).
- Evidence: `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml slicer_default` passed all current fixture-driven tests, while the independent Python probe above demonstrates a covered trailing-silence input that is absent from the fixture table.
- Required fix: Add a JSONL `slice` fixture for trailing silence with `silence_start + max_sil_kept >= total_frames`, expecting the Python chunk output, and keep the existing long-trailing-silence fixture.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_slicer_rms_and_default_core.py`: passed.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml slicer_default`: passed; 3 slicer_default tests passed.
- `git diff --check`: passed.
- `uv run python -c '...'` trailing-short Python probe: passed; legacy Python returned one chunk for `waveform=[1,1,1,1,0,0]`.

## Residual Risk

This review covered the behavior gate only. The unit still has a separate data/algorithm review requirement in the manifest. Inputs outside the accepted synthetic finite mono/channel-major stereo boundary, such as broad NumPy/librosa pad modes or non-finite values, remain unreviewed here.

## Promotion Note

This role blocks coordinator state update for `stage_behavior_reviewer`. Do not mark the manifest verified from this gate until the trailing-silence parity issue is fixed and rerun.
