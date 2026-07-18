# hfa_textgrid_export_core Bootstrap

## Boundary

Cover `Exporter.save_textgrids` as an in-memory path/byte planner. Preserve two
ordered IntervalTiers (`words`, then `phones`), phone `max(0, start)`, strict
interval validation/order, blank-gap insertion, quote doubling, Python float
text, UTF-8 long TextGrid bytes, path selection, and per-prediction isolation.
Filesystem effects and API artifact copying remain Python-owned.

## Dependency

The compatibility source is `textgrid==1.6.1` from `uv.lock`, vendored at
`third_party/sources/textgrid-1.6.1/textgrid/textgrid.py`. Hand-write only the
selected `TextGrid`/`IntervalTier` writer subset; do not add a general TextGrid
crate or port read, PointTier, or MLF APIs. Reuse verified HFA Word state.

## Fixture Contract

`rewrite-in-rust/fixtures/hfa_textgrid_export_core.jsonl` contains 18
Python-generated cases comparing exact UTF-8 bytes and ordered planned paths.
Coverage includes empty, contiguous, gapped, and sorted tiers; negative phone
starts; quotes/newlines/Unicode; Python float text; zero, overlapping, and
out-of-range interval errors; partial plans after prior writes; nested and
duplicate basenames; output-folder, wav-parent, empty-output, and current-dir
modes; POSIX exactly-two-leading-slash roots; pathlib suffix edge names; empty
wav-path errors; multiple predictions; and repeated calls. Expected bytes come
from installed/vendored textgrid 1.6.1.

## Rollback

Keep `Exporter.save_textgrids` and textgrid 1.6.1 as runtime owners.
