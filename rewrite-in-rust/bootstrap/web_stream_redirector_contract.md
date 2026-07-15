# web_stream_redirector_contract Bootstrap

## Boundary

`web_stream_redirector_contract` covers only:

```text
web_stream_redirector.py::WebStreamRedirector
```

The public compatibility surface is:

- `write(text)` calls the callback only when `text.strip()` is non-empty.
- callback payload is `(text.strip(), "info")`.
- callback exceptions are swallowed.
- every write forwards the original, unstripped `text` to the underlying stream.
- `flush()` delegates to the underlying stream.
- missing attributes delegate to the underlying stream through `__getattr__`.

The unit does not cover SocketIO emission payloads, task log entry construction,
stdout/stderr installation/restoration, task lifecycle, or model inference.

## Dependency Expansion

The selected source uses no third-party dependencies. It needs only a stream
object with `write`, `flush`, and arbitrary delegated attributes, plus an
optional callback.

It does not require Flask, Flask-SocketIO, Python threading, ONNX Runtime, Qwen
ASR, PyQt, or any model runtime package.

## Seam

Use the default rewrite seam:

- kind: independent Rust library
- crate: `rewrite-in-rust/rust/crates/v2m-core`
- runtime owner: legacy Python
- bridge dependencies: none

Rust models write/flush/delegation behavior against fixture inputs. No
production stream bridge is introduced.

## Fixture Harness

Python and Rust tests consume:

```text
rewrite-in-rust/fixtures/web_stream_redirector_contract.jsonl
```

The fixtures cover:

- non-empty write callback and original stream write
- whitespace-only write skipping callback
- callback exception swallowing
- missing callback
- flush delegation
- attribute delegation

The legacy Python side is checked by:

```bash
uv run python rewrite-in-rust/bootstrap/check_web_stream_redirector_contract.py
```

## Repeated-Call Behavior

For the same text and callback behavior, writes are deterministic and do not
depend on task state, SocketIO rooms, model state, or filesystem state.

## Rollback

Rollback is keeping production ownership unchanged:

```text
web_stream_redirector.WebStreamRedirector
```

No Web caller should import Rust output for this unit until a later promotion
record chooses and verifies a bridge.
