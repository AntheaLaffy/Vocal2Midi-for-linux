# 0091 - Fix HFA HTK Pathlib Suffix Names

Date: 2026-07-18

## Context

The second behavior rerun for `hfa_htk_label_export_core` passed the record
0089 and 0090 fixes, but found that Rust still used `PathBuf::set_extension`
semantics for HTK label basenames. Python uses
`Path.with_suffix(".lab").name`, which appends `.lab` when the final `.` is the
first or last character and treats a parent-directory component as the literal
name `..`.

The unit remains `reimplemented`; this record documents the writer fix before
another behavior rerun.

## Fix

The Rust label-name projection now mirrors Python 3.12 `pathlib`:

- empty names such as `""`, `"."`, and `"./"` still raise
  `ValueError: PosixPath('.') has an empty name`;
- `..` is accepted as the literal final name and becomes `...lab`;
- names with no Python suffix append `.lab`;
- names with a Python suffix replace only the last suffix.

This avoids Rust `PathBuf::set_extension` behavior that collapses trailing-dot
names such as `song.` to `song.lab`.

## Fixtures

`rewrite-in-rust/fixtures/hfa_htk_label_export_core.jsonl` was expanded from 17
to 20 Python-generated rows. Added coverage locks:

- `wav_path="song."` writes `song..lab`;
- `wav_path=".."` writes `...lab`;
- a compact suffix matrix for `song..`, `...`, `.hidden.`, `song.tar.gz`, and
  `..a`.

## Verification

Focused evidence:

```bash
uv run python rewrite-in-rust/bootstrap/check_hfa_htk_label_export_core.py
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_htk_label_export_core
```

The Python checker validates all 20 cases, and the Rust fixture parity test
matches the expanded projection.

## Review State

The second behavior rerun report remains a durable failed audit:

- `rewrite-in-rust/reviews/2026-07-18-hfa_htk_label_export_core-behavior_reviewer-rerun2.md`

Another behavior rerun is required before the coordinator may mark the unit
`verified`. Dependency/bootstrap, data/algorithm, and error/tracing reruns
passed before this path-name fix unless a later rerun identifies changed scope.

## Reversal

Rollback remains keeping Python `Exporter.save_htk` as runtime owner. No
production caller route changed.
