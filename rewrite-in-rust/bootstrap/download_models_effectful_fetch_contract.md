# download_models_effectful_fetch_contract Bootstrap

## Boundary

`download_models_effectful_fetch_contract` covers the download-facing helper
behavior in `download_models.py` through mocked effects:

```text
github_api_asset_sizes
stream_download
download_github_model
_run_cli
_have_uv
_pip_install
_venv_bin
_resolve_cli
_cleanup_qwen_artifacts
download_qwen_modelscope
download_qwen_huggingface
download_qwen
```

The compatibility surface is:

- GitHub release API URL, JSON asset-size filtering, failure fallback, and cache
  behavior;
- stream-download byte counting and progress text for known and unknown content
  lengths;
- GitHub model download skip behavior, size hints, size mismatch failure,
  successful extraction/marker verification, corrupt zip handling, unsafe layout
  handling, marker-missing failure, HTTP 404 note, and temp-zip cleanup;
- subprocess return-code and missing-executable mapping;
- `uv` vs `pip` package install command selection;
- venv-bin-first then PATH CLI resolution;
- Qwen cleanup for `.gitattributes` and immediate `*.incomplete` files;
- ModelScope and Hugging Face CLI download command construction, install
  fallback, return-code failures, cleanup, weight-marker checks, and success
  messages;
- `download_qwen` skip-if-present, force handling, explicit source dispatch,
  auto fallback, and unknown-source failure.

The unit does not cover real network traffic, real package installation, real
subprocess execution, real archive extraction, real GitHub asset downloads, or
model weight/format execution.

## Dependency Expansion

The selected behavior uses Python stdlib effect boundaries that are mocked in
the checker:

- `urllib.request.Request` and `urllib.request.urlopen`
- `urllib.error.HTTPError`, `urllib.error.URLError`, `TimeoutError`, and JSON
  decoding errors
- `subprocess.run`
- `shutil.which`
- `tempfile.NamedTemporaryFile` and `TemporaryDirectory`
- `pathlib.Path`
- stdout/stderr capture through `contextlib.redirect_stdout` and
  `redirect_stderr`

No third-party Python package, Rust network client, archive crate, package
installer, external CLI, or model-runtime dependency is required for
verification.

## Seam

Use the default rewrite seam:

- kind: independent Rust library
- crate: `rewrite-in-rust/rust/crates/v2m-core`
- runtime owner: legacy Python
- bridge dependencies: none

The Python side proves legacy behavior by patching every live effect boundary
and by using temp files only where file cleanup itself is the behavior under
test. The Rust side mirrors equivalent control-flow and output decisions from
explicit fixture inputs. No production bridge is introduced.

## Fixture Harness

Python and Rust tests consume:

```text
rewrite-in-rust/fixtures/download_models_effectful_fetch_contract.jsonl
```

The legacy Python side is checked by:

```bash
uv run python rewrite-in-rust/bootstrap/check_download_models_effectful_fetch_contract.py
```

## Rollback

Rollback is keeping production ownership unchanged:

```text
download_models.py
```

No caller should import Rust output for this unit until a later promotion record
chooses and verifies a real IO bridge.
