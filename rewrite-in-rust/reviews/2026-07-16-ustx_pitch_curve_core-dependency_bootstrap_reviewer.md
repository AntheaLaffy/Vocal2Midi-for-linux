# ustx_pitch_curve_core - dependency_bootstrap_reviewer

Date: 2026-07-16
Decision: pass-with-followups

## Findings

- Severity: low
- Location: rewrite-in-rust/bootstrap/ustx_pitch_curve_core.md:59
- Issue: The future Rust writer must preserve the narrow synthetic-sample seam and avoid re-expanding the unit into RMVPE preprocessing or ndarray parity.
- Evidence: The confirmed seam is plain note rows plus `time_step_seconds` and already-computed midi-pitch samples, while the legacy fixture harness materializes `RmvpeResult(time_step_seconds, midi_pitch, voiced_mask=None)` and calls only `_build_pitd_curve` (`rewrite-in-rust/bootstrap/check_ustx_pitch_curve_core.py:61`, `rewrite-in-rust/bootstrap/check_ustx_pitch_curve_core.py:66`). The broader RMVPE runtime owns model checks, ONNX sessions, librosa resampling, f0-to-midi interpolation, and `voiced_mask` creation before `_build_pitd_curve` ever runs (`inference/API/rmvpe_api.py:35`, `inference/API/rmvpe_api.py:77`, `inference/API/rmvpe_api.py:89`, `inference/API/rmvpe_api.py:96`, `inference/API/rmvpe_api.py:97`, `inference/API/rmvpe_api.py:122`). This is a writer-stage constraint, not a bootstrap blocker.
- Required fix: In the writer pass, keep the Rust API over already-computed midi-pitch samples and add behavior/data reviews before promotion.

- Severity: low
- Location: rewrite-in-rust/fixtures/ustx_pitch_curve_core.jsonl:3
- Issue: Fixture coverage is sufficient for dependency/bootstrap confirmation, but behavior review should still prove numeric parity for the future Rust implementation.
- Evidence: The fixture table covers empty inputs, NaN skipping, edge trimming, cents clipping, duplicate tick replacement, gap interpolation, smoothing, and multi-note flush (`rewrite-in-rust/fixtures/ustx_pitch_curve_core.jsonl:1`). The bootstrap record names the same behavior list and explicitly excludes model loading, ONNX Runtime, waveform preprocessing, f0-to-midi interpolation, `voiced_mask`, USTX YAML assembly, and filesystem writes (`rewrite-in-rust/bootstrap/ustx_pitch_curve_core.md:11`, `rewrite-in-rust/bootstrap/ustx_pitch_curve_core.md:27`). The dependency record confirms only local list/scalar transforms are in scope (`rewrite-in-rust/dependencies/ustx_pitch_curve_core.yaml:8`, `rewrite-in-rust/dependencies/ustx_pitch_curve_core.yaml:12`).
- Required fix: Run the required behavior and data/algorithm reviews after the Rust module exists.

## Boundary Decision

Manifest unit boundary: confirmed. It should not be split, merged, deferred, or replaced for dependency reasons.

It is valid to keep RMVPE model loading, ONNX Runtime provider/session execution, librosa/waveform preprocessing, f0-to-midi interpolation, `voiced_mask` calculation, USTX YAML assembly, and filesystem writes legacy-owned while moving only deterministic `_build_pitd_curve` behavior over notes plus synthetic `RmvpeResult(time_step_seconds, midi_pitch)` into future Rust.

The source boundary supports that split: `_build_pitd_curve` consumes only sorted notes, `rmvpe.midi_pitch`, `rmvpe.time_step_seconds`, tempo, local tick snapping, cents clipping, duplicate replacement, gap fill, median filtering, and adaptive smoothing (`inference/API/ustx_api.py:104`, `inference/API/ustx_api.py:114`, `inference/API/ustx_api.py:128`, `inference/API/ustx_api.py:137`, `inference/API/ustx_api.py:139`, `inference/API/ustx_api.py:146`). USTX curve insertion, YAML assembly, directory creation, and file writes remain in `save_ustx` (`inference/API/ustx_api.py:407`, `inference/API/ustx_api.py:413`, `inference/API/ustx_api.py:458`), and the separate project-export unit is already confirmed/verified for `rmvpe_result=None` (`rewrite-in-rust/manifest.yaml:894`, `rewrite-in-rust/manifest.yaml:910`).

Dependency evidence is adequate. The Python project declares NumPy, PyYAML, librosa, ONNX Runtime, SciPy, and soundfile, but this seam needs only scalar/list-level NumPy behavior and a local `RmvpeResult` shape (`pyproject.toml:13`, `pyproject.toml:17`, `pyproject.toml:18`, `pyproject.toml:27`, `pyproject.toml:30`, `pyproject.toml:32`). Vendored evidence includes NumPy and upstream ONNX Runtime sources, while the dependency record correctly avoids pulling those runtime stacks into this unit (`third_party/sources/manifest.json:475`, `third_party/sources/manifest.json:480`, `rewrite-in-rust/dependencies/ustx_pitch_curve_core.yaml:36`, `rewrite-in-rust/dependencies/ustx_pitch_curve_core.yaml:39`).

Writer/reviewer separation is preserved. This review did not edit production code, fixtures, dependency records, bootstrap docs, records, or manifest.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_ustx_pitch_curve_core.py`: pass
- `uv run python -c "import json, pathlib, yaml; yaml.safe_load(...); list(json.loads(...)); print('ok')"`: pass
- `git diff --check -- rewrite-in-rust/dependencies/ustx_pitch_curve_core.yaml rewrite-in-rust/bootstrap/ustx_pitch_curve_core.md rewrite-in-rust/fixtures/ustx_pitch_curve_core.jsonl rewrite-in-rust/bootstrap/check_ustx_pitch_curve_core.py rewrite-in-rust/records/0043-confirm-ustx-pitch-curve-boundary.md rewrite-in-rust/manifest.yaml`: pass
- `rg -n "ustx_pitch_curve_core|RMVPE|rmvpe|_build_pitd_curve|pitd" ...`: inspected manifest, records, dependency/bootstrap files, fixtures, legacy API files, and dependency evidence

## Residual Risk

This review confirms the dependency/bootstrap boundary only. It does not prove future Rust numeric parity, error policy for non-finite tempo or impossible tick conversion, or promotion integration into `save_ustx`.

## Promotion Note

This dependency/bootstrap role does not block writer work. The coordinator should still require the unit's behavior and data/algorithm reviews before marking `ustx_pitch_curve_core` verified.
