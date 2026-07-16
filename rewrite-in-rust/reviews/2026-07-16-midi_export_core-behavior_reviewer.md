# midi_export_core - behavior_reviewer

Date: 2026-07-16
Decision: pass-with-followups

## Findings

- Severity: low
- Location: rewrite-in-rust/fixtures/midi_export_core.jsonl:1
- Issue: The fixture table proves the main MIDI parity surface, but it does not include an exact half-tick onset/offset case or a same-onset valid-note tie. Those are documented behaviors through Python's stable `sorted(..., key=lambda n: n.onset)` and `round(note_time * tempo * 8)` tick conversion.
- Evidence: `inference/io/note_io.py:61` sorts valid notes by onset, and `inference/io/note_io.py:65`-`66` use Python half-even `round` for absolute ticks. The Rust implementation uses stable `sort_by` at `rewrite-in-rust/rust/crates/v2m-core/src/midi_export.rs:95` and half-even rounding at `rewrite-in-rust/rust/crates/v2m-core/src/midi_export.rs:103`-`105`, so current code matches by inspection. Existing fixture cases cover unsorted notes, overlap clamping, minimum one-tick duration, invalid skips, pitch clamp, UTF-8 lyrics, and all-invalid input, but not these two tie cases.
- Required fix: Add a future JSONL case with at least one exact `.5` tick boundary and same-onset valid notes, then rerun the MIDI checker and Rust `midi_export` tests.

## Scope Reviewed

- Python source: `inference/io/note_io.py::_finite_notes`, `_clamp_midi_pitch`, and `_save_midi`.
- Fixture: `rewrite-in-rust/fixtures/midi_export_core.jsonl`.
- Checker: `rewrite-in-rust/bootstrap/check_midi_export_core.py`.
- Rust implementation and tests: `rewrite-in-rust/rust/crates/v2m-core/src/midi_export.rs`.
- Manifest entry: `rewrite-in-rust/manifest.yaml` keeps runtime owner as legacy and rollback as `inference.io.note_io._save_midi`.
- Boundary records: `rewrite-in-rust/dependencies/midi_export_core.yaml`, `rewrite-in-rust/bootstrap/midi_export_core.md`, and `rewrite-in-rust/records/0041-confirm-midi-export-core-boundary.md`.

## Parity Notes

- Python filters non-finite notes and `offset <= onset` before sorting; Rust mirrors that and reports `skipped_invalid_notes`.
- Python emits `set_tempo`, optional `lyrics`, `note_on`, `note_off`, and Mido's automatic `end_of_track`; Rust emits the same event shape.
- Python clamps overlapping onsets to the previous absolute offset and extends non-positive tick durations to one tick; Rust mirrors both.
- Python clamps rounded MIDI pitch into `0..127`; Rust matches the covered half-even pitch and clamp cases.
- The checker validates inspected Mido events, MIDI type, ticks per beat, skipped invalid count, and exact MIDI hex. Rust tests compare the same fixture table against the Rust event plan and byte encoder.
- No production bridge is introduced. Filesystem writes, parent directory creation, and print output remain legacy-owned per rollback.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_midi_export_core.py`: pass.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml midi_export`: pass, 3 tests passed.

## Residual Risk

The review did not run full workspace clippy/fmt/doc checks because this role is behavior parity only. Large lyric payload lengths beyond one-byte MIDI variable-length quantities are not covered by a fixture, though the same variable-int encoder is exercised by delta-time tests.

## Promotion Note

This behavior review found no blocking Python/Rust parity mismatch. The unit can proceed from the behavior role with the low fixture-coverage follow-up above; the coordinator should still require the remaining declared review roles before marking the unit verified.
