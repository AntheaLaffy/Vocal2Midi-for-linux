# note_text_csv_export_core - behavior_reviewer

Date: 2026-07-15
Decision: pass

## Findings

No findings.

Evidence:

- Scope, owner, and rollback are correctly bounded: `rewrite-in-rust/manifest.yaml:133` marks only `note_text_csv_export_core`, `rewrite-in-rust/manifest.yaml:135` keeps it `reimplemented`, `rewrite-in-rust/manifest.yaml:137` keeps legacy Python as current owner, and `rewrite-in-rust/manifest.yaml:149` rolls back to `inference.io.note_io`.
- The bootstrap surface is limited to TXT/CSV row rendering, filtering, row order, three-decimal numeric formatting, Librosa-shaped pitch names, lyric-column selection, and CSV/TXT exact output behavior at `rewrite-in-rust/bootstrap/note_text_csv_export_core.md:5` and `rewrite-in-rust/bootstrap/note_text_csv_export_core.md:8`.
- Legacy Python filters finite `onset`, `offset`, and `pitch`, skips `offset <= onset`, and preserves surviving input order in `_finite_notes` at `inference/io/note_io.py:19`. Rust mirrors this in input order at `rewrite-in-rust/rust/crates/v2m-core/src/export.rs:43` and `rewrite-in-rust/rust/crates/v2m-core/src/export.rs:84`.
- Legacy Python formats onset/offset/pitch, applies optional half-even `round`, clips only pitch-name output, calls `librosa.midi_to_note(..., unicode=False, cents=not round_pitch)`, and chooses the lyric column from valid notes at `inference/io/note_io.py:103`. Rust mirrors those decisions at `rewrite-in-rust/rust/crates/v2m-core/src/export.rs:54`, `rewrite-in-rust/rust/crates/v2m-core/src/export.rs:91`, and `rewrite-in-rust/rust/crates/v2m-core/src/export.rs:160`.
- Legacy TXT and CSV output paths write tab-separated TXT rows, `csv.DictWriter` header order, lyric-column inclusion only when any valid lyric is non-empty, and CRLF CSV records at `inference/io/note_io.py:121`. Rust mirrors TXT and CSV rendering at `rewrite-in-rust/rust/crates/v2m-core/src/export.rs:104` and `rewrite-in-rust/rust/crates/v2m-core/src/export.rs:119`.
- The durable fixture table covers TXT with lyrics and blank lyric cells, TXT without lyrics, CSV invalid-note filtering, CSV header/order/CRLF, quote escaping, clamping, cents, half-even rounded pitch names, and numeric pitch output at `rewrite-in-rust/fixtures/note_text_csv_export_core.tsv:2`. Both Python and Rust harnesses consume this same table at `rewrite-in-rust/bootstrap/check_note_text_csv_export_core.py:88` and `rewrite-in-rust/rust/crates/v2m-core/src/export.rs:316`.
- The hand-written pitch-name replacement is tied to the right reference: dependency evidence names Librosa and NumPy behavior at `rewrite-in-rust/dependencies/note_text_csv_export_core.yaml:12`, and the vendored Librosa reference uses `np.round`, `np.around`, the `C:maj` ASCII note map, octave output, and optional cents at `third_party/sources/librosa-0.11.0/librosa/core/convert.py:983`.

## Checks

- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml export`: passed; 3 export tests passed.
- `uv run python rewrite-in-rust/bootstrap/check_note_text_csv_export_core.py`: passed; all fixture outputs matched legacy Python, with the expected skipped-note warning for the invalid-note fixture.
- Targeted Librosa/Rust cent-rounding probes for fractional MIDI literals around `69.005..69.125`: passed; Rust helper behavior matched Librosa's binary-float cent rounding for the checked edge literals.

## Residual Risk

This review covers only the pre-promotion library seam. It does not approve a Python/Rust bridge, filesystem-write ownership, warning-message mapping, or string-to-enum option parsing; the bootstrap explicitly keeps bridge and file-write behavior out of this unit at `rewrite-in-rust/bootstrap/note_text_csv_export_core.md:68` and `rewrite-in-rust/bootstrap/note_text_csv_export_core.md:110`.

The fixture table is representative rather than exhaustive over all finite float formatting ties and all possible lyric strings. The highest-risk Librosa cent-rounding edges were spot-checked separately, and the source paths for filtering, row ordering, lyric selection, TXT output, and CSV output are structurally equivalent.

## Promotion Note

This behavior role does not block moving `note_text_csv_export_core` from `reimplemented` to `verified`. It does not approve runtime ownership promotion; production ownership remains with legacy Python until a later promotion record verifies a bridge and rollback path.
