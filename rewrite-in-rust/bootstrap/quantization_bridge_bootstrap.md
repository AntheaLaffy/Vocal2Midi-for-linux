# quantization_bridge_bootstrap Bootstrap

## Boundary

`quantization_bridge_bootstrap` should prove a Python-to-Rust runtime seam for
quantization without changing production callers by default.

The bridge must cover the public quantization behavior, not private algorithm
helpers:

- `should_apply_quantization(mode, quantization_step)`
- `quantize_notes(notes, tempo, quantization_step, mode)`
- in-place note-list sorting and onset/offset mutation
- pitch and lyric metadata preservation
- mode dispatch for `simple`, `smart`, `bayes`, `dp`, and unknown fallback
- no-quantize behavior for disabled non-`dp` modes

GUI, Web, application defaults, pipeline routing, and final runtime owner change
remain separate promotion work.

## Seam Decision

Use a CLI/subprocess JSON bridge for the first proof unless a later record
replaces it.

PyO3/maturin is feasible but not prepared: the Python project has no extension
build setup and the Rust workspace is a plain library crate. A `cdylib`/`ctypes`
bridge is less attractive because it needs a manual C ABI, string ownership,
error ownership, and unsafe buffer handling. Keeping legacy is the current
default and rollback path.

The CLI bridge should run once per quantization operation, never once per note.

## Payload Contract

Request:

```json
{
  "version": 1,
  "mode": "bayes",
  "tempo": 120.0,
  "quantization_step": 60,
  "notes": [
    {
      "index": 0,
      "onset": 0.12,
      "offset": 0.34,
      "pitch": 60.0,
      "lyric": "la"
    }
  ]
}
```

Response:

```json
{
  "ok": true,
  "applied": true,
  "notes": [
    {
      "index": 0,
      "onset": 0.125,
      "offset": 0.375
    }
  ]
}
```

The response should provide mutation instructions, not new Python note objects.
The Python wrapper must mutate the original objects and preserve non-timing
metadata.

## Error And Fallback Contract

The bridge must map these cases before production routing changes:

- invalid JSON
- missing or unsupported payload version
- schema/type errors
- unsupported mode if a mode is intentionally unsupported by the bridge
- non-finite note timings
- non-positive tempo
- overflow-sized tick conversions
- child process nonzero exit, crash, or timeout

Legacy Python must remain the default. A feature flag or explicit backend
setting such as `V2M_QUANT_BACKEND=legacy|rust-json` should select the bridge
until promotion reviews pass. Rollback is returning that setting to `legacy`.

## Cancellation And Performance

The wrapper should check cancellation before and after the bridge call. If the
bridge runs as a subprocess and a Web task cancellation is observed while it is
running, the child process must be killable without corrupting pipeline state.

Benchmarking should use representative note counts and all public modes. The
bridge must not emit logs on stdout that mix with JSON responses; diagnostics
belong on stderr or a structured error field.

## Verification Before Caller Promotion

Required before production callers route to Rust:

- JSON payload fixture tests for all modes and disabled quantization
- bridge parity tests comparing legacy Python and Rust outputs for simple,
  smart, dp, and bayes public dispatch
- wrapper tests proving original note objects are sorted/mutated in place
- invalid payload and invalid numeric input tests
- fallback tests proving legacy remains selectable
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml`
- `uv run pytest tests/test_web_api.py`

Implemented bootstrap artifacts:

- `rewrite-in-rust/rust/crates/v2m-quant-bridge/`: `v2m-quant-bridge`
  stdin/stdout JSON CLI proof.
- `inference/quant/rust_bridge.py`: opt-in Python wrapper selected by
  `backend="rust-json"` or `V2M_QUANT_BACKEND=rust-json`; default remains
  `legacy`.
- `rewrite-in-rust/fixtures/quantization_bridge_payloads.jsonl`: versioned
  success fixtures for simple, smart, bayes, dp, disabled, unknown fallback,
  uppercase, padded mode, null mode, and empty note payloads.
- `rewrite-in-rust/fixtures/quantization_bridge_errors.jsonl`: invalid JSON,
  unsupported version, schema/type, tempo, numeric, and overflow error fixtures.
- `rewrite-in-rust/bootstrap/check_quantization_bridge_bootstrap.py`: parity
  checker comparing CLI output with legacy Python behavior and wrapper fallback
  behavior. It also covers timing-only note objects, wrapper-side non-finite
  timing diagnostics, and subprocess startup failure wrapping.

Copyable bootstrap check:

```bash
cargo build --manifest-path rewrite-in-rust/rust/Cargo.toml --bin v2m-quant-bridge
uv run python rewrite-in-rust/bootstrap/check_quantization_bridge_bootstrap.py
```

## Rollback

Rollback is keeping `inference.quant.quantization` as the production runtime
owner and selecting the `legacy` backend.
