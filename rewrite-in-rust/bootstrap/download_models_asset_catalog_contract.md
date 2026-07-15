# download_models_asset_catalog_contract Bootstrap

## Boundary

`download_models_asset_catalog_contract` covers deterministic catalog and
display helpers in `download_models.py`:

```text
GithubModel
GITHUB_MODELS
GITHUB_MODEL_BY_NAME
QWEN_MODEL_ID
QWEN_LOCAL_DIR
human_size
asset_url
target_has_model
qwen_has_weights
list_planned
```

The compatibility surface is:

- static GitHub model metadata and lookup-key ordering;
- Qwen model id and local-dir metadata;
- GitHub release asset URL formatting;
- binary-unit `human_size` output, including the TiB clamp behavior;
- marker checks based on `Path.exists()`;
- Qwen weight checks based on immediate child names ending in `.safetensors` or
  `.bin`;
- `list_planned` output rows with color disabled, injected asset-size maps, fake
  model targets, and temp Qwen state.

The unit does not cover CLI argument parsing, `main`, GitHub API requests,
streamed downloads, archive extraction, package installation, external
ModelScope/Hugging Face CLIs, cleanup of partial Qwen artifacts, or model
weight/format execution.

## Dependency Expansion

The selected behavior uses only Python stdlib:

- `dataclasses.dataclass`
- `pathlib.Path`
- `io.StringIO`
- `contextlib.redirect_stdout`
- `tempfile.TemporaryDirectory`

No third-party Python package, native library, Rust network client, archive
crate, or external CLI is required for the compatibility surface.

## Seam

Use the default rewrite seam:

- kind: independent Rust library
- crate: `rewrite-in-rust/rust/crates/v2m-core`
- runtime owner: legacy Python
- bridge dependencies: none

The Python side proves legacy behavior with temp paths, patched module globals,
and injected asset-size maps. The Rust side mirrors equivalent catalog and
display decisions from explicit fixture inputs. No production bridge is
introduced.

## Fixture Harness

Python and Rust tests consume:

```text
rewrite-in-rust/fixtures/download_models_asset_catalog_contract.jsonl
```

The legacy Python side is checked by:

```bash
uv run python rewrite-in-rust/bootstrap/check_download_models_asset_catalog_contract.py
```

## Rollback

Rollback is keeping production ownership unchanged:

```text
download_models.py
```

No caller should import Rust output for this unit until a later promotion record
chooses and verifies a bridge.
