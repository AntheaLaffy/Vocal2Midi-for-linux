# slice_method_and_bounds_contract - product_ergonomics_reviewer

Date: 2026-07-17
Decision: fail

## Findings

- Severity: medium
- Location: rewrite-in-rust/rust/crates/v2m-core/src/slice_method.rs:311
- Issue: A future promotion would change a user-visible unsupported-method
  diagnostic for method names containing a single quote. The unit's public
  policy requires preserving unsupported-method errors at
  rewrite-in-rust/manifest.yaml:963 and the bootstrap record calls those exact
  legacy errors part of the compatibility surface at
  rewrite-in-rust/bootstrap/slice_method_and_bounds_contract.md:20. Legacy API
  and CLI callers format the original value with Python `repr` at
  inference/API/slicer_api.py:64 and scripts/slice_asr_cli.py:135, so input
  `can't` is shown as `Unsupported slicing method: "can't". Supported values:
  default, smart, heuristic, grid`. Rust always wraps string repr output in
  single quotes at rewrite-in-rust/rust/crates/v2m-core/src/slice_method.rs:313
  and rewrite-in-rust/rust/crates/v2m-core/src/slice_method.rs:327, escaping the
  embedded quote at rewrite-in-rust/rust/crates/v2m-core/src/slice_method.rs:317.
  That would expose `Unsupported slicing method: 'can\'t'. Supported values:
  default, smart, heuristic, grid` instead.
- Evidence: `uv run python -c "... normalize_slicing_method(\"can't\") ..."`
  confirmed both legacy API and CLI surfaces return the double-quoted Python
  `repr` message. The requested fixture checks still pass because unsupported
  method fixtures only cover `unknown` and the empty string at
  rewrite-in-rust/fixtures/slice_method_and_bounds_contract.jsonl:17 and
  rewrite-in-rust/fixtures/slice_method_and_bounds_contract.jsonl:18; the focused
  Rust repr test covers newline but not quote-selection behavior at
  rewrite-in-rust/rust/crates/v2m-core/src/slice_method.rs:446. The existing
  behavior review independently records the same user-visible mismatch at
  rewrite-in-rust/reviews/2026-07-17-slice_method_and_bounds_contract-behavior_reviewer.md:8.
- Required fix: Implement Python-compatible string repr quote selection and
  escaping for unsupported method messages, or record an intentional narrower
  promoted diagnostic contract. Add a fixture for an unsupported method
  containing a single quote, then rerun the legacy fixture harness and Rust
  `slice_method` tests before promotion.

No other product ergonomics findings were found in the declared boundary. The
CLI/API custom-bound messages stay separated as required: CLI errors use
`--min-seconds` / `--max-seconds` at scripts/slice_asr_cli.py:139, API errors use
`min_len_sec` / `max_len_sec` at inference/API/slicer_api.py:691, and Rust exposes
separate resolver surfaces at
rewrite-in-rust/rust/crates/v2m-core/src/slice_method.rs:203 and
rewrite-in-rust/rust/crates/v2m-core/src/slice_method.rs:217.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_slice_method_and_bounds_contract.py`: passed with no output.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml slice_method`: passed; 3 `slice_method` tests passed, 73 `v2m_core` tests and 5 `v2m_quant_bridge` tests were filtered out.
- `uv run python -c "... normalize_slicing_method(\"can't\") ..."`: passed; confirmed both legacy Python surfaces report `Unsupported slicing method: "can't". Supported values: default, smart, heuristic, grid`.
- `rg -n "v2m_core|slice_method|resolve_cli_slice_bounds|resolve_api_slice_bounds|normalize_slicing_method" inference scripts application rewrite-in-rust --glob '!rust/target/**'`: inspected; no production Python caller is wired to the new Rust `slice_method` module. Hits were legacy Python functions, rewrite artifacts, Rust code, and unrelated/rewrite-only `v2m_core` references.

## Residual Risk

This role reviewed product ergonomics for the method/bounds contract only, not
actual audio slicing behavior, model execution, CLI argument parsing, filesystem
effects, or Web/GUI workflows. Because Python remains runtime owner and the Rust
module is not wired into production, current users are not affected yet. Before a
promotion bridge, the adapter still needs explicit treatment for Python
`str(method)` and `float(...)` coercion/conversion behavior, and it must keep the
CLI/API custom-bound message namespaces separate from the already-verified
application `slice_bounds_validation` policy.

## Promotion Note

This product ergonomics role blocks verification until the unsupported-method
message mismatch is fixed or intentionally accepted by a promotion record. The
manifest should remain under coordinator control and should not be moved to
`verified` from this review.
