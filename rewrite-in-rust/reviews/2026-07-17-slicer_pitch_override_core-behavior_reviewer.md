# slicer_pitch_override_core - behavior_reviewer

Date: 2026-07-17
Role: behavior_reviewer
Unit: slicer_pitch_override_core

## Findings

- Severity: low
- Location: `rewrite-in-rust/bootstrap/slicer_pitch_override_core.md:14`, `rewrite-in-rust/bootstrap/slicer_pitch_override_core.md:15`, `rewrite-in-rust/fixtures/slicer_pitch_override_core.jsonl:1`, `rewrite-in-rust/fixtures/slicer_pitch_override_core.jsonl:2`, `rewrite-in-rust/fixtures/slicer_pitch_override_core.jsonl:3`, `inference/API/slicer_api.py:506`, `inference/API/slicer_api.py:507`, `rewrite-in-rust/rust/crates/v2m-core/src/slicer_pitch.rs:221`
- Issue: Fixture coverage proves supplied-mask resampling clip behavior, longest-unvoiced selection, RMS fallback, short early return, offset adjustment, and merge handoff, but it does not directly exercise `_pitch_based_split` frame-window clamp behavior. The bootstrap names long-segment window frame conversion and clipping as covered behavior, while the current split fixtures keep the selected voiced/RMS windows inside vector bounds.
- Evidence: Python clamps split-window frame bounds against `len(voiced_flag)` at `inference/API/slicer_api.py:506` and `inference/API/slicer_api.py:507`; Rust mirrors that clamp at `rewrite-in-rust/rust/crates/v2m-core/src/slicer_pitch.rs:221` through `rewrite-in-rust/rust/crates/v2m-core/src/slicer_pitch.rs:225`. The fixture table has one direct resampling round/clip row at `rewrite-in-rust/fixtures/slicer_pitch_override_core.jsonl:1`, one longest-unvoiced split row at line 2, and one RMS fallback row at line 3, but no split row where the requested split window exceeds the available voiced or RMS vector length.
- Required fix: Add a follow-up fixture before any runtime bridge promotion that forces `_pitch_based_split` to clamp split-window start/end frames for supplied voiced flags, and preferably one RMS fallback row with a shortened RMS vector to prove fallback clamp behavior too.

- Severity: none
- Location: `inference/API/slicer_api.py:206`, `inference/API/slicer_api.py:210`, `inference/API/slicer_api.py:211`, `rewrite-in-rust/rust/crates/v2m-core/src/slicer_pitch.rs:129`, `rewrite-in-rust/rust/crates/v2m-core/src/slicer_pitch.rs:149`, `rewrite-in-rust/rust/crates/v2m-core/src/slicer_pitch.rs:153`
- Issue: No Python/Rust mismatch found for supplied voiced-mask override indexing.
- Evidence: Python bypasses `librosa.pyin` when a supplied mask and positive step are present, computes global frame times, uses NumPy round, and clips source indices at `inference/API/slicer_api.py:206` through `inference/API/slicer_api.py:212`. Rust mirrors the positive-step supplied-mask path and half-even round/clip lookup in `resample_voiced_override` at `rewrite-in-rust/rust/crates/v2m-core/src/slicer_pitch.rs:129` through `rewrite-in-rust/rust/crates/v2m-core/src/slicer_pitch.rs:157`. The fixture row at `rewrite-in-rust/fixtures/slicer_pitch_override_core.jsonl:1` covers offset-aware half-even rounding and upper-bound clipping.
- Required fix: none.

- Severity: none
- Location: `inference/API/slicer_api.py:475`, `inference/API/slicer_api.py:478`, `inference/API/slicer_api.py:511`, `inference/API/slicer_api.py:518`, `inference/API/slicer_api.py:530`, `inference/API/slicer_api.py:538`, `rewrite-in-rust/rust/crates/v2m-core/src/slicer_pitch.rs:181`, `rewrite-in-rust/rust/crates/v2m-core/src/slicer_pitch.rs:183`, `rewrite-in-rust/rust/crates/v2m-core/src/slicer_pitch.rs:229`, `rewrite-in-rust/rust/crates/v2m-core/src/slicer_pitch.rs:235`, `rewrite-in-rust/rust/crates/v2m-core/src/slicer_pitch.rs:384`
- Issue: No Python/Rust mismatch found for direct split behavior in the covered cases: short early return, first longest unvoiced-run midpoint, RMS fallback to first minimum, local offsets, and output order.
- Evidence: Python returns the original segment when `total_sec <= max_len_sec` at `inference/API/slicer_api.py:478`, groups unvoiced indices and updates only on `len(group) > max_len` at `inference/API/slicer_api.py:518`, and uses `np.argmin` for RMS fallback at `inference/API/slicer_api.py:538`. Rust mirrors sample-length duration, early return, first longest unvoiced midpoint, and first-min fallback through `pitch_based_split_with_override`, `longest_unvoiced_midpoint`, and `first_argmin`. Fixture rows at `rewrite-in-rust/fixtures/slicer_pitch_override_core.jsonl:2`, `rewrite-in-rust/fixtures/slicer_pitch_override_core.jsonl:3`, and `rewrite-in-rust/fixtures/slicer_pitch_override_core.jsonl:4` compare the resulting chunks against legacy Python.
- Required fix: none.

