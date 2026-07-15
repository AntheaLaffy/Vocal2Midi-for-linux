# 0005 - Split GAME Alignment Unit

## Context

The provisional `game_word_note_alignment` manifest unit grouped four functions
from `inference/game/alignment_utils.py`:

- `validate_phones`
- `parse_words`
- `merge_consecutive_uv_words`
- `align_notes_to_words`

Dependency discovery showed two different capability shapes:

- `validate_phones`, `parse_words`, and `merge_consecutive_uv_words` are pure
  list/string/float helper behavior.
- `align_notes_to_words` owns timeline alignment behavior: cumulative
  boundaries, nearest-boundary snapping, monotonic clamping, note slicing, rest
  insertion, unvoiced-word handling, and slur flags.

The live GAME API caller imports only `align_notes_to_words`; the phone/word
helpers are related by data shape but are independently fixtureable.

## Decision

Replace the provisional `game_word_note_alignment` unit with two smaller units:

- `game_phone_word_parsing`
- `game_note_word_alignment`

Do not add a Rust NumPy/ndarray dependency for the note alignment unit at this
stage. The Python implementation uses `np.cumsum`, `np.abs`, and `np.argmin`
over small lists. A narrow Rust `Vec<f64>` implementation should be written only
after fixture outputs capture the Python behavior, including tolerance and tie
edge cases.

## Consequences

- The next implementation target can be the smaller
  `game_phone_word_parsing` unit if the project wants another low-risk helper
  before the live note alignment surface.
- `game_note_word_alignment` can receive a separate data/algorithm review before
  promotion because it contains timeline and numeric edge behavior.
- GAME ONNX inference, NumPy array preparation, librosa note conversion, HFA
  extraction, and production API integration remain legacy-owned.

## Reversal

If later fixture work shows the two units require a shared Rust data model, add
that model as a small prerequisite unit or merge the units in a new record. Do
not silently re-expand either unit during writer work.
