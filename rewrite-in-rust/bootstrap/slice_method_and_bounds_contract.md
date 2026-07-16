# slice_method_and_bounds_contract Bootstrap

## Boundary

`slice_method_and_bounds_contract` covers the deterministic string and
optional-float contract around slicing method selection and custom duration
bounds.

The compatibility surface is:

- `None` slicing method defaults to `default`;
- canonical method names: `default`, `smart`, `heuristic`, `grid`;
- `auto` alias maps to `default`;
- Chinese labels map to the canonical method names;
- legacy mojibake candidates are repaired through GB18030/GBK encode plus UTF-8
  decode;
- lowercased aliases are accepted;
- keyword fallback accepts strings containing `smart`, `heuristic`, `grid`,
  `default`, `智能`, `启发式`, `网格`, `默认`, or `auto`;
- unsupported and empty methods raise the exact legacy `ValueError` message;
- CLI custom bounds preserve `--min-seconds` / `--max-seconds` messages;
- API custom bounds preserve `min_len_sec` / `max_len_sec` messages;
- NaN and infinity follow Python float comparison behavior.

The unit explicitly does not cover actual audio slicing, waveform arrays,
segment merging, RMS, default/heuristic/grid/smart slicing algorithms, RMVPE,
ASR, SoundFile writes, FFmpeg, argparse, or filesystem behavior.

## Dependency Expansion

`inference/API/slicer_api.py` imports heavy slicing dependencies:

- third party: `librosa`, `numpy`
- stdlib: `itertools`, `functools`, `concurrent.futures.ProcessPoolExecutor`
- local: `inference.slicer.slicer2.Slicer`

`scripts/slice_asr_cli.py` imports broader runtime dependencies:

- stdlib: `argparse`, `gc`, `hashlib`, `importlib`, `json`, `math`, `os`,
  `shutil`, `sys`, `tempfile`, `pathlib`, `typing`
- third party: `librosa`, `soundfile`
- local: ASR, RMVPE, slicer API, and device utils modules

The selected contract path uses only:

- string strip/lower/contains behavior
- Python encode/decode behavior for GB18030/GBK repair candidates
- `float(...)` coercion for custom bounds
- Python float comparisons for negative, zero, NaN, and infinity cases

Dependency evidence:

- `pyproject.toml` and requirements include `librosa`, `numpy`, `soundfile`,
  `scipy`, and ONNX packages because the owning modules perform real slicing
  and inference outside this unit.
- `uv.lock` records `librosa==0.11.0`, `numpy==1.26.4`, `soundfile==0.14.0`,
  `scipy==1.17.1`, and ONNX packages.
- `third_party/sources/manifest.json` records source directories for
  `librosa-0.11.0`, `numpy-1.26.4`, `soundfile-0.14.0`, and `scipy-1.17.1`.
- `third_party/sources/MISSING_SOURCES.md` records upstream source fallback for
  `onnxruntime-1.27.0`.
- `third_party/source_audit.json` reports all foreign runtime native binaries
  covered and zero `third_party` binary artifacts.

Those sources are dependency evidence for the modules, not implementation
requirements for this contract. Do not add Rust audio, ndarray, ONNX, or CLI
parser dependencies for this unit.

## Seam

Use the default rewrite seam:

- kind: independent Rust library
- crate: `rewrite-in-rust/rust/crates/v2m-core`
- suggested module: `slice_method`
- runtime owner: legacy Python
- bridge dependencies: none

The future Rust surface should expose:

- method normalization returning canonical method or exact unsupported-method
  error text;
- CLI bounds resolution returning optional `(min, max)` or CLI-specific error;
- API bounds resolution returning optional `(min, max)` or API-specific error.

It should not call Python, parse process argv, load audio, run slicing, write
files, or expose a runtime router.

## Fixture Harness

Python/Rust parity uses:

```text
rewrite-in-rust/fixtures/slice_method_and_bounds_contract.jsonl
```

The legacy Python side is checked by:

```bash
uv run python rewrite-in-rust/bootstrap/check_slice_method_and_bounds_contract.py
```

The future Rust side should be checked by:

```bash
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml slice_method
```

## Repeated-Call Behavior

The selected functions are deterministic for fixed inputs. They must not depend
on filesystem state, audio data, process global slicing function replacements,
model state, ASR/RMVPE caches, or CLI parser state.

## Rollback

Rollback is keeping production imports unchanged:

```text
inference.API.slicer_api.normalize_slicing_method
inference.API.slicer_api._resolve_custom_slice_bounds
scripts.slice_asr_cli.normalize_slicing_method
scripts.slice_asr_cli.resolve_slice_bounds
application.config.validate_slice_bounds
```

No production caller should import Rust contract helpers until a promotion
record defines the bridge and error mapping.
