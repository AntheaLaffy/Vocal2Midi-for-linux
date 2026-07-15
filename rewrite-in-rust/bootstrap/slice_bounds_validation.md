# slice_bounds_validation Bootstrap

## Boundary

`slice_bounds_validation` covers only `application/config.py::validate_slice_bounds`.
The public compatibility surface is:

- accepted `slice_min_sec` and `slice_max_sec` float pairs
- `ValueError` as the error type for invalid values
- exact error messages and check ordering

`PipelineConfig`, GUI settings widgets, Web config parsing, and inference runtime
option validation stay legacy-owned.

## Dependency Expansion

`application/config.py` imports only stdlib modules:

- `dataclasses`
- `pathlib.Path`
- `typing.Callable`
- `typing.Optional`

`validate_slice_bounds` does not call local helpers or third-party packages. The
project dependency manifests and vendored source audit remain relevant for later
units, but they do not expand this unit boundary.

## Seam

Use the default rewrite seam:

- kind: independent Rust library
- crate: `rewrite-in-rust/rust/crates/v2m-core`
- runtime owner: legacy Python
- bridge dependencies: none

Do not add PyO3, subprocess, CLI, HTTP, or runtime-router code for this unit.

## Fixture Harness

Rust tests should consume the durable parity table at:

```text
rewrite-in-rust/fixtures/slice_bounds_validation.tsv
```

The fixtures must cover:

- valid boundary pairs: `(0.0, 0.1)`, `(5.0, 10.0)`, `(60.0, 60.0)`
- invalid `slice_min_sec` below `0.0`
- invalid `slice_min_sec` above `60.0`
- invalid `slice_max_sec` below `0.0`
- invalid `slice_max_sec` above `60.0`
- `slice_max_sec == 0.0`, which passes the range check and then raises the
  greater-than-zero message
- `slice_min_sec > slice_max_sec`
- NaN and infinity inputs, preserving Python range-check ordering

The Rust error payload should expose the Python-compatible message string so a
future bridge can map it to `ValueError` without changing fixtures.

The legacy Python side of the table is checked by:

```bash
uv run python rewrite-in-rust/bootstrap/check_slice_bounds_validation.py
```

## Repeated-Call Behavior

The function is stateless. Repeated calls with the same inputs must return the
same result and must not depend on runtime, platform, model, GUI, or Web state.

## Rollback

Rollback is keeping all production imports unchanged:

```text
application.config.validate_slice_bounds
```

No production Python caller should import Rust output until a later promotion
record chooses and verifies a bridge.
