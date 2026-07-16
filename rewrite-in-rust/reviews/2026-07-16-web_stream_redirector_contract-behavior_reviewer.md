# web_stream_redirector_contract - behavior_reviewer

Date: 2026-07-16
Decision: pass-with-followups

## Findings

- Severity: low
- Location: rewrite-in-rust/rust/crates/v2m-core/src/web_stream.rs:57
- Issue: `__getattr__` parity is proven only for the `encoding` attribute, while the Python contract delegates any missing attribute to the wrapped stream.
- Evidence: Python delegates `getattr(self.stream, name)` generically at `web_stream_redirector.py:45`. The Rust model returns `"utf-8"` only for `"encoding"` and `None` for other names at `rewrite-in-rust/rust/crates/v2m-core/src/web_stream.rs:57`. The fixture table has a single delegated-attribute case for `encoding` at `rewrite-in-rust/fixtures/web_stream_redirector_contract.jsonl:6`.
- Required fix: Before any Rust-owned stream bridge promotion, add at least one non-`encoding` delegated attribute fixture or an attribute-map based model, or explicitly narrow the public bridge contract to the attributes callers may require.

No blocking parity findings were found for `write`, whitespace-only writes, newline stripping, callback payload shape, callback exception swallowing, original stream write preservation, or `flush`.

## Checks

- `env PYTHONDONTWRITEBYTECODE=1 UV_CACHE_DIR=/tmp/v2m-uv-cache uv run python rewrite-in-rust/bootstrap/check_web_stream_redirector_contract.py`: passed
- `env CARGO_TARGET_DIR=/tmp/v2m-web-stream-review-target cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml web_stream`: passed; 1 `web_stream` test passed
- `git diff --check`: passed

## Residual Risk

The review used fake stream and fake callback fixtures only. It does not prove behavior with live `sys.stdout`/`sys.stderr`, WebSocket delivery, task log construction, stdout/stderr installation and restoration, stream write failures, callback truthiness edge cases, or delegated stream methods beyond the fixture-covered attribute. Those surfaces are either legacy-owned or split into adjacent units by records 0019 and 0021.

## Promotion Note

This behavior role does not block coordinator state update for the current legacy-owned fixture contract, but the delegated-attribute follow-up should be closed or explicitly scoped before promoting a Rust-owned redirector bridge. Runtime ownership remains `legacy`; the manifest was not edited.
