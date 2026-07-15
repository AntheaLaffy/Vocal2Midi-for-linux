# runtime_device_normalization Bootstrap

## Boundary

`runtime_device_normalization` covers only
`inference/device_utils.py::normalize_runtime_device`.

The public compatibility surface is:

- default device selection: `dml` on Windows and `cpu` on non-Windows platforms
- `None`, empty string, and whitespace handling
- alias normalization for `cuda`, `directml`, `dml`, `gpu`, and `cpu`
- strip/lowercase behavior for unknown values
- explicit `default` behavior used by the Python function

`resolve_onnx_providers`, `use_dml`, DXGI adapter enumeration, ONNX Runtime
provider availability, and all model runtime ownership stay legacy-owned.

## Dependency Expansion

`inference/device_utils.py` imports:

- stdlib: `ctypes`, `platform`, `dataclasses.dataclass`, `functools.lru_cache`
- third party: `onnxruntime`

The selected function uses the module-level `_IS_WINDOWS` value and the local
`_DEVICE_ALIASES` table. It does not call `onnxruntime`, DXGI COM helpers, or
any model runtime code. Therefore the Rust unit should not add an ONNX Runtime,
DirectML, ctypes, or platform adapter discovery dependency.

Dependency evidence:

- `pyproject.toml` and `uv.lock` include `onnxruntime` for non-Windows and
  `onnxruntime-directml` for Windows runtime paths.
- `third_party/sources/MISSING_SOURCES.md` records `onnxruntime` source fallback
  under `third_party/upstream_sources/onnxruntime-1.27.0`.
- `third_party/source_audit.json` reports all foreign runtime native binaries
  covered and zero `third_party` binary artifacts.

Those sources matter for later ONNX/provider units. They are deliberately kept
out of this string-normalization unit.

## Seam

Use the default rewrite seam:

- kind: independent Rust library
- crate: `rewrite-in-rust/rust/crates/v2m-core`
- runtime owner: legacy Python
- bridge dependencies: none

Do not add PyO3, subprocess, CLI, HTTP, ONNX Runtime, DirectML, or runtime-router
code for this unit.

## Fixture Harness

Rust tests should consume the durable parity table at:

```text
rewrite-in-rust/fixtures/runtime_device_normalization.tsv
```

The fixtures must cover:

- non-Windows default cases: `None`, empty string, whitespace
- Windows default cases: `None`, empty string, whitespace
- aliases: `cuda`, `directml`, `dml`, `gpu`, `cpu`
- case and whitespace normalization
- unknown values returned after strip/lowercase
- explicit default behavior, including the Python edge case where an explicit
  empty-string default falls through `_DEVICE_ALIASES[""]` to `dml`

The legacy Python side of the table is checked by:

```bash
uv run python rewrite-in-rust/bootstrap/check_runtime_device_normalization.py
```

## Repeated-Call Behavior

The function is stateless for a fixed platform/default input pair. Repeated calls
with the same inputs must return the same result and must not depend on model,
ONNX Runtime provider, GUI, Web, or adapter state.

## Rollback

Rollback is keeping all production imports unchanged:

```text
inference.device_utils.normalize_runtime_device
```

No production Python caller should import Rust output until a later promotion
record chooses and verifies a bridge.
