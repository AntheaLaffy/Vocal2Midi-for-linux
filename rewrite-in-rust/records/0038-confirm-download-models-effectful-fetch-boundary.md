# 0038 - Confirm Download Models Effectful Fetch Boundary

## Context

`download_models_effectful_fetch_contract` is the final slice from the former
`download_models_asset_safety` unit. The archive, catalog, and CLI-selection
contracts are now isolated. This unit covers the remaining effectful helper
behavior in `download_models.py`, but only through mocked effects:

- GitHub release asset-size API lookup, cache, and fallback behavior;
- stream-download progress formatting and byte counts;
- `download_github_model` size/archive/marker/HTTP error mapping;
- `_run_cli`, `_pip_install`, `_resolve_cli`, and venv/PATH CLI resolution;
- Qwen artifact cleanup for `.gitattributes` and `*.incomplete`;
- ModelScope and Hugging Face CLI download flow with fake return codes;
- `download_qwen` source selection, force handling, and auto fallback.

The behavior is deterministic when network, file, subprocess, package install,
archive extraction, marker checks, and Qwen weight checks are replaced with
fixtures or temp-directory fakes.

## Decision

Confirm `download_models_effectful_fetch_contract` as an independent Rust
library seam with JSONL fixtures. The Python checker patches all live effect
boundaries and uses temp files only for stream bytes and cleanup behavior. The
Rust unit mirrors the same state machine and user-visible output decisions.

The fixture intentionally preserves legacy quirks:

- GitHub API failures cache an empty size map;
- stream progress writes carriage-return updates and a trailing newline;
- GitHub size mismatch stops before extraction;
- non-HTTP stream exceptions such as `URLError` and `TimeoutError` escape after
  the initial download lines while the temporary zip is still cleaned up;
- temporary GitHub zip files are removed on success and failure;
- `.gitattributes` removal is attempted before `*.incomplete` cleanup;
- Qwen artifact unlink failures are swallowed, retain the file, and do not print
  a successful removal log;
- Qwen CLI install success may still fall back to the venv binary path when the
  CLI resolver cannot find the command;
- `download_qwen("auto")` tries ModelScope first and only falls back to Hugging
  Face after a false ModelScope result;
- `download_qwen("skip")` is an unknown-source failure in download mode.

## Consequences

This unit can validate download-facing control flow without contacting GitHub,
installing packages, extracting real archives, running ModelScope/Hugging Face
CLIs, or reading model weights. Promotion will still need an explicit bridge
decision because the Rust model intentionally does not perform real network or
process IO.

## Reversal

Rollback is keeping `download_models.py` as the runtime owner. No production
bridge is introduced by this record.
