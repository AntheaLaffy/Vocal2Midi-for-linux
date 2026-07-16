# midi_export_core - error_tracing_reviewer

Date: 2026-07-16
Decision: pass-with-followups

## Findings

- Severity: low
- Location: rewrite-in-rust/rust/crates/v2m-core/src/midi_export.rs:63
- Issue: Rust exposes structured render failures as `MidiExportError` variants, but only `InvalidTempo` has direct edge coverage. `TempoOutOfRange`, `TickOutOfRange`, and `TrackTooLarge` are reachable from the implementation and documented as renderer errors, yet no unit test proves their mapping or failure messages.
- Evidence: `MidiExportError` defines `InvalidTempo`, `TempoOutOfRange`, `TickOutOfRange`, and `TrackTooLarge` at `rewrite-in-rust/rust/crates/v2m-core/src/midi_export.rs:63`; conversion sites return those variants at `rewrite-in-rust/rust/crates/v2m-core/src/midi_export.rs:172`, `rewrite-in-rust/rust/crates/v2m-core/src/midi_export.rs:194`, and `rewrite-in-rust/rust/crates/v2m-core/src/midi_export.rs:214`. Existing Rust edge coverage asserts only zero and NaN tempo mapping at `rewrite-in-rust/rust/crates/v2m-core/src/midi_export.rs:443`. Legacy probes show tempo `0` raises `ZeroDivisionError`, while negative or very low positive tempos fail Mido's 24-bit tempo range check before file writing.
- Required fix: Before runtime promotion, add Rust edge tests for `TempoOutOfRange` and `TickOutOfRange`; document or test the practical `TrackTooLarge` guard if it becomes caller-visible. This does not block the current fixture-bound reimplementation because production filesystem writes and caller-facing error mapping remain legacy-owned.

## Scope Notes

- The legacy `_save_midi` path creates the track, filters/sorts notes, emits events, creates parent directories, writes with `mido.MidiFile(charset="utf8")`, and prints status at `inference/io/note_io.py:57`.
- Skipped-note warning parity is preserved for the legacy runtime owner by `_finite_notes` printing `[Warning] Skipped ...` at `inference/io/note_io.py:19`. Rust returns `skipped_invalid_notes` at `rewrite-in-rust/rust/crates/v2m-core/src/midi_export.rs:53`; a future promotion must map that count back to the user-visible warning if Rust owns runtime export.
- Filesystem and write errors remain intentionally out of the Rust unit: bootstrap keeps parent-directory creation, `MidiFile.save(filepath)`, status printing, export routing, and promotion wiring legacy-owned at `rewrite-in-rust/bootstrap/midi_export_core.md:24`; rollback keeps `inference.io.note_io._save_midi` as runtime owner at `rewrite-in-rust/records/0041-confirm-midi-export-core-boundary.md:48`.
- UTF-8 lyric handling is covered at the byte boundary: bootstrap requires `MidiFile(charset="utf8")` parity at `rewrite-in-rust/bootstrap/midi_export_core.md:20`; Rust writes lyric byte length and `text.as_bytes()` at `rewrite-in-rust/rust/crates/v2m-core/src/midi_export.rs:243`; fixture case `invalid_negative_utf8` includes a UTF-8 lyric at `rewrite-in-rust/fixtures/midi_export_core.jsonl:3`.
- Test failure diagnostics are adequate for this stage: the Python checker includes `case_id` and fixture line for event mismatches at `rewrite-in-rust/bootstrap/check_midi_export_core.py:80`, and Rust parity assertions include case id plus fixture line for skipped count, type, ticks, events, and hex at `rewrite-in-rust/rust/crates/v2m-core/src/midi_export.rs:410`.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_midi_export_core.py`: pass
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml midi_export`: pass
- `uv run python -c '... _save_midi(..., tempo=0)'`: raised legacy `ZeroDivisionError` from `mido.bpm2tempo`, confirming Rust `InvalidTempo` is a protective structured mapping.
- `uv run python -c '... _save_midi(..., tempo=-1)'`: raised legacy `ValueError: attribute must be in range 0..16777215`, confirming Rust `TempoOutOfRange` maps the Mido set-tempo range failure.
- `uv run python -c '... _save_midi(..., tempo=1)'`: raised the same Mido range `ValueError`, confirming low positive BPM also exceeds the 24-bit tempo meta-message range.

## Residual Risk

The review did not run full workspace tests or inspect every Mido writer path. `TrackTooLarge` is a defensive Rust guard around Standard MIDI File chunk length; creating a real parity fixture for a multi-gigabyte track is not practical in this stage. If Rust later owns file writing or user-facing export errors, the bridge must add caller-readable error messages and map `skipped_invalid_notes` to the legacy warning.

## Promotion Note

This role does not block the current reimplemented, fixture-bound unit. The unit still needs the other required review roles before the coordinator marks it verified, and the low follow-up should be closed before runtime promotion.
