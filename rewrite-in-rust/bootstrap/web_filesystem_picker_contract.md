# web_filesystem_picker_contract Bootstrap

## Boundary

`web_filesystem_picker_contract` covers only the local path picker behavior in
`web_server.py`:

```text
_resolve_picker_path
_input_value_for_path
_filesystem_root_entry
_filesystem_roots
_parse_extensions
_filesystem_entry
GET /api/filesystem/roots
GET /api/filesystem/list
```

The public compatibility surface is:

- empty and whitespace picker paths resolve to `PROJECT_ROOT`;
- relative picker paths resolve under `PROJECT_ROOT`;
- `~` paths expand through `os.path.expanduser`;
- absolute paths outside the project remain absolute UI input values;
- project-root UI input value is `.`;
- nested project paths use POSIX relative UI input values;
- root entries include project, home, and platform root candidates when they
  exist and are readable directories, with duplicate paths removed;
- extension filters are comma-split, lowercased, dot-prefixed, blank-filtered,
  and deduplicated;
- directory mode returns directories only;
- file mode returns directories plus matching files, or all files when no
  extension filter is supplied;
- file-path inputs fall back to their parent directory;
- invalid mode, missing path, and unreadable directory errors preserve status
  and exact error shape;
- listed entries are sorted with directories first, then by lowercase name.

The unit does not cover browser UI code, Flask server replacement, CORS,
SocketIO, output download authorization, registered task outputs, real model
downloads, or pipeline execution.

## Dependency Expansion

The selected source uses:

- stdlib: `pathlib`, `os.path.expanduser`, `os.scandir`, `os.sep`, `Path.home`,
  and `os.name`;
- local global: `PROJECT_ROOT`;
- Flask test-client route wrappers for response status and JSON shape.

Fixture parity uses temporary project/home directories, synthetic files and
directories, and patched `os.scandir` results so fixture `children` order is
the explicit legacy enumeration order. No live Web server, network, model
runtime, task manager output registration, or browser frontend is needed.

## Seam

Use the default rewrite seam:

- kind: independent Rust library
- crate: `rewrite-in-rust/rust/crates/v2m-core`
- runtime owner: legacy Python
- bridge dependencies: none

The Rust side models path/root/listing outputs from explicit fixture inputs. No
production bridge is introduced.

## Fixture Harness

Python and Rust tests consume:

```text
rewrite-in-rust/fixtures/web_filesystem_picker_contract.jsonl
```

The legacy Python side is checked by:

```bash
uv run python rewrite-in-rust/bootstrap/check_web_filesystem_picker_contract.py
```

## Rollback

Rollback is keeping production ownership unchanged:

```text
web_server.py filesystem picker helpers and routes
```

No Web caller should import Rust output for this unit until a later promotion
record chooses and verifies a bridge.
