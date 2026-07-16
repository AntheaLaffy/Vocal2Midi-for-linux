# web_stream_redirector_contract - error_tracing_reviewer

Date: 2026-07-16
Decision: pass-with-followups

## Findings

- Severity: low
- Location: `web_stream_redirector.py:34`, `web_stream_redirector.py:37`, `rewrite-in-rust/fixtures/web_stream_redirector_contract.jsonl:3`
- Issue: Callback failures are deliberately swallowed with no diagnostic surface. The fixture table confirms the swallowed callback exception behavior, and record 0021 says this is intentional so logging failures do not break pipeline execution. This does not block the current legacy-owned contract, but a future Rust-owned bridge should document whether swallowed callback failures stay silent or expose an internal-only trace counter/log that cannot recurse into the same callback path.
- Evidence: `write_callback_exception_is_swallowed` records the callback call and `callback_error_swallowed: true`; the Python source catches `Exception` and executes `pass`.
- Required fix: Before Rust runtime promotion, either preserve the silent swallow as an explicit compatibility decision in the promotion record or add a non-recursive diagnostic path with matching fixtures.

- Severity: low
- Location: `web_stream_redirector.py:39`, `web_stream_redirector.py:43`, `web_stream_redirector.py:47`, `rewrite-in-rust/rust/crates/v2m-core/src/web_stream.rs:25`, `rewrite-in-rust/rust/crates/v2m-core/src/web_stream.rs:47`, `rewrite-in-rust/rust/crates/v2m-core/src/web_stream.rs:58`
- Issue: Underlying stream `write`, `flush`, and delegated attribute failures are not swallowed by legacy Python, but the current fixtures and Rust model cover only success paths. This is acceptable for the current fixture-bound behavior model, yet a caller-facing Rust bridge must not accidentally convert stream/delegation failures into success outcomes.
- Evidence: Python calls `self.stream.write(text)`, `self.stream.flush()`, and `getattr(self.stream, name)` outside any `try`; the Rust model returns successful `StreamRedirectOutcome` values and the fixture table contains no failing stream or missing-attribute cases.
- Required fix: Before promotion to a real Rust-owned stream bridge, add or document parity for stream write failure, flush failure, and missing delegated attribute propagation.

- Severity: low
- Location: `web_stream_redirector.py:36`, `rewrite-in-rust/rust/crates/v2m-core/src/web_stream.rs:30`, `rewrite-in-rust/fixtures/web_stream_redirector_contract.jsonl:1`
- Issue: Callback payloads forward raw stripped stream text without redaction. That matches the current stream redirector role and keeps SocketIO/task execution out of this unit, but it means this boundary must be treated as a log transport boundary rather than a sanitizer.
- Evidence: Python passes `text.strip()` directly to the callback; Rust stores the same stripped text as `StreamCallbackCall.message`; the fixture table expects stripped payloads such as `hello world` without any redaction transform. No redaction keywords or filtering paths are present in targeted scans.
- Required fix: Do not add ad hoc redaction in this narrow unit. If future user-visible logs need sanitization, add it at a higher log-policy boundary with fixtures so this redirector remains a transport wrapper.

## Checks

- `env PYTHONDONTWRITEBYTECODE=1 UV_CACHE_DIR=/tmp/v2m-uv-cache uv run python rewrite-in-rust/bootstrap/check_web_stream_redirector_contract.py`: passed.
- `env CARGO_TARGET_DIR=/tmp/v2m-web-stream-error-target cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml web_stream`: passed, 1 `web_stream` test.
- `rg -n "except Exception|logger|logging|traceback|redact|password|token|secret|callback|callback_error_swallowed|write\\(|flush\\(|__getattr__|getattr\\(" web_stream_redirector.py rewrite-in-rust/bootstrap/check_web_stream_redirector_contract.py rewrite-in-rust/rust/crates/v2m-core/src/web_stream.rs rewrite-in-rust/fixtures/web_stream_redirector_contract.jsonl`: inspected callback swallowing, logging absence, redaction terms, write/flush/delegation paths.

## Residual Risk

This review intentionally excludes SocketIO emission behavior, task execution, stdout/stderr installation/restoration, and pipeline-event error handling. The current Rust code is a fixture behavior model, not a production stream bridge, so actual cross-language stream exception typing remains unproven until a promotion unit defines that bridge.

## Promotion Note

This role does not block coordinator verification of the current legacy-owned `web_stream_redirector_contract`. Runtime ownership should remain `legacy`; Rust promotion needs explicit follow-up coverage for silent callback failure diagnostics and underlying stream/delegation error propagation.
