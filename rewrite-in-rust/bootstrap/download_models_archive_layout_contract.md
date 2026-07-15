# download_models_archive_layout_contract Bootstrap

## Boundary

`download_models_archive_layout_contract` covers the deterministic archive
member safety and merge-layout behavior in `download_models.py`:

```text
extract_zip
_safe_extractall
_validated_zip_member_path
_merge_tree
```

The compatibility surface is:

- unsafe member names are rejected when empty, contain NUL bytes, contain
  backslashes, are POSIX absolute, are Windows absolute or drive-relative, or
  contain `..` path parts;
- safe archive members extract under the destination only;
- a single top-level directory is stripped when its name does not end in
  `.onnx` or `.zip`;
- a single top-level file ending in `.onnx` or `.zip` is not stripped;
- mixed top-level archives merge all top-level entries directly into the target;
- nested directory members preserve their relative structure;
- existing files may be overwritten by later merge copies.

The unit does not cover GitHub model metadata, `human_size`, `asset_url`,
GitHub API size lookup, streamed downloads, target marker checks,
`qwen_has_weights`, CLI argument parsing, `list_planned`, `main`, package
installation, external Qwen CLIs, network access, or model weight/format
execution.

## Dependency Expansion

The selected behavior uses only Python stdlib:

- `zipfile.ZipFile`
- `pathlib.Path`, `PurePosixPath`, and `PureWindowsPath`
- `tempfile.TemporaryDirectory`
- `shutil.copy2`

No third-party Python package, native library, Rust archive crate, network
client, or external CLI is required for the compatibility surface.

## Seam

Use the default rewrite seam:

- kind: independent Rust library
- crate: `rewrite-in-rust/rust/crates/v2m-core`
- runtime owner: legacy Python
- bridge dependencies: none

The Python side proves real `zipfile` extraction effects with temp archives.
The Rust side models equivalent member validation and target layout from
explicit fixture member lists. No production bridge is introduced.

## Fixture Harness

Python and Rust tests consume:

```text
rewrite-in-rust/fixtures/download_models_archive_layout_contract.jsonl
```

The legacy Python side is checked by:

```bash
uv run python rewrite-in-rust/bootstrap/check_download_models_archive_layout_contract.py
```

## Rollback

Rollback is keeping production ownership unchanged:

```text
download_models.py
```

No caller should import Rust output for this unit until a later promotion record
chooses and verifies a bridge.
