# 0093 - Implement HFA TextGrid Export Core

Date: 2026-07-18

## Context

`hfa_textgrid_export_core` was the first manifest unit after the verified HTK
label export gate. Dependency/bootstrap records already confirmed the boundary:
mirror `Exporter.save_textgrids` as an independent in-memory path/byte planner
for the pinned `textgrid==1.6.1` long TextGrid writer subset.

Python remains the runtime owner. No production export route, filesystem write,
API artifact-copying path, GUI, Web, or model-inference code changed.

## Implementation

Added `v2m-core::hfa_textgrid_export` with:

- prediction snapshots reusing verified HFA `Word`/`Phoneme` data;
- two ordered tiers, `words` then `phones`;
- phone `max(0, start)` behavior, including Python's integer `0` text when the
  clamp wins;
- textgrid 1.6.1 interval insertion, sorting, strict overlap rejection,
  out-of-range checks, blank-gap insertion, quote doubling, UTF-8 writer
  encoding, and Python float text;
- TextGrid path planning with Python `pathlib` current-directory normalization
  and `Path.with_suffix(".TextGrid").name` suffix-name behavior;
- Python-compatible errors with partial side-effect plans.

Added `rewrite-in-rust/bootstrap/check_hfa_textgrid_export_core.py`, which
monkeypatches `Path.mkdir` and `codecs.open` around the real
`Exporter.save_textgrids` to capture ordered side effects without writing files.

## Fixtures

`rewrite-in-rust/fixtures/hfa_textgrid_export_core.jsonl` contains 15
Python-generated cases covering:

- exact long TextGrid bytes for empty, gapped, contiguous, and sorted tiers;
- negative phone starts, quotes, newlines, Unicode, and float rendering;
- zero-duration, overlapping, out-of-range, and empty-path errors;
- partial plans after prior writes;
- per-prediction isolation and duplicate basename write order;
- output-folder, wav-parent, empty-output, current-directory, and pathlib suffix
  edge path behavior;
- repeated calls.

## Verification

Focused evidence:

```bash
uv run python rewrite-in-rust/bootstrap/check_hfa_textgrid_export_core.py
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_textgrid_export_core -- --nocapture
```

Both passed against the 15-case fixture set.

## Review State

The unit is `reimplemented`, not `verified`. Required independent review roles
remain:

- `dependency_bootstrap_reviewer`
- `stage_behavior_reviewer`
- `error_tracing_reviewer`

## Reversal

Rollback remains keeping Python `Exporter.save_textgrids` and textgrid 1.6.1 as
runtime owners. Because no production route changed, reversal is removing the
independent Rust module, fixture, checker, and manifest entries if the boundary
is later re-cut.
