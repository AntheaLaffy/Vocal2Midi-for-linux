# Third-party Sources

This document is for dependency auditors and Rust migration implementers. The
directory stores source inputs for packages installed in the current `uv.lock`
environment; it is a reference corpus, not an import or build path.

It is split by source type:

- `sources/`: Python source distributions from `uv.lock`
- `upstream_sources/`: upstream source fallbacks for packages that do not ship
  an sdist in the lock file
- `native_sources/`: upstream C, C++, Fortran, and other native library sources
  used by Python wheels through FFI or bundled shared libraries
- `cargo_vendor/`: vendored Rust crates for Python packages that build native
  Rust extensions

Regenerate all vendored sources from the repository root with:

```bash
uv run python scripts/vendor_sources.py --force
uv run python scripts/vendor_native_sources.py --force
uv run python scripts/audit_vendored_sources.py
```

`scripts/vendor_sources.py` reads the installed distributions from the active uv
environment, matches them against `uv.lock`, downloads locked sdists, verifies
their SHA-256 hashes, and extracts them under `third_party/sources/`.

Packages without an sdist in `uv.lock` are listed in
`third_party/sources/MISSING_SOURCES.md`. When a package has a pinned upstream
ref, the same script downloads that source tree under `third_party/upstream_sources/`.

`scripts/vendor_native_sources.py` downloads pinned upstream source archives for
native/FFI libraries and runs `cargo vendor` for Rust-backed Python packages.

Both vendoring scripts remove compiled binary artifacts after extraction,
including wheel files, Python bytecode, shared libraries, object files, static
libraries, jars, wasm files, and executables. The vendored dependency
directories are source-only; the runnable `.venv` may still contain compiled
extensions installed by uv and remains ignored by git.

The generated manifests record `remaining_binary_artifact_count`; vendoring
fails if that count would be non-zero.

`scripts/audit_vendored_sources.py` is the repeatable verification gate. It
checks that every installed Python distribution has a source record, every
no-sdist package has an upstream fallback, git-recursive fallbacks have
non-empty submodule paths, runtime foreign native binaries map to source
directories, no `.git` metadata remains, and `third_party/` contains no compiled
binary artifacts.

## Current Limits

- Source presence does not prove behavioral compatibility or license
  compatibility with a Rust replacement.
- Second-layer or deeper source expansion requires evidence that the code is on
  a selected unit's public call path.
- Generated manifests describe the locked environment only; refresh them after
  dependency changes.
- Do not patch vendored sources to implement Vocal2Midi behavior. Put project
  behavior in the owning Python or Rust module and record the source reference.

## Licensing

Vendored sources retain their upstream licenses and notices. The repository's
Apache-2.0 license does not replace those terms. See
[`ACKNOWLEDGEMENTS.md`](../ACKNOWLEDGEMENTS.md) and the license files within each
source tree before copying or adapting code.
