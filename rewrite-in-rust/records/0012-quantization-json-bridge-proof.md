# 0012 - Implement Quantization JSON Bridge Proof

## Context

The verified quantization algorithm units existed only as Rust library helpers.
Runtime promotion still needed a small Python-to-Rust seam proof that did not
change GUI, Web, application, or pipeline defaults.

The bridge also needed an ordering contract. Python quantization mutates and
sorts the original note list in place, while a subprocess cannot return Python
objects.

## Decision

Add `v2m-quant-bridge`, a stdin/stdout JSON CLI under the independent Rust
workspace. It accepts one versioned quantization payload per process and returns
mutation instructions keyed by the caller-provided original note indexes.

Keep legacy Python as the default owner. Add `inference/quant/rust_bridge.py`
as an opt-in wrapper only; it requires `backend="rust-json"` or
`V2M_QUANT_BACKEND=rust-json` and a bridge executable path. It mutates the
original Python note objects and reorders the original list according to the
Rust response.

The response order is the post-quantization order. For applied modes, the bridge
uses stable onset ordering to pair Rust-mutated rows with original indexes. For
disabled non-`dp` modes, the response preserves original order and values.

Review follow-up fixed two bridge-wrapper edge cases before verification:

- timing-only note objects are accepted by sending a bridge-only default pitch
  while leaving the original object metadata untouched;
- wrapper-originated non-finite timings and subprocess startup failures are
  mapped to `QuantizationBridgeError` before they leak as JSON parser errors or
  raw `OSError` subclasses.

## Consequences

- The bridge proof adds `serde` and `serde_json` only to the bridge binary
  crate, not to production Python.
- Production callers remain unchanged until the later promotion unit.
- Bridge fixtures now cover payload parity, schema/numeric errors, fallback
  selection, and child-process failure handling.
- Promotion reviews must still check architecture, behavior, and error tracing
  before any caller routes to Rust by default.

## Reversal

Rollback is selecting the legacy backend or not importing
`inference.quant.rust_bridge`. No production routing depends on the bridge proof.
