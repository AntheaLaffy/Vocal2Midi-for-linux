# web_output_download_security - dependency_bootstrap_reviewer

Date: 2026-07-16
Decision: pass

## Findings

No findings.

The manifest boundary is confirmed. The unit should not be split, merged,
deferred, or replaced for dependency/bootstrap reasons.

Evidence:

- `rewrite-in-rust/manifest.yaml:540` confirms the unit as
  `web_output_download_security`, with legacy ownership and source references
  limited to `web_server.py` and `web_task_manager.py`.
- `rewrite-in-rust/manifest.yaml:549` defines the compatibility surface as
  registered-output-only authorization, project-relative URL resolution, exact
  canonical matching, traversal rejection, backslash/NUL rejection, Windows
  drive-prefix rejection, and unregistered-file rejection.
- `rewrite-in-rust/records/0027-confirm-web-output-download-security-boundary.md:15`
  confirms the unit around `WINDOWS_DRIVE_RE`,
  `_safe_requested_download_path`, `_authorized_output_file`, and
  `GET /api/download/<path>`, while excluding picker behavior, task lifecycle,
  pipeline output collection, Flask replacement, streaming internals, and UI
  behavior.
- `rewrite-in-rust/dependencies/web_output_download_security.yaml:4` expands
  the capability set into requested path validation, registered output
  authorization, and route response shape.
- `rewrite-in-rust/dependencies/web_output_download_security.yaml:16` keeps the
  seam as a legacy-owned independent library unit with no bridge dependencies.
- `rewrite-in-rust/dependencies/web_output_download_security.yaml:48` keeps
  Flask route binding/streaming, task manager lifecycle, pipeline output
  collection, and filesystem picker behavior legacy-owned or assigned to other
  units.
- `rewrite-in-rust/bootstrap/web_output_download_security.md:41` documents the
  selected dependencies as stdlib path/regex/file checks, local
  `Task.output_files` access, and Flask wrappers only.
- `rewrite-in-rust/bootstrap/web_output_download_security.md:50` states that
  parity uses temporary roots, synthetic files, symlinks, task outputs, and
  Flask test-client calls, with no live Web server, SocketIO, model runtime,
  pipeline execution, network, or browser frontend.
- `web_server.py:768` through `web_server.py:821` matches the scoped helper and
  route behavior: validation rejects unsafe URL paths, authorization compares
  canonical task outputs, and success returns `send_file` with
  `download_name=safe_path.name`.
- `web_task_manager.py:41` and `web_task_manager.py:239` show that
  `Task.output_files` is a simple list populated from output directory globs;
  lifecycle and output collection remain outside this unit.
- `rewrite-in-rust/fixtures/web_output_download_security.jsonl:1` through
  `rewrite-in-rust/fixtures/web_output_download_security.jsonl:24` cover empty,
  absolute, traversal, backslash, NUL, Windows-prefix, unregistered,
  registered-relative, registered-absolute, canonical parent, symlink,
  nonexistent, neighbor-file, route-decoding, encoded-absolute, and empty-route
  cases.
- `rewrite-in-rust/bootstrap/check_web_output_download_security.py:89` through
  `rewrite-in-rust/bootstrap/check_web_output_download_security.py:150` builds
  synthetic task outputs and materializes temp files/symlinks for helper and
  route checks instead of running a server or pipeline.
- `rewrite-in-rust/rust/crates/v2m-core/src/web_output_download.rs:31` through
  `rewrite-in-rust/rust/crates/v2m-core/src/web_output_download.rs:140` models
  the same fixture-backed path validation, canonical authorization, 404 shape,
  body, and download filename behavior without adding Web/model/network
  dependencies.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_web_output_download_security.py`:
  passed.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml web_output_download`:
  passed; 1 `web_output_download` test passed, 48 unrelated tests filtered in
  `v2m-core`, and 5 unrelated bridge tests filtered.
- `cargo tree --manifest-path rewrite-in-rust/rust/Cargo.toml -p v2m-core --edges normal`:
  passed; normal dependency tree is `v2m-core -> serde_json` with only
  `serde_json` transitive crates (`itoa`, `memchr`, `serde_core`, `zmij`).

## Residual Risk

This dependency/bootstrap review does not prove full behavior, error tracing,
or product ergonomics. It also does not run `uv run pytest tests/test_web_api.py`,
which remains listed as broader verification in the manifest. Those are covered
by separate review roles or coordinator verification.

## Promotion Note

This role does not block promotion. The dependency/bootstrap evidence supports
the confirmed unit boundary, kept-legacy decisions, fixture seam, rollback
route, and no-live-server/model/network check strategy.
