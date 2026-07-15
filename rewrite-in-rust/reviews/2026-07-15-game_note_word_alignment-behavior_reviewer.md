# game_note_word_alignment - behavior_reviewer

Date: 2026-07-15
Decision: pass

## Findings

No findings.

The reviewed behavior surface is `inference/game/alignment_utils.py::align_notes_to_words`.
The Python implementation returns empty outputs for empty word durations, note
durations, or note sequence, builds `np.float64` cumulative boundaries, snaps
internal word boundaries to the nearest note boundary within `tol`, uses NumPy
`argmin` first-minimum behavior, clamps boundaries with `_ALIGN_MIN_GAP`, slices
overlapping note spans, merges repeated adjacent note names inside a word,
inserts rests for spans with no note overlap, applies unvoiced-word rest
conversion when `apply_word_uv=True`, and emits `[0, 1, ...]` slur flags per
word. Evidence: `../inference/game/alignment_utils.py:92`.

The Rust implementation mirrors that public behavior in an independent library
function while keeping legacy Python as runtime owner. Evidence:
`rust/crates/v2m-core/src/game/note_word.rs:9`, `manifest.yaml:112`,
`manifest.yaml:128`.

The durable fixture table covers empty returns, exact boundaries, snapped and
unsnapped boundaries, monotonic clamp, repeated note merge, rest insertion,
`apply_word_uv`, unvoiced note consumption, slur reset, and NumPy first-tie
`argmin` behavior. Evidence: `fixtures/game_note_word_alignment.tsv:2`,
`fixtures/game_note_word_alignment.tsv:5`,
`fixtures/game_note_word_alignment.tsv:8`,
`fixtures/game_note_word_alignment.tsv:9`,
`fixtures/game_note_word_alignment.tsv:10`,
`fixtures/game_note_word_alignment.tsv:11`,
`fixtures/game_note_word_alignment.tsv:13`,
`fixtures/game_note_word_alignment.tsv:14`,
`fixtures/game_note_word_alignment.tsv:15`,
`fixtures/game_note_word_alignment.tsv:16`.

Python and Rust both consume the same fixture table and compare sequence,
duration, and slur outputs. Evidence:
`bootstrap/check_game_note_word_alignment.py:50`,
`rust/crates/v2m-core/src/game/note_word.rs:193`.

## Checks

- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml note_word`: passed, 2 tests passed, 0 failed, 17 filtered out.
- `uv run python rewrite-in-rust/bootstrap/check_game_note_word_alignment.py`: passed.

## Residual Risk

This review is limited to behavior parity for the fixture-backed
`align_notes_to_words` boundary. It does not promote a Python/Rust bridge, does
not change runtime ownership, and does not replace the required
`data_algorithm_reviewer` pass listed for this unit. Broader GAME ONNX
inference, NumPy array preparation, librosa note conversion, HFA word
extraction, and production API integration remain outside this unit.

## Promotion Note

This behavior review does not block promotion. The coordinator can use this
report as behavior-review evidence, while keeping rollback unchanged:
`inference.game.alignment_utils.align_notes_to_words` remains the runtime owner
until a later promotion record verifies a bridge.
