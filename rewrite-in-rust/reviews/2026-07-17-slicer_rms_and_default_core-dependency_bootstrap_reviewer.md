# slicer_rms_and_default_core - dependency_bootstrap_reviewer

Date: 2026-07-17
Decision: pass-with-followups

## Findings

- Severity: low
- Location: rewrite-in-rust/bootstrap/check_slicer_rms_and_default_core.py:19
- Issue: The fixture harness does not directly exercise `inference.API.slicer_api.default_slice`, even though the confirmed boundary includes its caller parameters.
- Evidence: The boundary record includes `inference.API.slicer_api.default_slice` in scope at `rewrite-in-rust/records/0046-confirm-slicer-rms-default-boundary.md:16`, and the legacy caller fixes `threshold=-30.`, `min_length=5000`, and `max_sil_kept=500` at `inference/API/slicer_api.py:649`. The Python harness imports only `Slicer` and `get_rms` at `rewrite-in-rust/bootstrap/check_slicer_rms_and_default_core.py:19`, dispatches only `get_rms`, `constructor_state`, `constructor_error`, and `slice` cases at `rewrite-in-rust/bootstrap/check_slicer_rms_and_default_core.py:92`, and the JSONL fixture table has no `default_slice` case at `rewrite-in-rust/fixtures/slicer_rms_and_default_core.jsonl:1`. The Rust side has `Slicer::default_for_sample_rate` at `rewrite-in-rust/rust/crates/v2m-core/src/slicer_default.rs:182` and a hard-coded default-parameter test at `rewrite-in-rust/rust/crates/v2m-core/src/slicer_default.rs:584`, but this is not tied to a legacy Python `default_slice` fixture.
- Required fix: Before verification or promotion evidence is considered complete, add a small `default_slice` or default-caller fixture case that invokes `inference.API.slicer_api.default_slice` and proves the caller constants against the Rust `default_for_sample_rate` surface, or narrow the record to exclude the caller helper from this unit.

## Boundary Decision

The manifest unit boundary remains confirmed. It should not be split, merged, deferred, or replaced based on this dependency/bootstrap review.

The capability split is coherent: `slicer2.py` imports only NumPy and contains the local `get_rms` copy plus `Slicer` state machine at `inference/slicer/slicer2.py:1`, `inference/slicer/slicer2.py:5`, and `inference/slicer/slicer2.py:41`. Adjacent heavy behavior in `slicer_api.py` stays out of scope: `librosa.pyin` at `inference/API/slicer_api.py:216`, `librosa.feature.rms` at `inference/API/slicer_api.py:236`, frame/time conversion and grid/heuristic splitting at `inference/API/slicer_api.py:274` and `inference/API/slicer_api.py:310`, and `ProcessPoolExecutor` pitch splitting at `inference/API/slicer_api.py:620`. The manifest keeps separate planned units for heuristic/grid and supplied-voiced-mask smart slicing at `rewrite-in-rust/manifest.yaml:1032` and `rewrite-in-rust/manifest.yaml:1050`.

The seam choice is appropriate for this stage. The dependency record declares a library seam with legacy runtime owner and no bridge dependencies at `rewrite-in-rust/dependencies/slicer_rms_and_default_core.yaml:16`, and the bootstrap record keeps production imports unchanged until a promotion record exists at `rewrite-in-rust/bootstrap/slicer_rms_and_default_core.md:97`. The Rust module is exposed only inside the independent `v2m-core` crate at `rewrite-in-rust/rust/crates/v2m-core/src/lib.rs:21`.

The hand-written replacement choice is justified. The Rust implementation does not add ndarray, librosa, soundfile, or audio IO crates; the existing `v2m-core` dependencies are unchanged at `rewrite-in-rust/rust/crates/v2m-core/Cargo.toml:12`. Dependency evidence for the broader Python stack exists in `pyproject.toml:13`, `requirements.txt:8`, `uv.lock:710`, `third_party/sources/manifest.json:375`, and `third_party/native_sources/manifest.json:349`, while the selected unit only needs fixture-bound padding, framing, scalar conversion, silence-tag selection, and waveform slicing.

The fixture strategy is adequate for dependency/bootstrap confirmation aside from the caller-helper follow-up above. The fixture file covers RMS padding, constructor conversion/errors, short/no-silence behavior, leading/middle/trailing silence, middle-silence branch variants, and stereo slicing across `rewrite-in-rust/fixtures/slicer_rms_and_default_core.jsonl:1` through `rewrite-in-rust/fixtures/slicer_rms_and_default_core.jsonl:14`.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_slicer_rms_and_default_core.py`: passed with no output.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml slicer_default`: passed, 3 tests passed and 78 filtered out.
- `git diff --check`: passed with no output.

## Residual Risk

This review did not perform behavior or data/algorithm approval. Numeric edge cases outside the synthetic finite fixtures, future bridge payload validation, error mapping, and production ownership remain for the later required review roles and promotion planning.

## Promotion Note

This dependency/bootstrap role does not block keeping the unit boundary confirmed. The coordinator can use this gate as `pass-with-followups`, but should track the missing direct `default_slice` fixture before treating the unit's caller-parameter evidence as complete.
