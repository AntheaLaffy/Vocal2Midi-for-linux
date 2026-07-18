# 0090 - Fix HFA HTK Current-Directory Paths

Date: 2026-07-18

## Context

The first behavior rerun for `hfa_htk_label_export_core` passed the record 0089
fixes for huge finite timestamps and empty-name `wav_path` errors, but found a
remaining path-planning parity gap. Python `pathlib.Path` drops lexical current
directory segments when rendering paths, so `output_folder="."`,
`output_folder="./"`, and wav-parent mode with `wav_path="./song.wav"` produce
`HTK/Phones/song.lab` rather than `./HTK/Phones/song.lab`.

The unit remains `reimplemented`; this record documents the writer fix before
another behavior rerun.

## Fix

The Rust HTK planner now applies a narrow `pathlib`-style lexical projection to
planned roots by removing `.` components while preserving absolute roots and
`..` components. The same projection is used for empty-name path error display,
so `Path("./")` projects like Python's `PosixPath('.')` surface.

No filesystem route changed. Python `Exporter.save_htk` remains the runtime
owner.

## Fixtures

`rewrite-in-rust/fixtures/hfa_htk_label_export_core.jsonl` was expanded from 14
to 17 Python-generated rows. Added coverage locks:

- `output_folder="."`;
- `output_folder="./"`;
- wav-parent mode with `wav_path="./song.wav"`.

## Verification

Focused evidence:

```bash
uv run python rewrite-in-rust/bootstrap/check_hfa_htk_label_export_core.py
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_htk_label_export_core
```

The Python checker validates all 17 cases, and the Rust fixture parity test
matches the expanded projection.

## Review State

The behavior rerun report remains a durable failed audit:

- `rewrite-in-rust/reviews/2026-07-18-hfa_htk_label_export_core-behavior_reviewer-rerun.md`

A second behavior rerun is required before the coordinator may mark the unit
`verified`. Dependency/bootstrap, data/algorithm, and error/tracing reruns
passed after record 0089 unless a later rerun identifies a changed scope.

## Reversal

Rollback remains keeping Python `Exporter.save_htk` as runtime owner. No
production caller route changed.
