# 0028 - Close Web Output Download Security Gate

## Context

`web_output_download_security` was split from filesystem picker behavior in
record 0025 and confirmed in record 0027. The unit now has fixture-backed Python
and Rust evidence for URL path rejection, registered-output authorization,
canonical matching, route 404 shapes, and successful download metadata.

## Evidence

Focused unit checks:

- `uv run python rewrite-in-rust/bootstrap/check_web_output_download_security.py`
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml web_output_download`

Independent review reports:

- `rewrite-in-rust/reviews/2026-07-16-web_output_download_security-dependency_bootstrap_reviewer.md`
- `rewrite-in-rust/reviews/2026-07-16-web_output_download_security-behavior_reviewer.md`

Both review decisions are `pass`.

Coordinator broad gate after implementation:

- `cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check`
- `cargo clippy --manifest-path rewrite-in-rust/rust/Cargo.toml --all-targets --all-features -- -D warnings`
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml`
- `RUSTDOCFLAGS='-D warnings' cargo doc --manifest-path rewrite-in-rust/rust/Cargo.toml --no-deps`
- `uv run pytest tests/test_web_api.py tests/test_quantization_caller_defaults_contract.py`

## Decision

Accept `web_output_download_security` as reimplemented with passing
dependency/bootstrap and behavior evidence. Keep it `reimplemented`, not
`verified`, until the remaining required review roles are explicitly handled or
the coordinator makes a later stage-level verification decision.

## Consequences

The next coordinator unit is `web_model_download_contract`, still planned and
provisional. Runtime ownership for output downloads stays with legacy Python;
no Rust bridge is introduced.
