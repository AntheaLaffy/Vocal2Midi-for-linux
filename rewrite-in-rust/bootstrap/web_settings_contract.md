# web_settings_contract Bootstrap

## Boundary

`web_settings_contract` covers Web settings behavior in `web_server.py`:

- `DEFAULT_SETTINGS` shape and values
- `_merge_settings`
- `_load_settings_from_disk`
- `_save_settings_to_disk`
- `GET /api/settings` response shape
- `PUT /api/settings` known-section update and errors
- `POST /api/settings/reset`

The compatibility surface is:

- top-level sections are `models`, `params`, `debug`, `pipeline`, and
  `downloads`;
- persisted settings merge over defaults only for known top-level sections;
- unknown top-level sections are ignored;
- unknown inner keys inside known object sections are preserved by `.update`;
- a non-object persisted file falls back to defaults;
- missing or malformed settings files fall back to defaults;
- update rejects known sections whose request value is not an object;
- invalid JSON update bodies return a 400 error;
- reset deep-copies defaults and persists them;
- saved settings are UTF-8, pretty JSON, and end with a newline.

The unit does not cover pipeline-start config assembly, model download request
validation, filesystem picker behavior, output download authorization, or live
Flask/SocketIO transport behavior.

## Dependency Expansion

The selected source uses:

- stdlib: `copy`, `json`, `pathlib`, and atomic path replacement
- local: `DEFAULT_SETTINGS`
- Flask route/request helpers in production

The Rust parity model uses `serde_json` because JSON merge, parse, and pretty
serialization are the selected behavior, not just test-fixture plumbing.

No model runtime, ONNX Runtime, Qwen ASR, PyQt, task manager, SocketIO, or
network dependency is required for this unit.

## Seam

Use the default rewrite seam:

- kind: independent Rust library
- crate: `rewrite-in-rust/rust/crates/v2m-core`
- runtime owner: legacy Python
- bridge dependencies: none

No production route imports Rust output before a future promotion record.

## Fixture Harness

Python and Rust tests consume:

```text
rewrite-in-rust/fixtures/web_settings_contract.jsonl
```

The fixtures cover:

- partial known-section merge
- non-object override fallback
- missing/malformed/partial persisted settings load
- update success and persistence
- unknown top-level update ignore
- non-object section rejection
- invalid JSON rejection
- reset behavior
- UTF-8 pretty payload serialization

The legacy Python side is checked by:

```bash
uv run python rewrite-in-rust/bootstrap/check_web_settings_contract.py
```

## Repeated-Call Behavior

For the same defaults, current settings, request body, and settings-file
contents, the resulting settings object and response status are deterministic.
The unit does not depend on active pipeline tasks, model assets, or network
state.

## Rollback

Rollback is keeping production ownership unchanged:

```text
web_server.py settings helpers and routes
```

No Web caller should import Rust output for this unit until a later promotion
record chooses and verifies a bridge.
