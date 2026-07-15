# 0024 - Close Web Pipeline Follow-ups and Settings Bootstrap

## Context

The first `web_pipeline_execution_events` behavior review failed because the
fixture/Rust model did not fully prove stdout/stderr restoration, SocketIO
payload identity/order, or multi-file output ordering.

The coordinator updated the shared fixture table, Python checker, Rust model,
bootstrap note, dependency record, boundary record, and manifest text for that
unit. The updated model now asserts:

- `stdout_restored` and `stderr_restored`;
- one ordered SocketIO emit trace with event name, room, `task_id`, log
  timestamp, status payload shape, and completed result payload;
- multiple files per collected output extension.

The same pass also completed focused implementation checks for
`web_settings_contract` and received a dependency/bootstrap review.

## Evidence

`web_pipeline_execution_events`:

- `uv run python rewrite-in-rust/bootstrap/check_web_pipeline_execution_events.py`
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml web_pipeline_events`
- `rewrite-in-rust/reviews/2026-07-16-web_pipeline_execution_events-behavior_reviewer-rerun.md`

The rerun behavior review decision is `pass`.

`web_settings_contract`:

- `uv run python rewrite-in-rust/bootstrap/check_web_settings_contract.py`
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml web_settings`
- `rewrite-in-rust/reviews/2026-07-16-web_settings_contract-dependency_bootstrap_reviewer.md`

The dependency/bootstrap review decision is `pass`.

Shared gate after these changes:

- `cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check`
- `cargo clippy --manifest-path rewrite-in-rust/rust/Cargo.toml --all-targets --all-features -- -D warnings`
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml`
- `RUSTDOCFLAGS='-D warnings' cargo doc --manifest-path rewrite-in-rust/rust/Cargo.toml --no-deps`
- `uv run python rewrite-in-rust/bootstrap/check_application_job_contract.py`
- `uv run python rewrite-in-rust/bootstrap/check_web_pipeline_config_mapping.py`
- `uv run python rewrite-in-rust/bootstrap/check_web_task_registry_contract.py`
- `uv run python rewrite-in-rust/bootstrap/check_web_stream_redirector_contract.py`
- `uv run python rewrite-in-rust/bootstrap/check_web_pipeline_execution_events.py`
- `uv run python rewrite-in-rust/bootstrap/check_web_settings_contract.py`
- `uv run pytest tests/test_web_api.py tests/test_quantization_caller_defaults_contract.py`

## Decision

Accept the `web_pipeline_execution_events` follow-up fixes as closed for the
behavior gate. Keep the unit `reimplemented`, not `verified`, until the
remaining required review roles are complete.

Accept `web_settings_contract` dependency/bootstrap as passing. Keep it
`reimplemented`, not `verified`, until behavior and error-tracing reviews pass.

## Consequences

The next coordinator step can move to `web_filesystem_download_security`
without carrying the earlier pipeline behavior failure as an open blocker.

Rollback remains unchanged: production runtime ownership stays with legacy
Python for both units.
