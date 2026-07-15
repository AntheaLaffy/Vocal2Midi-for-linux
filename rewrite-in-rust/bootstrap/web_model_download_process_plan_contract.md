# web_model_download_process_plan_contract Bootstrap

## Boundary

`web_model_download_process_plan_contract` covers the deterministic subprocess
plan and output-line parser in `web_model_download_manager.py`:

```text
ModelDownloadManager._build_command
ModelDownloadManager._build_process_env
ModelDownloadManager._popen_process_group_kwargs
ModelDownloadManager._read_process_output
ModelDownloadManager._handle_output_line
ModelDownloadManager._guess_model_from_line
ModelDownloadManager._emit_progress_for_model
ModelDownloadManager._emit_log
ModelDownloadManager._emit_progress
```

The compatibility surface is:

- command construction starts with the Python executable and
  `download_models.py`, appends one `--only` per selected model in order,
  appends `--qwen-source` only when `qwen` is selected, and appends `--force`
  only when force is true;
- process env planning always sets `PYTHONUNBUFFERED=1`;
- `system` proxy mode inherits the environment;
- `none` proxy mode removes all upper/lower proxy keys;
- `manual` proxy mode removes inherited proxy keys and sets HTTP/HTTPS/ALL
  proxy variables in upper/lower case using the trimmed proxy URL;
- POSIX process spawn kwargs are `{start_new_session: true}`;
- Windows process spawn kwargs are `{creationflags: CREATE_NEW_PROCESS_GROUP}`;
- fake process output is split on `\n` and `\r`, empty lines are ignored, and a
  final unterminated buffer is handled;
- log level is `error` for lines containing `failed` or `error`, `success` for
  lines containing `ready` or `already`, and `info` otherwise;
- active model guessing follows selected-model order and matches qwen names,
  GitHub model ids, asset names, and target directory names;
- progress parsing preserves the legacy `\b(\d{1,3})%\b` behavior, including
  the fact that a line ending in `50%` does not match;
- per-model progress is mapped into total task progress, clamped to `0..99`
  while the process is still running, and never decreases;
- logs are capped to the most recent 500 entries.

The unit does not cover real child process spawning, process waiting, task
completion/failure/cancel transitions, `create_task` registry mutation,
active-task locking, stop requests, process-tree termination, SocketIO delivery
errors, model downloads, network traffic, package installation, archive
extraction, or asset marker safety.

## Dependency Expansion

The selected behavior uses:

- stdlib: `pathlib`, `sys.executable`, `os.environ`, `os.name`,
  `subprocess.CREATE_NEW_PROCESS_GROUP`, character-by-character stdout reads,
  regex search, list/set mutation, and integer truncation;
- local metadata: `download_models.GITHUB_MODELS`, `QWEN_LOCAL_DIR`, and model
  labels.

Fixture parity injects or normalizes every unstable value: Python executable,
project root, OS name, base environment, fake process output, and log
timestamps.

## Seam

Use the default rewrite seam:

- kind: independent Rust library
- crate: `rewrite-in-rust/rust/crates/v2m-core`
- runtime owner: legacy Python
- bridge dependencies: none

The Rust side models the process plan and output parser from explicit JSON
fixture inputs. No production bridge is introduced.

## Fixture Harness

Python and Rust tests consume:

```text
rewrite-in-rust/fixtures/web_model_download_process_plan_contract.jsonl
```

The legacy Python side is checked by:

```bash
uv run python rewrite-in-rust/bootstrap/check_web_model_download_process_plan_contract.py
```

## Rollback

Rollback is keeping production ownership unchanged:

```text
web_model_download_manager.ModelDownloadManager
download_models.py
```

No Web caller should import Rust output for this unit until a later promotion
record chooses and verifies a bridge.
