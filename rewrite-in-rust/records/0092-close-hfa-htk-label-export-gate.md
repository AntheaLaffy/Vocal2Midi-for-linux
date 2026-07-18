# 0092 - Close HFA HTK Label Export Gate

Date: 2026-07-18

## Decision

`hfa_htk_label_export_core` is verified.

The Rust implementation remains an independent `v2m-core::hfa_htk_export`
planned-file renderer. Python `Exporter.save_htk` remains the runtime owner;
there is no production bridge, filesystem route change, or export-dispatch
promotion in this gate.

## Evidence

Final fixture state:

- `rewrite-in-rust/fixtures/hfa_htk_label_export_core.jsonl` contains 20
  Python 3.12 planned-side-effect cases generated from the real
  `Exporter.save_htk` with monkeypatched `mkdir` and `open` sinks.

Final required review reports:

- `rewrite-in-rust/reviews/2026-07-18-hfa_htk_label_export_core-dependency_bootstrap_reviewer-rerun3.md`
- `rewrite-in-rust/reviews/2026-07-18-hfa_htk_label_export_core-behavior_reviewer-rerun3.md`
- `rewrite-in-rust/reviews/2026-07-18-hfa_htk_label_export_core-data_algorithm_reviewer-rerun2.md`
- `rewrite-in-rust/reviews/2026-07-18-hfa_htk_label_export_core-error_tracing_reviewer-rerun3.md`

The failed intermediate reports remain durable audit evidence for the fixes in
records 0089, 0090, and 0091.

## Closed Findings

The final implementation and fixtures close these review findings:

- finite HTK timestamps beyond `i128` now render with Python-compatible
  arbitrary-size integer text;
- empty-name `wav_path` inputs now raise Python-compatible `ValueError` before
  side effects;
- current-directory roots such as `.`, `./`, and `./song.wav` now project path
  text like `pathlib`;
- HTK label basenames now follow `Path.with_suffix(".lab").name` for
  trailing-dot, parent-directory, all-dot, hidden, and multi-suffix names.

## Verification

Coordinator checks run before closeout:

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

All passed.

## Reversal

Rollback remains keeping Python `Exporter.save_htk` as runtime owner. Because
no production route changed, reversal is removing the independent Rust module,
fixtures, and manifest verification entries if this seam is later re-cut.
