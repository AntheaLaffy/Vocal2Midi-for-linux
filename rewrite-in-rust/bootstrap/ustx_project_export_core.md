# ustx_project_export_core Bootstrap

## Boundary

`ustx_project_export_core` covers deterministic USTX project YAML export from
`inference/API/ustx_api.py::save_ustx` when `rmvpe_result is None`.

The compatibility surface is:

- invalid note filtering by finite `onset`, `offset`, and `pitch`
- invalid note filtering when `offset <= onset`
- stable sorting by note onset after filtering
- seconds-to-ticks conversion with Python half-even `round(seconds * tempo * 8)`
- minimum note duration of `10` ticks
- MIDI tone rounding with Python half-even `round` and clamping to `0..127`
- fallback lyric `"a"` when `note.lyric` is empty
- UTF-8 lyric output through `yaml.safe_dump(..., allow_unicode=True)`
- fixed pitch data and vibrato defaults on every note
- fixed expression descriptors from `_default_expressions`
- project metadata: name, comment, output/cache dirs, USTX version, resolution,
  tempo, time signature, track metadata, empty wave parts, and one voice part
- voice-part duration as `max(480, max_end_tick)`
- empty `curves: []`

The unit explicitly does not cover `_build_pitd_curve`, `RmvpeResult`, RMVPE
model inference, pitch-curve smoothing, or runtime promotion wiring.

## Dependency Expansion

`inference/API/ustx_api.py` imports:

- stdlib: `dataclasses.dataclass`, `pathlib.Path`, `typing.Any`
- third party: `numpy`, `yaml`
- local: `inference.API.rmvpe_api.RmvpeResult`

The selected project export path uses:

- `_finite_notes` for scalar finite checks and `offset > onset`
- `_to_ticks` for Python half-even tick rounding
- `numpy.isfinite` and `numpy.clip` for scalar values only
- `_default_expressions` for a fixed descriptor map
- `yaml.safe_dump(project, allow_unicode=True, sort_keys=False)` for output
- `filepath.stem` for project and voice-part names
- `filepath.parent.mkdir` and `filepath.open` in the legacy runtime owner

Dependency evidence:

- `pyproject.toml` and `requirements.txt` include `numpy<2.0.0` and `PyYAML`.
- `uv.lock` records `numpy==1.26.4` and `pyyaml==6.0.3`.
- `third_party/sources/manifest.json` records source directories:
  `third_party/sources/numpy-1.26.4` and
  `third_party/sources/pyyaml-6.0.3`.
- `third_party/native_sources/manifest.json` records OpenBLAS source coverage
  for NumPy/SciPy, but this unit does not need BLAS or array kernels.
- `third_party/source_audit.json` reports all foreign runtime native binaries
  covered and zero `third_party` binary artifacts.

The PyYAML reference paths for a narrow renderer are:

- `third_party/sources/pyyaml-6.0.3/lib/yaml/__init__.py`
- `third_party/sources/pyyaml-6.0.3/lib/yaml/representer.py`
- `third_party/sources/pyyaml-6.0.3/lib/yaml/emitter.py`

Do not add broad PyYAML, NumPy, ONNX Runtime, RMVPE, PyO3, subprocess, or HTTP
bridge dependencies for this unit.

## Seam

Use the default rewrite seam:

- kind: independent Rust library
- crate: `rewrite-in-rust/rust/crates/v2m-core`
- module: `ustx_project`
- runtime owner: legacy Python
- bridge dependencies: none

The Rust surface should return in-memory YAML text and skipped invalid-note
count. It should not create directories, write files, print warning/status
lines, call Python, or expose a runtime router.

## Fixture Harness

Python/Rust parity should use:

```text
rewrite-in-rust/fixtures/ustx_project_export_core.jsonl
```

The fixtures should cover:

- empty input producing one voice part with `duration: 480`, no notes, and
  `curves: []`
- invalid notes skipped by finite checks and `offset <= onset`
- stable note sorting after filtering
- half-even tick conversion and minimum duration
- tone clamping at both bounds
- fallback lyric `"a"` and UTF-8 lyric preservation
- tempo values in root `bpm` and `tempos`
- output stem propagation into `name` and the voice part name

The legacy Python side should be checked by:

```bash
uv run python rewrite-in-rust/bootstrap/check_ustx_project_export_core.py
```

The Rust side should be checked by:

```bash
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml ustx_project
```

## Repeated-Call Behavior

The renderer is deterministic for a fixed note list, tempo, and output stem.
Repeated calls must return the same skipped count and YAML text. It must not
depend on filesystem state, model state, GUI state, Web state, global audio
runtime state, or RMVPE runtime state.

## Rollback

Rollback is keeping all production imports unchanged:

```text
inference.API.ustx_api.save_ustx
```

No production Python caller should import Rust USTX output until a later
promotion record chooses and verifies filesystem-write behavior, skipped-note
warning mapping, YAML compatibility, and optional pitch-curve handling.
