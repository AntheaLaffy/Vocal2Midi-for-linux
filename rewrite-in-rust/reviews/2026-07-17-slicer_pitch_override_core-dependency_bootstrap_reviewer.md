# slicer_pitch_override_core - dependency_bootstrap_reviewer

Date: 2026-07-17
Decision: pass

## Findings

No findings.

Dependency/bootstrap scope is covered for this gate: the manifest confirms
`slicer_pitch_override_core` as a reimplemented, confirmed-inventory unit for
supplied voiced-mask smart slicing, with explicit verification artifacts and an
explicit `librosa.pyin`/RMVPE/model-execution exclusion
(`rewrite-in-rust/manifest.yaml:1120`, `rewrite-in-rust/manifest.yaml:1128`,
`rewrite-in-rust/manifest.yaml:1133`, `rewrite-in-rust/manifest.yaml:1141`).
The dependency record confirms a library seam with no bridge dependencies,
confirmed inventory impact, hand-written fixture-bound replacement rationale,
and kept-legacy decisions for `librosa.pyin`, RMVPE/model execution, default
Slicer internals, RMS-dB internals, merge internals, ProcessPoolExecutor
mechanics, audio IO, GUI/Web/CLI, and production routing
(`rewrite-in-rust/dependencies/slicer_pitch_override_core.yaml:16`,
`rewrite-in-rust/dependencies/slicer_pitch_override_core.yaml:23`,
`rewrite-in-rust/dependencies/slicer_pitch_override_core.yaml:31`).

The seam choice is appropriate for this stage. The bootstrap record keeps the
Rust surface as an independent `v2m-core::slicer_pitch` library module, keeps
legacy Python as runtime owner, and introduces no production bridge
(`rewrite-in-rust/bootstrap/slicer_pitch_override_core.md:54`,
`rewrite-in-rust/bootstrap/slicer_pitch_override_core.md:64`). The Rust module
also documents that Python remains runtime owner for `librosa.pyin`, RMVPE/model
execution, audio IO, multiprocessing, GUI/Web callers, and production routing
(`rewrite-in-rust/rust/crates/v2m-core/src/slicer_pitch.rs:1`).

The fixture boundary matches the dependency decision. Legacy Python bypasses
`librosa.pyin` when a supplied voiced-mask override and positive step are
present (`inference/API/slicer_api.py:206`), while the fallback `pyin` path
remains separate (`inference/API/slicer_api.py:216`). The checker exercises only
supplied-mask override indexing, split policy, RMS fallback through an injected
provider, and outer policy orchestration; it replaces `Slicer`,
`_pitch_based_split`, and `ProcessPoolExecutor` for the orchestration cases
(`rewrite-in-rust/bootstrap/check_slicer_pitch_override_core.py:75`,
`rewrite-in-rust/bootstrap/check_slicer_pitch_override_core.py:87`,
`rewrite-in-rust/bootstrap/check_slicer_pitch_override_core.py:125`). The Rust
test consumes the same fixture table and returns `PyinUnsupported` when the
fallback path would otherwise be needed
(`rewrite-in-rust/rust/crates/v2m-core/src/slicer_pitch.rs:190`,
`rewrite-in-rust/rust/crates/v2m-core/src/slicer_pitch.rs:463`,
`rewrite-in-rust/rust/crates/v2m-core/src/slicer_pitch.rs:618`).

## Boundary Decision

The manifest unit boundary is confirmed. It should not be split, merged,
deferred, or replaced for this dependency/bootstrap gate. The selected unit is
the supplied-voiced-mask smart-slicing policy only; `librosa.pyin`, RMVPE/model
execution, real ProcessPoolExecutor scheduling, audio IO, and production routing
remain legacy-owned.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_slicer_pitch_override_core.py`: passed.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml slicer_pitch`: passed; 3 slicer pitch tests passed, 0 failed.
- `cargo tree --manifest-path rewrite-in-rust/rust/Cargo.toml -p v2m-core`: passed; no pyin/RMVPE/model/runtime/network/bridge crate dependency appeared.
- `rg -n "pyin|RMVPE|rmvpe|ProcessPoolExecutor|multiprocessing|librosa|onnx|model|bridge|pyo3|tokio|reqwest" ...`: inspected; matches are legacy source references, explicit exclusions, checker fakes, or Rust `PyinUnsupported` handling.
- `git diff --check`: passed.
- `cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check`: passed.
- Current diff/unit additions inspected with `git diff -- rewrite-in-rust/manifest.yaml rewrite-in-rust/rust/crates/v2m-core/src/lib.rs` and `git ls-files --others --exclude-standard | rg 'slicer_pitch_override_core|slicer_pitch.rs|0053'`.

## Residual Risk

This review did not perform the required behavior or data/algorithm review
roles. Multi-case parity is limited to the current fixture table, and real
`librosa.pyin`, RMVPE/model execution, multiprocessing scheduling, production
bridge payload validation, logging text, and Python-facing error mapping remain
unproven by design.

## Promotion Note

This dependency/bootstrap role does not block the unit. The coordinator should
not mark the unit verified until the remaining required review roles pass.
