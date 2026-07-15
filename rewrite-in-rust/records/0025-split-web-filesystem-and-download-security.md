# 0025 - Split Web Filesystem Picker and Download Security

## Context

The provisional `web_filesystem_download_security` unit mixed two behaviors in
`web_server.py`:

- local filesystem picker path/listing behavior;
- output download authorization for files registered on pipeline tasks.

The first is a product/local-filesystem contract. The second is a security
authorization contract tied to `Task.output_files` and Flask `send_file`
responses. They share path handling concerns, but their dependencies and review
risk are different.

## Decision

Replace `web_filesystem_download_security` with two units:

- `web_filesystem_picker_contract`;
- `web_output_download_security`.

Confirm `web_filesystem_picker_contract` as the current implementation unit,
covering:

- `_resolve_picker_path`;
- `_input_value_for_path`;
- `_filesystem_root_entry`;
- `_filesystem_roots`;
- `_parse_extensions`;
- `_filesystem_entry`;
- `GET /api/filesystem/roots`;
- `GET /api/filesystem/list`.

Leave `web_output_download_security` planned/provisional for a later pass,
covering `_safe_requested_download_path`, `_authorized_output_file`, and
`GET /api/download/<path>`.

## Consequences

- Picker fixtures can use fake project/home roots, synthetic directory entries,
  and Flask test-client route calls without creating registered tasks.
- Download security fixtures can later focus on traversal, Windows-path,
  canonical registered-output matching, and response filename behavior without
  mixing picker sorting or extension filtering.
- No Rust bridge or Flask route replacement is introduced.

## Reversal

If later implementation needs shared path utilities, add a small shared Rust
helper module while keeping the manifest units separate. Rollback remains
keeping `web_server.py` as the runtime owner.
