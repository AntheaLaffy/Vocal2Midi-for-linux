# web_model_download_process_termination_contract Bootstrap

## Boundary

`web_model_download_process_termination_contract` covers the deterministic
branch behavior in:

```text
ModelDownloadManager._terminate_process_tree
ModelDownloadManager.stop_task
```

The compatibility surface is:

- `_terminate_process_tree` returns immediately when `process.poll()` is not
  `None`;
- POSIX normal termination calls `os.killpg(process.pid, signal.SIGTERM)`;
- POSIX forced termination calls `os.killpg(process.pid, signal.SIGKILL)`;
- POSIX `ProcessLookupError` is swallowed;
- POSIX `OSError` falls back to `process.terminate()` or `process.kill()`;
- Windows normal termination runs `taskkill /PID <pid> /T`;
- Windows forced termination adds `/F`;
- Windows `subprocess.run` `OSError` falls back to `process.terminate()` or
  `process.kill()`;
- `stop_task` with a running live process sets status `stopping`, sets the stop
  event if present, invokes termination, and returns true when termination does
  not raise;
- `stop_task` returns false when that termination call raises `OSError`.

The unit does not cover Web route response mapping, task creation/start
lifecycle, no-process stop behavior, subprocess command/env parsing, output
parsing, `_execute_download`, real process termination, SocketIO delivery,
network downloads, package installation, archive extraction, or model marker
safety.

## Dependency Expansion

The selected behavior uses:

- stdlib: `os.name`, `os.killpg`, `signal.SIGTERM`, `signal.SIGKILL`,
  `subprocess.run`, `subprocess.DEVNULL`;
- local dataclass: `ModelDownloadTask`;
- fake process methods: `poll`, `terminate`, and `kill`.

Fixture parity injects fake process objects, fake `os.name`, fake `killpg`,
fake `subprocess.run`, and fake termination outcomes. No real OS termination API
is called.

## Seam

Use the default rewrite seam:

- kind: independent Rust library
- crate: `rewrite-in-rust/rust/crates/v2m-core`
- runtime owner: legacy Python
- bridge dependencies:
  - `web_model_download_task_lifecycle_contract`

The Rust side models termination decisions from explicit JSON fixture inputs.
No production bridge is introduced.

## Fixture Harness

Python and Rust tests consume:

```text
rewrite-in-rust/fixtures/web_model_download_process_termination_contract.jsonl
```

The legacy Python side is checked by:

```bash
uv run python rewrite-in-rust/bootstrap/check_web_model_download_process_termination_contract.py
```

## Rollback

Rollback is keeping production ownership unchanged:

```text
web_model_download_manager.ModelDownloadManager.stop_task
web_model_download_manager.ModelDownloadManager._terminate_process_tree
```

No Web caller should import Rust output for this unit until a later promotion
record chooses and verifies a bridge.
