# 0094 - Fix HFA TextGrid Double-Slash Paths

Date: 2026-07-18

## Context

The first behavior review for `hfa_textgrid_export_core` passed the main
TextGrid serialization fixture matrix, but found a path-planning mismatch for
POSIX paths with exactly two leading slashes. Python `pathlib` preserves the
exactly-two-slash root, while Rust `Path::components()` rebuilt it as a single
slash.

The unit remains `reimplemented`; this record documents the writer fix before
behavior review rerun.

## Fix

The TextGrid path projection now preserves Python's exactly-two-leading-slash
root behavior while still dropping lexical `.` components. Wav-parent mode also
handles `//song.wav`, where Rust `Path::parent()` would otherwise lose the
second root slash before normalization.

This is scoped to `hfa_textgrid_export_core`. No production caller route
changed.

## Fixtures

`rewrite-in-rust/fixtures/hfa_textgrid_export_core.jsonl` was expanded from 15
to 18 Python-generated rows. Added coverage locks:

- `output_folder="//out"`;
- wav-parent mode with `wav_path="//a/b/song.wav"`;
- wav-parent mode with `wav_path="//song.wav"`.

## Verification

Focused evidence:

```bash
uv run python rewrite-in-rust/bootstrap/check_hfa_textgrid_export_core.py
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_textgrid_export_core -- --nocapture
```

The Python checker validates all 18 cases, and the Rust fixture parity test
matches the expanded projection.

## Review State

The first behavior review remains a durable failed audit:

- `rewrite-in-rust/reviews/2026-07-18-hfa_textgrid_export_core-behavior_reviewer.md`

A behavior rerun is required before the coordinator may mark the unit
`verified`. Dependency/bootstrap and error/tracing reviews passed before this
path fix unless a rerun identifies changed scope.

## Reversal

Rollback remains keeping Python `Exporter.save_textgrids` and textgrid 1.6.1 as
runtime owners. No production caller route changed.
