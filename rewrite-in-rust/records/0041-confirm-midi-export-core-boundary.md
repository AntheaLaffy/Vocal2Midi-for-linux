# 0041 - Confirm MIDI Export Core Boundary

## Context

`inference/io/note_io.py::_save_midi` builds a small deterministic MIDI file
through Mido:

- filter invalid notes with `_finite_notes`;
- sort remaining notes by onset;
- emit a `set_tempo` meta event from `mido.bpm2tempo`;
- calculate onset/offset ticks with Python `round(note_time * tempo * 8)`;
- clamp overlapping onsets to the previous note's offset;
- enforce a minimum one-tick duration;
- clamp rounded MIDI pitch into `0..127`;
- emit optional lyric meta events before note-on;
- write a single-track type-1 MIDI file with UTF-8 meta text.

Mido and NumPy are available as vendored source evidence, but full package
parity would be broader than this public export behavior.

## Decision

Confirm `midi_export_core` as a fixture-bound Rust library unit covering the
selected MIDI event plan and exact Standard MIDI File byte rendering for the
subset used by `_save_midi`.

Keep legacy-owned:

- production filesystem writes, parent directory creation, and status printing;
- arbitrary Mido APIs, ports, playback, broad parser behavior, sysex, and
  unrelated meta/message types;
- runtime export routing and promotion wiring;
- TXT/CSV, USTX, TextGrid, model inference, and quantization dispatch behavior.

The Rust implementation may hand-write this subset instead of adding a broad
MIDI crate, as long as fixtures compare both inspected events and exact Mido
bytes.

## Consequences

The Python checker can generate `.mid` files in a temp directory and inspect
them with `mido.MidiFile(charset="utf8")`. The Rust implementation can remain
pure and in-memory, emitting both a typed event plan and the byte stream.

This makes the important compatibility details reviewable without introducing
a Python/Rust bridge or touching user-visible export paths.

## Reversal

Rollback is keeping `inference.io.note_io._save_midi` as the runtime owner. No
production bridge is introduced by this record.
