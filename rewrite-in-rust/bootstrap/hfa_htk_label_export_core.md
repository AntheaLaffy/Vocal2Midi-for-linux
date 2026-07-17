# hfa_htk_label_export_core Bootstrap

## Boundary

Cover `Exporter.save_htk` as an in-memory ordered planned-file renderer over
prediction snapshots. Preserve `int(float(time) * 10000000)`, raw text/newlines,
Phones/Words paths, basename replacement, output-folder versus wav-parent mode,
and prediction order. Directory creation and writes stay legacy-owned.

## Legacy Stateful Contract

`w_out` and `ph_out` are initialized before the prediction loop. A controlled
Python 3.12 probe confirmed that each later `.lab` file contains all earlier
prediction labels plus its own labels. This appears accidental but is observable
legacy behavior and must not be silently repaired. Duplicate basenames overwrite
in prediction order and must also be fixture-bound.

## Seam And Fixtures

- planned module: `v2m-core::hfa_htk_export`
- reuse verified HFA Word/Phoneme snapshots; no parallel interval model
- return ordered planned paths and exact UTF-8 bytes
- no bridge or new crate

Cover empty/single/multiple predictions, cumulative later files, nested and
duplicate basenames, output-folder modes, fractional/negative/special/large
times, Unicode/quote/newline text, empty phones, errors, and repeated exporter
calls.

## Rollback

Keep `Exporter.save_htk` as runtime owner, including cumulative buffers.
