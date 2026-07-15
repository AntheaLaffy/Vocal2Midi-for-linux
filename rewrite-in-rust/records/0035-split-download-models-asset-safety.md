# 0035 - Split Download Models Asset Safety

## Context

The provisional `download_models_asset_safety` unit bundled several independent
behaviors from `download_models.py`:

- GitHub model metadata, asset URL formatting, human-size formatting, and
  marker checks;
- safe zip member validation and merge layout;
- command-line selection rules for `--only`, `--no-qwen`, `--list`, and
  `--qwen-source`;
- GitHub API size lookup and streamed downloads;
- Qwen ModelScope / Hugging Face CLI resolution, package installation fallback,
  download execution, and partial-artifact cleanup.

Those capabilities share one script, but not one verification shape. The archive
layout behavior is pure stdlib path/zip/file behavior. The catalog and marker
helpers need fake filesystem and injected asset-size data. CLI planning needs
fake download outcomes. Network and external CLI execution must be mocked and
reviewed separately.

## Decision

Re-cut `download_models_asset_safety` into four units:

- `download_models_archive_layout_contract`;
- `download_models_asset_catalog_contract`;
- `download_models_cli_selection_contract`;
- `download_models_effectful_fetch_contract`.

Confirm and start only `download_models_archive_layout_contract` now. It covers:

- `_validated_zip_member_path` rejection for empty, NUL, backslash, POSIX
  absolute, Windows absolute/drive, and parent traversal members;
- `_safe_extractall` destination containment behavior through temp-zip fixtures;
- `extract_zip` single-top-level directory stripping;
- direct-file and mixed-top-level merge behavior;
- file-named-top-level behavior where a single top-level entry ending in
  `.onnx` or `.zip` is not stripped.

It does not cover GitHub model metadata, `human_size`, `asset_url`,
`target_has_model`, `qwen_has_weights`, `list_planned`, CLI selection,
`download_github_model`, `stream_download`, GitHub API calls, package
installation, external Qwen CLIs, or model weight/format execution.

## Consequences

The first writer can use temp zip fixtures on the Python side and member-list
fixtures on the Rust side without adding a zip crate or invoking the network.
Later units can decide whether catalog/CLI/effectful behavior should be
implemented as separate narrow Rust models or remain legacy-owned until
promotion planning.

## Reversal

Rollback is keeping `download_models.py` as the runtime owner. No production
bridge is introduced by this split.
