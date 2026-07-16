# slice_method_and_bounds_contract - behavior_reviewer rerun

Date: 2026-07-17
Decision: pass

## Findings

No behavior findings remain for this rerun.

The previous blocking unsupported-method repr mismatch is fixed and covered. The compatibility policy requires preserving unsupported-method errors at `rewrite-in-rust/manifest.yaml:963`, and the bootstrap boundary includes exact unsupported/empty method errors at `rewrite-in-rust/bootstrap/slice_method_and_bounds_contract.md:20`. Legacy API and CLI callers format unsupported inputs with Python repr at `inference/API/slicer_api.py:64` and `scripts/slice_asr_cli.py:135`; a direct legacy spot check for input `can't` returned `Unsupported slicing method: "can't". Supported values: default, smart, heuristic, grid` from both surfaces.

The fixture table now includes that formerly missing case at `rewrite-in-rust/fixtures/slice_method_and_bounds_contract.jsonl:19`. Rust `python_string_repr` now selects double quotes when the original value contains a single quote and no double quote at `rewrite-in-rust/rust/crates/v2m-core/src/slice_method.rs:311`, and the focused Rust test asserts `can't` plus the both-quotes fallback at `rewrite-in-rust/rust/crates/v2m-core/src/slice_method.rs:460`.

The rest of the behavior surface stayed inside the confirmed boundary: method aliases, repaired mojibake candidates, keyword fallback, CLI/API-specific bound error messages, NaN/infinity comparison behavior, and the absence of a production Rust bridge match the declared scope in `rewrite-in-rust/bootstrap/slice_method_and_bounds_contract.md:5`, `rewrite-in-rust/bootstrap/slice_method_and_bounds_contract.md:21`, `rewrite-in-rust/bootstrap/slice_method_and_bounds_contract.md:23`, and `rewrite-in-rust/records/0044-confirm-slice-method-bounds-boundary.md:7`.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_slice_method_and_bounds_contract.py`: pass with no output.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml slice_method`: pass; 3 `slice_method` tests passed, 73 `v2m_core` tests and 5 `v2m_quant_bridge` tests were filtered out.
- `uv run python - <<'PY' ...`: pass; confirmed both legacy Python normalization surfaces return the double-quoted repr message for input `can't`.
- `rg -n "v2m_core|slice_method|resolve_cli_slice_bounds|resolve_api_slice_bounds|normalize_slicing_method" inference scripts application rewrite-in-rust --glob '!rust/target/**'`: inspected; no production Python caller is wired to the new Rust `slice_method` module. Hits were legacy Python functions, rewrite artifacts, Rust code, and unrelated/rewrite-only `v2m_core` references.

## Residual Risk

This behavior rerun covers the string and optional-float contract only. Actual audio slicing, model execution, CLI argument parsing, filesystem behavior, Web/GUI workflows, and product ergonomics are outside this role. Before any production bridge, the adapter still needs an explicit decision for Python non-string `method` objects, because legacy code normalizes with `str(method)` but formats unsupported errors with the original object repr, while the Rust surface accepts `Option<&str>`. Python `float(...)` conversion failures for non-float-like bound inputs also remain adapter/promotion behavior, not this library contract.

## Promotion Note

This `behavior_reviewer` rerun does not block promotion. It does not mark the manifest verified; coordinator state remains under the coordinator's control after the required review set is satisfied.
