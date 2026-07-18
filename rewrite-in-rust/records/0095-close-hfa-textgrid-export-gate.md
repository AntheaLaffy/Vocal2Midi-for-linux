# 0095 - Close HFA TextGrid Export Gate

Date: 2026-07-18

## Decision

`hfa_textgrid_export_core` is verified.

The Rust implementation remains an independent `v2m-core::hfa_textgrid_export`
planned-file renderer for the selected `Exporter.save_textgrids` behavior.
Python `Exporter.save_textgrids` and textgrid 1.6.1 remain the runtime owners;
there is no production bridge, filesystem route change, artifact-copying route
change, or export-dispatch promotion in this gate.

## Evidence

Final fixture state:

- `rewrite-in-rust/fixtures/hfa_textgrid_export_core.jsonl` contains 18 Python
  3.12 planned-side-effect cases generated from the real
  `Exporter.save_textgrids` with monkeypatched `mkdir` and `codecs.open` sinks.

Final required review reports:

- `rewrite-in-rust/reviews/2026-07-18-hfa_textgrid_export_core-dependency_bootstrap_reviewer-rerun.md`
- `rewrite-in-rust/reviews/2026-07-18-hfa_textgrid_export_core-behavior_reviewer-rerun.md`
- `rewrite-in-rust/reviews/2026-07-18-hfa_textgrid_export_core-error_tracing_reviewer-rerun.md`

The failed initial behavior report remains durable audit evidence for the fix in
record 0094.

## Closed Findings

The final implementation and fixtures close the behavior-review finding that
POSIX paths with exactly two leading slashes were collapsed by Rust path
normalization. The planner now preserves Python `pathlib` path text for:

- `output_folder="//out"`;
- wav-parent mode with `wav_path="//a/b/song.wav"`;
- wav-parent mode with `wav_path="//song.wav"`.

The fixture set also covers exact TextGrid bytes, gap filling, interval
validation, sorted inserts, quote doubling, Python float text, path suffix edge
names, partial plans, repeated calls, and per-prediction isolation.

## Verification

Coordinator checks run before closeout:

```bash
uv run python rewrite-in-rust/bootstrap/check_hfa_textgrid_export_core.py
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_textgrid_export_core -- --nocapture
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml
cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check
cargo clippy --manifest-path rewrite-in-rust/rust/Cargo.toml --all-targets --all-features -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --manifest-path rewrite-in-rust/rust/Cargo.toml --no-deps
uv run python scripts/audit_vendored_sources.py
git diff --check
```

All passed.

## Reversal

Rollback remains keeping Python `Exporter.save_textgrids` and textgrid 1.6.1 as
runtime owners. Because no production route changed, reversal is removing the
independent Rust module, checker, fixture, and manifest verification entries if
this seam is later re-cut.
