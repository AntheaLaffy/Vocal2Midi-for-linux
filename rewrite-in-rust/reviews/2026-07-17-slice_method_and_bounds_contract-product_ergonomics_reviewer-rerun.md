# slice_method_and_bounds_contract - product_ergonomics_reviewer

Date: 2026-07-17
Decision: pass

## Findings

No findings.

The previously blocking unsupported-method diagnostic mismatch is fixed for this
role. The public policy requires preserving unsupported-method errors and
CLI/API consistency at `rewrite-in-rust/manifest.yaml:963`, and the fixture list
now explicitly includes unsupported-method Python repr quote selection at
`rewrite-in-rust/manifest.yaml:969`. The legacy API and CLI still format the
original method with Python `repr` at `inference/API/slicer_api.py:65` and
`scripts/slice_asr_cli.py:136`; the new fixture case `method_18` records input
`can't` with the expected double-quoted diagnostic at
`rewrite-in-rust/fixtures/slice_method_and_bounds_contract.jsonl:19`. Rust now
chooses double quotes when the value contains a single quote and no double quote
at `rewrite-in-rust/rust/crates/v2m-core/src/slice_method.rs:311`, and the
focused Rust test asserts the same message at
`rewrite-in-rust/rust/crates/v2m-core/src/slice_method.rs:460`.

The CLI/API user-visible bound messages remain separated as required by
`rewrite-in-rust/bootstrap/slice_method_and_bounds_contract.md:21` and
`rewrite-in-rust/bootstrap/slice_method_and_bounds_contract.md:22`. Python CLI
errors use `--min-seconds` / `--max-seconds` at
`scripts/slice_asr_cli.py:145`, while Python API errors use `min_len_sec` /
`max_len_sec` at `inference/API/slicer_api.py:697`. Rust exposes separate
resolver surfaces and message variants at
`rewrite-in-rust/rust/crates/v2m-core/src/slice_method.rs:88`,
`rewrite-in-rust/rust/crates/v2m-core/src/slice_method.rs:203`, and
`rewrite-in-rust/rust/crates/v2m-core/src/slice_method.rs:217`.

The unit is still ergonomically reversible. The manifest keeps the runtime owner
as legacy at `rewrite-in-rust/manifest.yaml:957`, the bootstrap record states
that the independent Rust library has no bridge dependencies at
`rewrite-in-rust/bootstrap/slice_method_and_bounds_contract.md:73`, and rollback
remains the existing Python API/CLI functions at
`rewrite-in-rust/bootstrap/slice_method_and_bounds_contract.md:117`.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_slice_method_and_bounds_contract.py`: passed with no output.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml slice_method`: passed; 3 `slice_method` tests passed, with 73 `v2m_core` tests and 5 `v2m_quant_bridge` tests filtered out.
- `uv run python -c "... normalize_slicing_method(\"can't\") ..."`: passed; both legacy API and CLI surfaces returned `ValueError` with `Unsupported slicing method: "can't". Supported values: default, smart, heuristic, grid`.
- `rg -n "v2m_core|slice_method|resolve_cli_slice_bounds|resolve_api_slice_bounds|normalize_slicing_method" inference scripts application rewrite-in-rust --glob '!rust/target/**'`: inspected; no production Python caller is wired to the new Rust `slice_method` module. Hits were legacy Python call sites, rewrite artifacts, independent Rust code, and existing unrelated/rewrite-only `v2m_core` references.

## Residual Risk

This rerun reviewed product ergonomics for the method/bounds contract only. It
did not review actual audio slicing behavior, model execution, Web/GUI flows,
filesystem effects, argparse parse failures before helper calls, or the future
runtime bridge. Before promotion, any adapter that exposes this Rust library to
Python should explicitly preserve Python-side `str(method)`, original-object
`repr`, and `float(...)` coercion behavior, while keeping CLI/API bound message
namespaces distinct from the already verified application slice-bounds policy.

## Promotion Note

This product ergonomics role no longer blocks coordinator state update for
`slice_method_and_bounds_contract`. The manifest should remain under coordinator
control; this review does not mark it `verified`.
