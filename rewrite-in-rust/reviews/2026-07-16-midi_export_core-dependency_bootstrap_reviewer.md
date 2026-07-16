# midi_export_core - dependency_bootstrap_reviewer

Date: 2026-07-16
Decision: pass

## Boundary Decision

Confirmed. `midi_export_core` is a valid fixture-bound unit for deterministic
MIDI event planning and exact Standard MIDI File bytes produced by
`inference/io/note_io.py::_save_midi`. The split is appropriate: production
filesystem writes, status printing, runtime export routing, broad Mido APIs,
TXT/CSV, USTX, TextGrid, model inference, and quantization dispatch remain
legacy-owned.

## Findings

No findings.

## Reviewed Evidence

- Manifest boundary: `rewrite-in-rust/manifest.yaml:865` keeps Python as current
  owner, confirms inventory, names this dependency review, and records the
  fixture/checker paths and rollback route.
- Legacy source: `inference/io/note_io.py:19` filters finite notes and
  `inference/io/note_io.py:57` implements the selected `_save_midi` behavior:
  tempo meta event, onset/offset tick rounding, overlap clamping, minimum
  one-tick duration, lyric placement, pitch clamp, parent directory creation,
  Mido save, and print output.
- Dependency/bootstrap artifacts:
  `rewrite-in-rust/dependencies/midi_export_core.yaml:3` covers finite-note
  filtering, event planning, tick conversion, and SMF subset encoding;
  `rewrite-in-rust/bootstrap/midi_export_core.md:8` names the compatibility
  surface and `rewrite-in-rust/bootstrap/midi_export_core.md:73` keeps the seam
  as an independent Rust library with no bridge dependencies.
- Boundary record: `rewrite-in-rust/records/0041-confirm-midi-export-core-boundary.md:21`
  confirms the hand-written subset and `rewrite-in-rust/records/0041-confirm-midi-export-core-boundary.md:27`
  lists kept-legacy capabilities with a rollback route at line 50.
- Mido source evidence is sufficient for a hand-written replacement:
  `third_party/sources/mido-1.3.3/mido/midifiles/units.py:27` for
  `bpm2tempo`, `third_party/sources/mido-1.3.3/mido/midifiles/meta.py:105`
  for variable-length integers, `third_party/sources/mido-1.3.3/mido/midifiles/meta.py:237`
  and `third_party/sources/mido-1.3.3/mido/midifiles/meta.py:300` for lyric
  and tempo meta messages, `third_party/sources/mido-1.3.3/mido/midifiles/tracks.py:84`
  for automatic `end_of_track`, and
  `third_party/sources/mido-1.3.3/mido/midifiles/midifiles.py:238` plus
  `third_party/sources/mido-1.3.3/mido/midifiles/midifiles.py:462` for track
  and file writing.
- Fixture/checker evidence:
  `rewrite-in-rust/fixtures/midi_export_core.jsonl:1` through line 4 cover
  sorted notes, overlaps, lyrics, invalid skips, pitch clamp, minimum duration,
  tempo conversion, UTF-8 lyrics, and all-invalid input. The checker at
  `rewrite-in-rust/bootstrap/check_midi_export_core.py:74` writes legacy Mido
  output to a temp file and verifies inspected events, type, ticks per beat,
  exact MIDI hex, and skipped invalid-note count.
- Current diff review: the tracked diff contains the manifest update, unrelated
  `batch_cli_reslice_json` test diagnostics, and `v2m-core` module wiring.
  Untracked unit artifacts include the dependency/bootstrap/record/fixture,
  checker, and Rust `midi_export` module. No production Python caller or runtime
  bridge is introduced by this unit.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_midi_export_core.py`: pass.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml midi_export`:
  pass, 3 tests.
- `uv run python scripts/audit_vendored_sources.py`: pass, source audit
  reported 135 Python packages, 41 native-extension packages, 269 foreign
  runtime native binaries, and 0 `third_party` binary artifacts.
- `git diff --check`: pass.

## Residual Risk

This review only covers dependency/bootstrap quality. Behavior parity,
data/algorithm details, and error tracing remain separate required reviews for
this unit. The approved boundary intentionally does not prove arbitrary MIDI
parsing/writing, filesystem error mapping, runtime promotion, or broad Mido API
compatibility because those capabilities stay legacy-owned.

## Promotion Note

This role supports the unit boundary as `confirmed` and does not block
coordinator state update for dependency/bootstrap evidence. It does not mark
the unit verified; the remaining required review roles still need separate
reports.
