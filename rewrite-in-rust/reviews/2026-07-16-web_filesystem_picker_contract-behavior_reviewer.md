# web_filesystem_picker_contract - behavior_reviewer

Date: 2026-07-16
Decision: fail

## Findings

- Severity: high
- Location: rewrite-in-rust/rust/crates/v2m-core/src/web_filesystem_picker.rs:201
- Issue: Rust adds an original-name tie-breaker for sorted entries, but legacy Python sorts only by directory-first and `name.lower()`. Python's stable sort preserves enumeration order for case-only ties; Rust reorders those ties lexicographically by original name. A directory containing entries such as `alpha.onnx` and `Alpha.onnx`, or directories `alpha` and `Alpha`, can therefore return a different order.
- Evidence: Legacy sort key is `entries.sort(key=lambda item: (item['type'] != 'directory', item['name'].lower()))` in `web_server.py:581`. Rust compares `(entry_type != "directory", name.to_lowercase(), name.clone())` in `rewrite-in-rust/rust/crates/v2m-core/src/web_filesystem_picker.rs:201`. The fixture table covers lowercase sorting at `rewrite-in-rust/fixtures/web_filesystem_picker_contract.jsonl:8` through `rewrite-in-rust/fixtures/web_filesystem_picker_contract.jsonl:10`, but does not include same-lowercase ties.
- Required fix: Match the legacy comparator by removing the original-name tie-breaker, or add a recorded compatibility decision that intentionally changes tie ordering. Add fixture cases for case-only ties in both directory and file modes before coordinator verification.

- Severity: medium
- Location: rewrite-in-rust/bootstrap/check_web_filesystem_picker_contract.py:48
- Issue: Error response shape is only subset-checked, so fixtures do not prove the public error payload remains exact. Legacy invalid mode, missing path, and unreadable directory responses contain only `success` and `error`; the Rust test adapter always materializes `entries` and `roots` for list responses, including errors.
- Evidence: Legacy error returns are at `web_server.py:553`, `web_server.py:560`, and `web_server.py:575`. The checker's `assert_subset` only requires expected keys and ignores extras at `rewrite-in-rust/bootstrap/check_web_filesystem_picker_contract.py:48`. Rust test serialization always includes `entries` and `roots` at `rewrite-in-rust/rust/crates/v2m-core/src/web_filesystem_picker.rs:414`. The bootstrap requires invalid mode, missing path, and unreadable directory errors to preserve status and error shape at `rewrite-in-rust/bootstrap/web_filesystem_picker_contract.md:35`.
- Required fix: Add exact-shape assertions for error cases, or model error responses so extra list/root fields cannot appear when this contract is serialized.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_web_filesystem_picker_contract.py`: passed
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml web_filesystem_picker`: passed

## Residual Risk

This review did not approve any production bridge, Flask route replacement, browser UI behavior, or output download authorization. Symlink resolution and platform-specific Windows root behavior remain outside the current Linux/POSIX fixture model and should be rechecked before any live filesystem ownership transfer.

## Promotion Note

This behavior review blocks coordinator verification. The unit should not move to verified until entry ordering parity and exact error response shape coverage are fixed or explicitly re-scoped by a record.
