# download_models_cli_selection_contract Bootstrap

## Boundary

`download_models_cli_selection_contract` covers deterministic command-line
selection behavior in `download_models.py`:

```text
parse_args
main
```

The compatibility surface is:

- accepted `--only` choices from `GITHUB_MODEL_BY_NAME` plus `qwen`;
- `--only` repetition and default `None`;
- `--force`, `--qwen-source`, `--no-qwen`, and `--list` defaults;
- invalid `--only` and `--qwen-source` choices exit with parser status 2;
- list mode passes `args.qwen_source`, or `skip` when `--no-qwen` is set, then
  returns 0 without creating the experiments directory or calling downloads;
- normal mode creates the experiments directory before planning downloads;
- GitHub model downloads are selected by membership but executed in catalog
  order;
- Qwen is downloaded when `--no-qwen` is false and either no explicit selection
  is provided or `qwen` is selected;
- fake download failures are aggregated in GitHub order and then Qwen;
- success and failure output shape with color disabled.

The unit does not cover static catalog metadata, `list_planned` display rows,
GitHub API requests, streamed downloads, archive extraction, package
installation, external ModelScope/Hugging Face CLIs, cleanup of partial Qwen
artifacts, or model weight/format execution.

## Dependency Expansion

The selected behavior uses only Python stdlib:

- `argparse.ArgumentParser`
- `contextlib.redirect_stdout` and `redirect_stderr`
- `io.StringIO`
- `tempfile.TemporaryDirectory`
- `pathlib.Path`

No third-party Python package, native library, Rust network client, archive
crate, or external CLI is required for the compatibility surface.

## Seam

Use the default rewrite seam:

- kind: independent Rust library
- crate: `rewrite-in-rust/rust/crates/v2m-core`
- runtime owner: legacy Python
- bridge dependencies: none

The Python side proves legacy behavior by patching `download_github_model`,
`download_qwen`, `list_planned`, `_USE_COLOR`, and `EXPERIMENTS_DIR`. The Rust
side mirrors equivalent parser and main-plan decisions from explicit fixture
inputs. No production bridge is introduced.

## Fixture Harness

Python and Rust tests consume:

```text
rewrite-in-rust/fixtures/download_models_cli_selection_contract.jsonl
```

The legacy Python side is checked by:

```bash
uv run python rewrite-in-rust/bootstrap/check_download_models_cli_selection_contract.py
```

## Rollback

Rollback is keeping production ownership unchanged:

```text
download_models.py
```

No caller should import Rust output for this unit until a later promotion record
chooses and verifies a bridge.
