# web_filesystem_picker_contract - behavior_reviewer rerun

Date: 2026-07-16
Decision: pass

## Findings

No findings.

The follow-up fixes address the prior behavior blockers:

- `rewrite-in-rust/fixtures/web_filesystem_picker_contract.jsonl:11` adds `list_case_only_ties_preserve_enumeration_order`, covering same-lowercase directory and file names in the fixture enumeration order.
- `rewrite-in-rust/bootstrap/check_web_filesystem_picker_contract.py:203` through `rewrite-in-rust/bootstrap/check_web_filesystem_picker_contract.py:216` patch `os.scandir` so fixture `children` are the legacy enumeration order, and `rewrite-in-rust/bootstrap/check_web_filesystem_picker_contract.py:229` through `rewrite-in-rust/bootstrap/check_web_filesystem_picker_contract.py:238` exact-check error payloads after preserving dynamic `error_contains` text.
- `rewrite-in-rust/rust/crates/v2m-core/src/web_filesystem_picker.rs:201` now sorts entries by `(entry_type != "directory", name.to_lowercase())`, matching Python's stable sort key at `web_server.py:581` without an original-name tie-breaker.
- `rewrite-in-rust/rust/crates/v2m-core/src/web_filesystem_picker.rs:403` through `rewrite-in-rust/rust/crates/v2m-core/src/web_filesystem_picker.rs:414` serialize failed list responses as only `status_code`, `success`, and `error`, matching the legacy error response shapes at `web_server.py:553`, `web_server.py:560`, and `web_server.py:576`.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_web_filesystem_picker_contract.py`: passed.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml web_filesystem_picker`: passed; 1 picker test passed, 47 v2m-core tests filtered out, and 0 bridge tests ran.

## Residual Risk

This review only covers behavior parity for the fixture-backed POSIX picker contract. It does not prove live filesystem behavior beyond the patched `os.scandir` harness, Windows path/root behavior, browser UI behavior, Flask server replacement, output download authorization, or the separate error tracing and product ergonomics roles listed for the unit in `rewrite-in-rust/manifest.yaml:528`.

## Promotion Note

Behavior review no longer blocks promotion of `web_filesystem_picker_contract`. Coordinator state updates remain separate; runtime ownership should stay with `web_server.py` until a later promotion record chooses and verifies a bridge.
