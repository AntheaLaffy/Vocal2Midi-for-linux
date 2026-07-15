# 0006 - Keep TXT/CSV Export Formatting Narrow

## Context

The provisional `note_text_csv_export_core` unit pointed at
`inference/io/note_io.py`, which imports `librosa`, `mido`, and `numpy`.
Dependency discovery showed that `_save_text` only needs deterministic text row
rendering, while the same module also owns MIDI and other export-adjacent
helpers.

The pitch-name branch calls:

```text
librosa.midi_to_note(float(np.clip(pitch, 0, 127)), unicode=False, cents=not round_pitch)
```

`librosa.midi_to_note` uses NumPy-style half-even rounding for both MIDI note
selection and cent suffix calculation. Rust's default floating-point `round`
does not match that tie behavior.

## Decision

Confirm `note_text_csv_export_core` as a narrow TXT/CSV rendering unit:

- keep MIDI, USTX, TextGrid, filesystem writes, and production export routing
  legacy-owned;
- implement finite filtering, row ordering, lyric-column selection, CSV quoting,
  and pitch formatting directly in Rust;
- preserve half-even rounding in the Rust pitch formatter instead of relying on
  Rust's default half-away rounding.

Do not add Rust dependencies for NumPy, Librosa, or Mido for this unit.

## Consequences

- The Rust implementation remains fixture-bound and independently testable.
- Future pitch-formatting or export promotion work must account for half-even
  rounding before comparing against Python/Librosa output.
- A future bridge must separately map filesystem errors and the legacy skipped
  note warning text; this verified unit only owns in-memory rendering.

## Reversal

If a later promotion requires broader export ownership, add a separate bridge or
export-promotion record. Keep this unit's fixtures as the regression baseline
for TXT/CSV row rendering and pitch-name formatting.
