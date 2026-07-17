# slicer_pitch_override_core - data_algorithm_reviewer

Date: 2026-07-17
Decision: pass

## Findings

No findings.

The data/algorithm scope is consistent with the confirmed unit boundary. The
manifest marks `slicer_pitch_override_core` as a reimplemented, confirmed unit
for supplied voiced-mask smart slicing and explicitly requires preserving
voiced-mask indexing, longest-unvoiced cut choice, RMS fallback, offset
adjustment, and short/tiny merge behavior
(`rewrite-in-rust/manifest.yaml:1120`, `rewrite-in-rust/manifest.yaml:1128`).
Record 0053 limits this reviewable surface to supplied-mask behavior and keeps
`librosa.pyin`, RMVPE/model execution, audio IO, real multiprocessing, GUI, Web,
CLI parsing, and production routing legacy-owned
(`rewrite-in-rust/records/0053-confirm-slicer-pitch-override-boundary.md:22`,
`rewrite-in-rust/records/0053-confirm-slicer-pitch-override-boundary.md:33`).

Round/clip mask indexing is preserved for the covered path. Legacy Python uses
`ceil(len(y) / hop_length) + 1`, global frame times including
`segment_offset_sec`, NumPy round-to-even, and clipping into the supplied mask
(`inference/API/slicer_api.py:206`). Rust mirrors that with
`Waveform::sample_len`, a target-frame count, explicit half-even rounding, and
clamping to the supplied mask bounds
(`rewrite-in-rust/rust/crates/v2m-core/src/slicer_pitch.rs:129`,
`rewrite-in-rust/rust/crates/v2m-core/src/slicer_pitch.rs:149`,
`rewrite-in-rust/rust/crates/v2m-core/src/slicer_pitch.rs:443`). Fixture line 1
proves offset-sensitive round/clip behavior against both Python and Rust
(`rewrite-in-rust/fixtures/slicer_pitch_override_core.jsonl:1`).

Frame/time conversion and split policy match the legacy algorithm for fixture
inputs. Python converts window start/end seconds to frame indices, clips them
to voiced/RMS array lengths, cuts at the midpoint of the first longest unvoiced
run, and falls back to the first RMS argmin when no unvoiced frame exists
(`inference/API/slicer_api.py:503`, `inference/API/slicer_api.py:516`,
`inference/API/slicer_api.py:528`). Rust uses truncating seconds-to-samples
frame conversion, clipped frame ranges, first-longest unvoiced grouping,
first-min RMS fallback, and frame-to-time/sample slicing
(`rewrite-in-rust/rust/crates/v2m-core/src/slicer_pitch.rs:221`,
`rewrite-in-rust/rust/crates/v2m-core/src/slicer_pitch.rs:235`,
`rewrite-in-rust/rust/crates/v2m-core/src/slicer_pitch.rs:359`,
`rewrite-in-rust/rust/crates/v2m-core/src/slicer_pitch.rs:384`,
`rewrite-in-rust/rust/crates/v2m-core/src/slicer_pitch.rs:393`). Fixture lines
2-4 cover longest-unvoiced first-tie behavior, RMS fallback, and short early
return (`rewrite-in-rust/fixtures/slicer_pitch_override_core.jsonl:2`).

Outer policy data structures preserve the legacy offset and duration choices
within the fixture-backed scope. Python classifies segments using
`len(seg["waveform"]) / sr`, passes the parent segment offset into the split
helper, adds that offset back to local sub-chunks, sorts processed chunks, then
hands off to verified tiny/short merge helpers
(`inference/API/slicer_api.py:449`, `inference/API/slicer_api.py:590`,
`inference/API/slicer_api.py:617`, `inference/API/slicer_api.py:627`,
`inference/API/slicer_api.py:633`). Rust uses the same Python-`len`-compatible
`outer_len` for policy classification, carries parent offsets in
`PitchSplitRequest`, adds offsets to returned sub-chunks, sorts with
`total_cmp`, and calls the verified merge helpers
(`rewrite-in-rust/rust/crates/v2m-core/src/slicer_segment.rs:24`,
`rewrite-in-rust/rust/crates/v2m-core/src/slicer_pitch.rs:317`,
`rewrite-in-rust/rust/crates/v2m-core/src/slicer_pitch.rs:334`,
`rewrite-in-rust/rust/crates/v2m-core/src/slicer_pitch.rs:346`). Fixture lines
5-6 cover split-wrapper global offset handling, no-long short merge, and tiny
merge handoff (`rewrite-in-rust/fixtures/slicer_pitch_override_core.jsonl:5`).

Complexity is acceptable for the confirmed unit. The split loop advances by a
selected cut frame and each window scan is linear in the current voiced or RMS
slice, matching the Python grouping and `argmin` approach
(`inference/API/slicer_api.py:493`, `inference/API/slicer_api.py:516`,
`inference/API/slicer_api.py:538`;
`rewrite-in-rust/rust/crates/v2m-core/src/slicer_pitch.rs:208`,
`rewrite-in-rust/rust/crates/v2m-core/src/slicer_pitch.rs:359`,
`rewrite-in-rust/rust/crates/v2m-core/src/slicer_pitch.rs:384`). No new
algorithmic state, cache, or superlinear data structure was introduced.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_slicer_pitch_override_core.py`: passed.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml slicer_pitch`: passed; 3 tests passed, 0 failed, 89 filtered out.
- `git diff --check`: passed.
- `rg -n "pyin|RMVPE|rmvpe|voiced_flag_override|round|clip|time_to_frames|frames_to_time|argmin|groupby|ProcessPoolExecutor|outer_len|sample_len|NaN|nan" ...`: inspected; matches are expected legacy references, explicit exclusions, fixture/checker coverage, or Rust supplied-mask implementation points.
- Current diff inspected with `git diff -- rewrite-in-rust/manifest.yaml rewrite-in-rust/rust/crates/v2m-core/src/lib.rs` and `git ls-files --others --exclude-standard | rg 'slicer_pitch_override_core|slicer_pitch.rs|0053'`.

## Residual Risk

This role did not review behavior outside the supplied-mask unit boundary.
`librosa.pyin`, RMVPE/model execution, real `ProcessPoolExecutor` scheduling,
audio IO, production bridge validation, logging text, and Python-facing error
mapping remain legacy-owned. The fixture table does not stress non-finite
numeric inputs, negative offsets, ragged stereo payloads, or NaN RMS fallback
ordering; those should be handled by promotion-time payload validation or an
explicit compatibility record before accepting external Rust-facing inputs.

## Promotion Note

This data/algorithm role does not block promotion. Coordinator state should only
advance after the remaining required review roles for `slicer_pitch_override_core`
also pass.
