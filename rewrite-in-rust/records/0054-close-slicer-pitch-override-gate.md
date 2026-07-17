# 0054 - Close Slicer Pitch Override Gate

Date: 2026-07-17

## Context

`slicer_pitch_override_core` was confirmed by record 0053 as a fixture-bound
unit for smart slicing when a caller supplies a voiced-mask override. The unit
keeps `librosa.pyin`, RMVPE/model execution, real process scheduling, audio IO,
GUI/Web/CLI routing, and production bridge wiring legacy-owned.

The unit now has:

- dependency/bootstrap review:
  `rewrite-in-rust/reviews/2026-07-17-slicer_pitch_override_core-dependency_bootstrap_reviewer.md`
- behavior review:
  `rewrite-in-rust/reviews/2026-07-17-slicer_pitch_override_core-behavior_reviewer.md`
- behavior follow-up review:
  `rewrite-in-rust/reviews/2026-07-17-slicer_pitch_override_core-behavior_reviewer-rerun2.md`
- data/algorithm review:
  `rewrite-in-rust/reviews/2026-07-17-slicer_pitch_override_core-data_algorithm_reviewer.md`

The initial behavior review returned `pass-with-followups`. The first rerun
found a short/no-override ordering mismatch and an incomplete voiced-frame clamp
fixture. The coordinator fixed both issues, and rerun2 returned `pass`.

## Decision

Accept `slicer_pitch_override_core` as verified.

The verified Rust unit preserves:

- supplied voiced-mask round/clip indexing;
- Python's short-segment early return before pitch lookup;
- split-window voiced and RMS frame clamping;
- first longest-unvoiced midpoint cut selection;
- first-minimum RMS fallback when no unvoiced cut exists;
- split-wrapper parent offset adjustment;
- final offset sorting;
- no-long short merge and tiny merge handoff behavior.

## Verification

```bash
uv run python rewrite-in-rust/bootstrap/check_slicer_pitch_override_core.py
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml slicer_pitch
```

Broader checks also passed during coordinator review:

```bash
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml
cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check
cargo clippy --manifest-path rewrite-in-rust/rust/Cargo.toml --all-targets --all-features -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --manifest-path rewrite-in-rust/rust/Cargo.toml --no-deps
git diff --check
```

## Residual Risk

The verified proof is fixture-bound and does not promote Rust into production.
Runtime promotion must still define external waveform payload validation,
dependency wiring, logging text, and Python-facing error mapping. The pyin and
RMVPE/model paths remain out of scope.

## Reversal

Rollback remains keeping `inference.API.slicer_api.pitch_based_slice` and
`inference.API.slicer_api._pitch_based_split` as runtime owners. No production
bridge was introduced.
