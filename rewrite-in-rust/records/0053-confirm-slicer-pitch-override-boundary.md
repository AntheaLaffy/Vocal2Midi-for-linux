# 0053 - Confirm Slicer Pitch Override Boundary

Date: 2026-07-17

## Context

After the RMS/window, heuristic, and grid slicer policy units were verified,
the next manifest unit is `slicer_pitch_override_core`.

Dependency expansion of `inference/API/slicer_api.py` showed two very different
smart-slicing paths:

- the `librosa.pyin` fallback path, which estimates pitch directly from audio;
- the supplied voiced-mask path, used when RMVPE or another caller provides a
  boolean voiced mask and positive time step.

Stage 1 excludes model execution, and the manifest already requires supplied
boolean voiced-mask fixtures that do not call `librosa.pyin` or RMVPE.

## Decision

Confirm `slicer_pitch_override_core` as the supplied-voiced-mask smart slicing
unit.

The unit covers:

- voiced-mask round/clip indexing in `get_pitch_curve`;
- `_pitch_based_split` cut selection over supplied voiced flags;
- RMS fallback policy using the already verified RMS dependency;
- `_split_wrapper` global offset handling;
- `pitch_based_slice` outer orchestration and merge-helper handoff.

Keep `librosa.pyin`, RMVPE model execution, audio IO, real multiprocessing
scheduling, GUI, Web, CLI parsing, and production routing legacy-owned.

## Consequences

The fixture harness uses synthetic waveform arrays and boolean masks. It
monkeypatches heavy or already-verified dependencies where needed so the unit
does not re-cover default Slicer internals, RMS internals, merge internals, or
process-pool mechanics.

The Rust target is `rewrite-in-rust/rust/crates/v2m-core/src/slicer_pitch.rs`
with no production bridge.

The next unit after this gate remains `lyric_sequence_alignment_core` unless
dependency discovery re-cuts the remaining Stage 1 inventory.

## Reversal

Rollback is keeping `inference.API.slicer_api.get_pitch_curve`,
`inference.API.slicer_api._pitch_based_split`, and
`inference.API.slicer_api.pitch_based_slice` as runtime owners. No production
bridge is introduced.