- Severity: none
- Location: `inference/API/slicer_api.py:449`, `inference/API/slicer_api.py:455`, `inference/API/slicer_api.py:457`, `inference/API/slicer_api.py:591`, `inference/API/slicer_api.py:620`, `inference/API/slicer_api.py:625`, `inference/API/slicer_api.py:629`, `rewrite-in-rust/rust/crates/v2m-core/src/slicer_pitch.rs:320`, `rewrite-in-rust/rust/crates/v2m-core/src/slicer_pitch.rs:333`, `rewrite-in-rust/rust/crates/v2m-core/src/slicer_pitch.rs:336`, `rewrite-in-rust/rust/crates/v2m-core/src/slicer_pitch.rs:342`, `rewrite-in-rust/rust/crates/v2m-core/src/slicer_pitch.rs:346`
- Issue: No Python/Rust mismatch found for outer `pitch_based_slice` orchestration in the covered cases: pre-slicer parameters, long/short classification, split offset adjustment, order restoration, and tiny/short merge handoff.
- Evidence: Python passes parent segment offset into `_split_wrapper` and then adds that offset to returned sub-chunks at `inference/API/slicer_api.py:455` through `inference/API/slicer_api.py:458`; classifies long segments with legacy `len(waveform) / sr` at `inference/API/slicer_api.py:591`; flattens executor output and sorts by offset at `inference/API/slicer_api.py:620` through `inference/API/slicer_api.py:629`; then calls the merge helpers at `inference/API/slicer_api.py:633` and `inference/API/slicer_api.py:642`. Rust mirrors classification, per-long request offsets, local-to-global offset adjustment, sorting, and merge helper handoff in `apply_pitch_override_policy`. Fixture rows at `rewrite-in-rust/fixtures/slicer_pitch_override_core.jsonl:5` and `rewrite-in-rust/fixtures/slicer_pitch_override_core.jsonl:6` prove one long split path and the no-long merge path.
- Required fix: none.

- Severity: none
- Location: `rewrite-in-rust/manifest.yaml:1120`, `rewrite-in-rust/manifest.yaml:1122`, `rewrite-in-rust/manifest.yaml:1124`, `rewrite-in-rust/manifest.yaml:1142`, `rewrite-in-rust/bootstrap/slicer_pitch_override_core.md:61`, `rewrite-in-rust/bootstrap/slicer_pitch_override_core.md:105`, `rewrite-in-rust/records/0053-confirm-slicer-pitch-override-boundary.md:49`
- Issue: Rollback and public ownership are preserved.
- Evidence: The manifest keeps `slicer_pitch_override_core` at `status: reimplemented` with legacy runtime ownership at `rewrite-in-rust/manifest.yaml:1122` and `rewrite-in-rust/manifest.yaml:1124`, then records rollback to the Python pitch slicer at `rewrite-in-rust/manifest.yaml:1142`. The bootstrap states the Rust seam has no bridge and legacy Python remains runtime owner at `rewrite-in-rust/bootstrap/slicer_pitch_override_core.md:61`, then names the Python rollback imports at `rewrite-in-rust/bootstrap/slicer_pitch_override_core.md:105`. Record 0053 confirms no production bridge is introduced at `rewrite-in-rust/records/0053-confirm-slicer-pitch-override-boundary.md:49`.
- Required fix: none.

## Decision

pass-with-followups

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_slicer_pitch_override_core.py`: pass.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml slicer_pitch`: pass; 3 focused `slicer_pitch` tests passed, 89 filtered in `v2m_core`, and 0 matching bridge tests ran.
- `git diff --check`: pass for tracked changes.
- `git status --short`: inspected current unit scope. The pitch unit artifacts are untracked additions; tracked changes in `rewrite-in-rust/manifest.yaml` and `rewrite-in-rust/rust/crates/v2m-core/src/lib.rs` include this unit among broader slicer-workspace changes.
- Targeted source inspection: compared `inference/API/slicer_api.py::get_pitch_curve`, `_pitch_based_split`, `_split_wrapper`, and `pitch_based_slice` against `rewrite-in-rust/rust/crates/v2m-core/src/slicer_pitch.rs`; inspected `rewrite-in-rust/fixtures/slicer_pitch_override_core.jsonl`, `rewrite-in-rust/bootstrap/check_slicer_pitch_override_core.py`, `rewrite-in-rust/dependencies/slicer_pitch_override_core.yaml`, `rewrite-in-rust/bootstrap/slicer_pitch_override_core.md`, and `rewrite-in-rust/records/0053-confirm-slicer-pitch-override-boundary.md`.

## Residual Risk

The behavior proof is fixture-bound and intentionally excludes `librosa.pyin`, RMVPE/model execution, real `ProcessPoolExecutor` scheduling, audio IO, GUI/Web/CLI routing, and production bridge error mapping. Supplied-mask invalid-input behavior is not fully proven for promotion cases such as empty masks, non-positive steps on long segments, non-finite offsets, malformed stereo payloads, or splitter dependency errors; those remain acceptable while Python is the runtime owner but need explicit validation/error policy before a bridge accepts external payloads.

## Promotion Note

This behavior review does not block the unit on observed Python/Rust parity. The low fixture coverage follow-up should be tracked before runtime bridge promotion, and coordinator state should still wait for the other required review roles listed for the unit before marking it verified.
