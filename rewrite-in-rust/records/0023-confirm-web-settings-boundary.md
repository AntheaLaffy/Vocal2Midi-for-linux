# 0023 - Confirm Web Settings Boundary

## Context

The next Stage 1 unit after Web pipeline events is `web_settings_contract`.
Source inspection shows that settings behavior is concentrated in
`web_server.py`: default settings, merge/load/save helpers, and the
`GET`/`PUT`/`reset` routes.

This boundary is JSON/file behavior. It is separate from pipeline-start config
assembly, filesystem picker security, output downloads, model download request
validation, and SocketIO task behavior.

## Decision

Confirm `web_settings_contract` as one settings persistence unit covering:

- `DEFAULT_SETTINGS`;
- `_merge_settings`;
- `_load_settings_from_disk`;
- `_save_settings_to_disk`;
- `GET /api/settings`;
- `PUT /api/settings`;
- `POST /api/settings/reset`.

Use `serde_json` as a normal `v2m-core` dependency because JSON parsing,
object merging, and pretty UTF-8 serialization are selected unit behavior, not
only fixture plumbing.

Do not introduce a Flask route bridge or replace production `web_server.py`.

## Consequences

- The unit can be verified with JSONL fixtures and a temporary settings file.
- Unknown top-level settings remain filtered while unknown inner keys inside
  known sections remain preserved, matching Python `.update`.
- Later `web_filesystem_download_security` and `web_model_download_contract`
  units must not absorb settings persistence behavior.

## Reversal

If later packaging wants to avoid `serde_json` in `v2m-core`, move this unit
behind a separate crate before promotion. Until then, rollback is keeping
`web_server.py` settings helpers and routes as the production owners.
