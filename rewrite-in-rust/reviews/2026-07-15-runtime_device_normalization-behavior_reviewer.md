# runtime_device_normalization - behavior_reviewer

Date: 2026-07-15
Decision: pass

## Findings

No findings after checking the full role scope.

Behavior evidence checked:

- Python source: `inference/device_utils.py:76` preserves the public `normalize_runtime_device(device, default=None)` boundary, platform default selection, `device or default` handling, stripping/lowercasing, and `_DEVICE_ALIASES` output mapping.
- Rust source: `rewrite-in-rust/rust/crates/v2m-core/src/device.rs:32` exposes current-platform normalization, `:38` exposes explicit platform normalization for parity tests, and `:46` exposes explicit default normalization matching the Python default edge cases.
- Fixture coverage: `rewrite-in-rust/fixtures/runtime_device_normalization.tsv:2` through `:23` covers non-Windows and Windows defaults for `None`, empty string, and whitespace; aliases `cuda`, `directml`, `dml`, `gpu`, and `cpu`; case/whitespace normalization; unknown string preservation after strip/lowercase; explicit `dml` and `cpu` defaults; and Python's explicit empty-string default behavior.
- Python harness: `rewrite-in-rust/bootstrap/check_runtime_device_normalization.py:31` mutates and restores `_IS_WINDOWS` so the shared fixture table checks both Windows and non-Windows platform default behavior on the current host.
- Rust harness: `rewrite-in-rust/rust/crates/v2m-core/src/device.rs:88` consumes the same TSV table, and `:109` through `:154` adds targeted tests for platform defaults, aliases, unknown values, and explicit default edge cases.
- Rollback route: `rewrite-in-rust/manifest.yaml` keeps `current_owner: legacy` and documents rollback as keeping `inference.device_utils.normalize_runtime_device` as runtime owner; `rewrite-in-rust/bootstrap/runtime_device_normalization.md` states no production Python caller should import Rust output until a later promotion record chooses and verifies a bridge.

## Checks

- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml device`: passed, 5 tests passed, 0 failed, 8 filtered out.
- `uv run python rewrite-in-rust/bootstrap/check_runtime_device_normalization.py`: passed.

## Residual Risk

This review proves parity for the declared public `str | None` input surface and explicit string defaults represented in the fixtures. It does not prove behavior for out-of-contract Python values such as arbitrary non-string objects, and it does not verify any ONNX Runtime provider selection, DirectML adapter enumeration, GUI/Web caller behavior, or a future Python/Rust bridge. Those capabilities are explicitly outside this unit and remain legacy-owned.

## Promotion Note

This behavior review does not block promotion. The behavior evidence is strong enough for the coordinator to use this report as the `runtime_device_normalization` behavior-review input for a state update, subject to any separate coordinator-required review or batching policy.
