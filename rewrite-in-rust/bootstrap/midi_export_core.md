# midi_export_core Bootstrap

## Boundary

`midi_export_core` covers the deterministic MIDI note export behavior in
`inference/io/note_io.py::_save_midi`.

The compatibility surface is:

- finite-note filtering by finite `onset`, `offset`, and `pitch`
- invalid note filtering when `offset <= onset`
- stable sorting by onset after invalid notes are removed
- `mido.bpm2tempo(tempo)` tempo meta event
- absolute onset and offset tick calculation with Python half-even `round`
- clamping overlapping note onsets to the prior note offset
- extending notes with non-positive tick duration to one tick
- MIDI pitch rounding with Python half-even `round` and clamping to `0..127`
- optional lyrics meta event before note-on, consuming the note-on delta
- note-on and note-off velocity `100` on channel `0`
- Mido's type-1, single-track, 480 ticks-per-beat file shape
- UTF-8 lyric payload encoding through `MidiFile(charset="utf8")`
- automatic `end_of_track` meta event added during save

Production parent-directory creation, `MidiFile.save(filepath)` filesystem
effects, print output, runtime export routing, and promotion wiring stay
legacy-owned.

## Dependency Expansion

`inference/io/note_io.py` imports:

- stdlib: `csv`, `pathlib`, `dataclasses.dataclass`, `typing.Literal`
- third party: `librosa`, `mido`, `numpy`

The selected `_save_midi` path uses:

- `_finite_notes` for scalar finite checks and `offset > onset`
- `numpy.isfinite`, `numpy.clip`, and Python/NumPy half-even rounding behavior
- `mido.bpm2tempo`
- `mido.MidiTrack`
- `mido.MetaMessage("set_tempo")`
- `mido.MetaMessage("lyrics")`
- `mido.Message("note_on")`
- `mido.Message("note_off")`
- `mido.MidiFile(charset="utf8").save`

Dependency evidence:

- `pyproject.toml` and `requirements.txt` include `mido` and `numpy<2.0.0`.
- `uv.lock` records `mido==1.3.3` and `numpy==1.26.4`.
- `third_party/sources/manifest.json` records source directories:
  `third_party/sources/mido-1.3.3` and
  `third_party/sources/numpy-1.26.4`.
- `third_party/source_audit.json` reports all foreign runtime native binaries
  covered and zero `third_party` binary artifacts.
- `third_party/native_sources/manifest.json` records OpenBLAS source coverage
  for NumPy/SciPy, but this unit needs only scalar finite, clip, and rounding
  behavior, not BLAS or array kernels.

The Mido reference paths used by the narrow Rust replacement are:

- `third_party/sources/mido-1.3.3/mido/midifiles/units.py`
- `third_party/sources/mido-1.3.3/mido/midifiles/meta.py`
- `third_party/sources/mido-1.3.3/mido/midifiles/midifiles.py`
- `third_party/sources/mido-1.3.3/mido/midifiles/tracks.py`
- `third_party/sources/mido-1.3.3/mido/messages/messages.py`
- `third_party/sources/mido-1.3.3/mido/messages/specs.py`
- `third_party/sources/mido-1.3.3/mido/messages/checks.py`

Do not add a broad Rust MIDI dependency for this unit. The compatibility surface
is smaller and more stable than package parity.

## Seam

Use the default rewrite seam:

- kind: independent Rust library
- crate: `rewrite-in-rust/rust/crates/v2m-core`
- module: `midi_export`
- runtime owner: legacy Python
- bridge dependencies: none

The Rust surface should return an in-memory event plan and MIDI bytes. It should
not create directories, write files, print status lines, call Python, or expose a
runtime router.

## Fixture Harness

Python/Rust parity uses:

```text
rewrite-in-rust/fixtures/midi_export_core.jsonl
```

Each fixture case contains:

- `tempo`
- note rows with string numeric fields so NaN and infinities can be represented
- expected skipped invalid note count
- expected MIDI file type and ticks per beat
- expected inspected event list from `mido.MidiFile(..., charset="utf8")`
- exact expected MIDI file hex from legacy `_save_midi`

The legacy Python side is checked by:

```bash
uv run python rewrite-in-rust/bootstrap/check_midi_export_core.py
```

The Rust side is checked by:

```bash
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml midi_export
```

## Repeated-Call Behavior

The renderer is deterministic for a fixed note list and tempo. Repeated calls
must return the same skipped count, event list, and MIDI bytes. It must not
depend on filesystem state, model state, GUI state, Web state, global audio
runtime state, or Mido port/backend state.

## Rollback

Rollback is keeping all production imports unchanged:

```text
inference.io.note_io._save_midi
```

No production Python caller should import Rust MIDI output until a later
promotion record chooses and verifies filesystem-write behavior and error
mapping.
