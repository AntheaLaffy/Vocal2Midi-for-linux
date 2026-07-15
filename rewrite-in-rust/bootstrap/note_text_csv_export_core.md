# note_text_csv_export_core Bootstrap

## Boundary

`note_text_csv_export_core` covers only TXT/CSV note row rendering from
`inference/io/note_io.py::_save_text` and the local helper behavior it calls.

The public compatibility surface is:

- invalid note filtering by finite `onset`, `offset`, and `pitch`
- invalid note filtering when `offset <= onset`
- input row ordering for valid notes
- onset and offset formatting with three decimals
- numeric pitch formatting with three decimals
- pitch-name formatting through the selected `librosa.midi_to_note` call shape
- optional pre-rounding by Python half-even `round`
- lyric column inclusion when any valid note has a non-empty lyric
- TXT tab-separated rows and CSV header/escaping/newline behavior

MIDI, USTX, TextGrid, `pad_1d_arrays`, directory creation, print messages, and
production export routing stay legacy-owned.

## Dependency Expansion

`inference/io/note_io.py` imports:

- stdlib: `csv`, `pathlib`, `dataclasses.dataclass`, `typing.Literal`
- third party: `librosa`, `mido`, `numpy`

The selected `_save_text` path uses:

- `_finite_notes` for scalar finite checks and `offset > onset`
- `numpy.clip` and scalar finite behavior
- `librosa.midi_to_note(..., unicode=False, cents=not round_pitch)`
- `csv.DictWriter` for CSV output

It does not use `mido`, MIDI tick/event behavior, TextGrid output, ONNX Runtime,
model inference, PyQt, Flask, or any native audio processing path.

Dependency evidence:

- `pyproject.toml` and `requirements.txt` include `librosa`, `mido`, and
  `numpy<2.0.0`.
- `uv.lock` records `librosa==0.11.0`, `mido==1.3.3`, and `numpy==1.26.4`.
- `third_party/sources/manifest.json` records source directories:
  `third_party/sources/librosa-0.11.0`,
  `third_party/sources/mido-1.3.3`, and
  `third_party/sources/numpy-1.26.4`.
- `third_party/native_sources/manifest.json` records OpenBLAS native coverage
  for NumPy/SciPy, but this unit does not need BLAS or array kernels.
- `third_party/source_audit.json` reports all foreign runtime native binaries
  covered and zero `third_party` binary artifacts.

`librosa.core.convert.midi_to_note` is the only nontrivial formatting reference
needed here. For this call site, the Rust implementation should cover the
default `C:maj` ASCII note map, octave output, optional cent suffixes, and
NumPy/Python half-even rounding behavior.

## Seam

Use the default rewrite seam:

- kind: independent Rust library
- crate: `rewrite-in-rust/rust/crates/v2m-core`
- runtime owner: legacy Python
- bridge dependencies: none

Do not add PyO3, subprocess, CLI, HTTP, runtime-router, NumPy, Librosa, Mido, or
filesystem-write bridge code for this unit.

## Fixture Harness

Rust tests should consume the durable parity table at:

```text
rewrite-in-rust/fixtures/note_text_csv_export_core.tsv
```

The fixtures must cover:

- TXT with lyrics and blank lyric cells
- TXT without lyrics
- CSV with lyrics and invalid skipped notes
- CSV quote escaping and CRLF row terminators
- pitch names with cents and 0..127 clipping
- rounded half-even pitch names
- numeric pitch formatting

The legacy Python side of the table is checked by:

```bash
uv run python rewrite-in-rust/bootstrap/check_note_text_csv_export_core.py
```

## Repeated-Call Behavior

The formatter is deterministic for a fixed input note list and option tuple.
Repeated calls must return the same rendered content and skipped-note count, and
must not depend on filesystem state, model state, GUI state, Web state, or
global audio runtime state.

## Rollback

Rollback is keeping all production imports unchanged:

```text
inference.io.note_io._save_text
```

No production Python caller should import Rust output until a later promotion
record chooses and verifies a bridge, including filesystem-write and warning
message mapping.
