# web_filesystem_picker_contract - dependency_bootstrap_reviewer

Date: 2026-07-16
Decision: pass

## Findings

No findings.

The manifest unit boundary is confirmed. Record 0025 justifies splitting the
former filesystem/download unit because local picker behavior and registered
output download authorization have different dependencies and risk profiles
(`rewrite-in-rust/records/0025-split-web-filesystem-and-download-security.md:5`).
The confirmed picker unit covers `_resolve_picker_path`, `_input_value_for_path`,
root entries, extension parsing, filesystem entries, and the two
`/api/filesystem/*` routes, while `web_output_download_security` remains a
separate planned/provisional unit for `_safe_requested_download_path`,
`_authorized_output_file`, and `/api/download/<path>`
(`rewrite-in-rust/manifest.yaml:519`, `rewrite-in-rust/manifest.yaml:540`).

Dependency expansion is complete for this bootstrap stage. The dependency record
names the stdlib/path capabilities, synthetic root/listing model, fixture table,
kept legacy Flask route binding, live filesystem enumeration, browser UI, and
download authorization decisions
(`rewrite-in-rust/dependencies/web_filesystem_picker_contract.yaml:3`,
`rewrite-in-rust/dependencies/web_filesystem_picker_contract.yaml:46`). The seam
is an independent Rust library with no bridge dependencies and legacy Python as
runtime owner (`rewrite-in-rust/bootstrap/web_filesystem_picker_contract.md:56`).

Fixture coverage is sufficient for dependency/bootstrap promotion. The fixture
table covers empty and whitespace paths, relative and absolute project paths,
home expansion, project-relative input values, extension normalization and
deduplication, root entries, directory mode filtering, file mode extension
filtering, empty-extension file mode, file-path parent fallback, missing paths,
invalid modes, scandir errors, and directory-first lowercase sorting
(`rewrite-in-rust/fixtures/web_filesystem_picker_contract.jsonl:1`). The Python
harness uses temp project/home directories and Flask's test client only for JSON
route shape, with no live Web server, model runtime, network, registered task
outputs, or browser behavior
(`rewrite-in-rust/bootstrap/check_web_filesystem_picker_contract.py:153`).

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_web_filesystem_picker_contract.py`: passed.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml web_filesystem_picker`: passed; 1 picker test passed, 47 filtered out in `v2m-core`, and 0 bridge tests ran.
- `cargo tree --manifest-path rewrite-in-rust/rust/Cargo.toml -p v2m-core --edges normal`: passed; `v2m-core` normal dependency tree shows only `serde_json` and its transitive dependencies.

## Residual Risk

The Rust model is intentionally fixture-bound and POSIX-shaped for this Linux
workspace. It does not prove Windows picker roots, browser UI behavior, live
production directory permission edge cases beyond the explicit scandir error, or
download authorization. Those are either legacy-owned or assigned to the later
`web_output_download_security` unit.

## Promotion Note

This role does not block promotion. The dependency/bootstrap boundary is
confirmed and should not be split, merged, deferred, or replaced for the current
picker contract unit.
