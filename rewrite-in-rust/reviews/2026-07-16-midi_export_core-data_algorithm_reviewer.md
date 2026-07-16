# midi_export_core - data_algorithm_reviewer

Date: 2026-07-16
Decision: pass

## Findings

No blocking findings.

Evidence:

- Python `_save_midi` filters invalid notes through `_finite_notes`, stable-sorts by onset, rounds onset/offset ticks with Python `round(note_time * tempo * 8)`, clamps overlapping onsets to the previous offset, enforces one tick minimum duration, clamps rounded MIDI pitch, places lyrics before note-on, and writes with `MidiFile(charset="utf8")` in `inference/io/note_io.py:19` and `inference/io/note_io.py:57`.
- Mido reference behavior matches the narrow Rust subset: `bpm2tempo` uses `round(60 * 1e6 / bpm)` in `third_party/sources/mido-1.3.3/mido/midifiles/units.py:27`; `MidiFile` defaults to type 1 and 480 ticks per beat in `third_party/sources/mido-1.3.3/mido/midifiles/midifiles.py:291`; save writes one header and each track in `third_party/sources/mido-1.3.3/mido/midifiles/midifiles.py:462`; `fix_end_of_track` appends the final end-of-track message in `third_party/sources/mido-1.3.3/mido/midifiles/tracks.py:84`; meta payload lengths use variable-length byte counts in `third_party/sources/mido-1.3.3/mido/midifiles/meta.py:105` and `third_party/sources/mido-1.3.3/mido/midifiles/meta.py:542`.
- The Rust implementation keeps the same data algorithm: finite filtering, stable `sort_by` on onset, half-even rounding, monotonic tick clamp, one-tick minimum, pitch clamp, event construction, and byte rendering are in `rewrite-in-rust/rust/crates/v2m-core/src/midi_export.rs:79`, `rewrite-in-rust/rust/crates/v2m-core/src/midi_export.rs:172`, `rewrite-in-rust/rust/crates/v2m-core/src/midi_export.rs:194`, `rewrite-in-rust/rust/crates/v2m-core/src/midi_export.rs:214`, and `rewrite-in-rust/rust/crates/v2m-core/src/midi_export.rs:274`.
- Running-status compression in Mido only omits a channel status byte when adjacent channel messages share the same status byte, as shown in `third_party/sources/mido-1.3.3/mido/midifiles/midifiles.py:238`. The `_save_midi` event pattern alternates note-on and note-off, and lyric meta events reset running status, so the hand-written byte construction is equivalent for this unit's public event plan.
- Fixture and Rust tests compare skipped counts, event lists, type/ticks-per-beat, exact SMF hex, invalid-note filtering, overlapping/min-duration behavior, UTF-8 lyrics, pitch clamp, tempo conversion, and selected VLQ edges in `rewrite-in-rust/fixtures/midi_export_core.jsonl:1`, `rewrite-in-rust/bootstrap/check_midi_export_core.py:63`, and `rewrite-in-rust/rust/crates/v2m-core/src/midi_export.rs:391`.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_midi_export_core.py`: pass. The checker regenerated all four legacy MIDI fixture cases through Python/Mido and matched event lists, type, ticks per beat, exact MIDI hex, and skipped invalid-note counts.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml midi_export`: pass. Three Rust tests passed, including fixture parity, invalid tempo rejection, and VLQ edge checks for 0, 127, 128, and 287.

## Residual Risk

The fixture table is intentionally narrow. It does not separately pin every numeric boundary, such as exact tick half-even ties, tempo values that round to 0 or 0x00ff_ffff, very large VLQ lengths, or track-size overflow. The implementation has explicit helper coverage or error paths for these areas, and the reviewed Python/Mido source supports the selected algorithm, but those outer edges are not golden-fixture evidence yet.

Complexity is acceptable for the unit: note filtering is O(n), stable onset sorting is O(n log n), and event/byte construction is linear in event count plus UTF-8 lyric byte length. The implementation stores the filtered note list, event list, and output bytes in memory, which matches the fixture-bound in-memory seam and does not affect the legacy production writer.

## Promotion Note

This data_algorithm_reviewer role does not block promotion. The coordinator should still wait for the other required review roles before marking `midi_export_core` verified.
