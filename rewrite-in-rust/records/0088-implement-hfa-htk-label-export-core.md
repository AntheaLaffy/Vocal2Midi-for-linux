# 0088 - Implement HFA HTK Label Export Core

Date: 2026-07-18

## Context

`hfa_htk_label_export_core` follows the now-verified HFA config loader work.
The unit boundary was already confirmed by the dependency/bootstrap records: it
covers `Exporter.save_htk` as an in-memory planned-file renderer, while Python
keeps directory creation, file writes, status printing, dispatch, and production
routing.

The important legacy quirk is that `w_out` and `ph_out` are initialized before
the prediction loop, so later `.lab` writes contain all earlier prediction
labels. Conversion failures can also happen after earlier prediction files were
already written.

## Implementation

Added `v2m-core::hfa_htk_export`.

The Rust planner preserves:

- prediction order;
- `int(float(time) * 10000000)` truncation for finite values;
- Python `ValueError` and `OverflowError` messages for NaN and infinity;
- raw word/phoneme text, including quotes and newlines;
- output-folder versus wav-parent path selection;
- basename replacement with `.lab`;
- ordered `mkdir(parents=True, exist_ok=True)` and UTF-8 write plans;
- cross-prediction cumulative `w_out` and `ph_out` buffers;
- partial planned side effects on conversion errors.

The Python fixture harness monkeypatches `pathlib.Path.mkdir` and `open` around
the real `Exporter.save_htk` so expected rows capture ordered side effects
without writing production paths.

## Fixtures

Added:

- `rewrite-in-rust/bootstrap/check_hfa_htk_label_export_core.py`
- `rewrite-in-rust/fixtures/hfa_htk_label_export_core.jsonl`

The fixture matrix covers empty/single/multiple predictions, cumulative later
files, duplicate basenames, nested paths, output-folder, wav-parent and empty
output-folder modes, fractional/negative/special/large times, Unicode, quotes,
newlines, empty words/phones, partial plans on conversion errors, and repeated
exporter calls.

## Verification

Current writer evidence:

```bash
uv run python rewrite-in-rust/bootstrap/check_hfa_htk_label_export_core.py
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_htk_label_export_core
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml
cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check
cargo clippy --manifest-path rewrite-in-rust/rust/Cargo.toml --all-targets --all-features -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --manifest-path rewrite-in-rust/rust/Cargo.toml --no-deps
uv run python -m py_compile inference/HubertFA/tools/export_tool.py rewrite-in-rust/bootstrap/check_hfa_htk_label_export_core.py
uv run python scripts/audit_vendored_sources.py
git diff --check
```

The Python checker validates the HTK fixture matrix. The full Rust workspace passes 116
`v2m-core` tests plus five quant bridge tests. Independent
dependency/bootstrap, behavior, data/algorithm, and error/tracing reviews are
still required before this unit may be marked `verified`.

## Reversal

Rollback remains keeping Python `Exporter.save_htk` as runtime owner. No
production import, bridge, GUI, Web, CLI, export dispatch, or inference route
changed.
