# hfa_dictionary_g2p_core - dependency_bootstrap_reviewer

Date: 2026-07-17
Decision: pass-with-followups

Unit: `hfa_dictionary_g2p_core`
Role: `dependency_bootstrap_reviewer`
Writer: `/root/hfa_dictionary_writer`
Reviewer: `/root/hfa_dict_dep_review`

## Findings

- Severity: low
- Location: `rewrite-in-rust/dependencies/hfa_dictionary_g2p_core.yaml:2`
- Issue: The dependency record uses `status: reimplemented`, which is not a
  valid dependency-record state. The dependency-record schema allows only
  `planned`, `active`, `done`, or `blocked`, and every other completed current
  HFA dependency record uses `done`. The content proves completed dependency
  expansion, but its status does not durably close that phase.
- Evidence: `rewrite-in-rust/dependencies/README.md:16` defines the allowed
  states. A repository status audit found 51 `done`, 6 `planned`, 1 `blocked`,
  and only this one `reimplemented` dependency record. The manifest correctly
  uses its separate unit state vocabulary and remains `status: reimplemented`
  at `rewrite-in-rust/manifest.yaml:1450`; record 0077 describes the same unit
  implementation state at
  `rewrite-in-rust/records/0077-implement-hfa-dictionary-g2p-core.md:45`.
- Required fix: Change only the dependency record's status to `done`. Keep the
  manifest at `reimplemented`, the inventory boundary `confirmed`, and runtime
  ownership `legacy` until all required independent reviews pass.

No critical, high, or medium dependency/bootstrap findings were found.

## Boundary Decision

The manifest boundary remains **confirmed**. It should not be split, merged,
deferred, or replaced.

Record 0074 correctly separated dictionary-backed G2P from the pure
phoneme/mora converter, YAML config parsing, HTK export, TextGrid serialization,
and export dispatch (`rewrite-in-rust/records/0074-split-hfa-g2p-config-export-lifecycle.md:16`).
The selected legacy implementation is one cohesive object: construction reads
and parses a dictionary snapshot, conversion performs lookup and ordered
warnings, and the inherited base call applies SP assertions and language
prefixing (`inference/HubertFA/tools/g2p.py:177`). Dataset discovery, dictionary
path selection, lab IO, and exception wrapping occur in the caller at
`inference/HubertFA/tools/infer_base.py:155` and remain outside this unit.

The current Rust implementation stays inside that boundary. It extends the
verified `v2m-core::hfa_g2p` module and reuses `HfaG2pOutput`, `HfaG2pError`, and
`apply_base_g2p_contract` rather than introducing a parallel output contract
(`rewrite-in-rust/rust/crates/v2m-core/src/hfa_g2p.rs:23`, line 270, and line
368). `HfaDictionaryG2p::from_path` owns only one-time open/read/UTF-8/newline
normalization and parsing; `convert` owns immutable lookup, indexes, warnings,
and the shared base projection (lines 277 through 377). No dataset, model,
config, export, bridge, or routing behavior entered the module.

## Dependency And Seam Audit

Capability alignment is complete for the confirmed seam. The dependency record
maps the four required capabilities to legacy and Rust owners: Linux UTF-8
snapshot loading, row parsing, lookup/index construction, and ordered warning
policy (`rewrite-in-rust/dependencies/hfa_dictionary_g2p_core.yaml:3`). The
bootstrap fixes the supported environment to Python 3.12/Linux/UTF-8 and records
the exact whole-file strip, tab fields, literal-space tokens, duplicate
replacement, warning order, shared assertions, repeated-call behavior, and
rollback (`rewrite-in-rust/bootstrap/hfa_dictionary_g2p_core.md:16`).

The hand-written replacement choice remains justified. Legacy `DictionaryG2P`
uses only `pathlib`, built-in text IO, and `warnings`; the selected Rust
production path imports only `std` filesystem, IO, collections, path, error,
formatting, and UTF-8 types. Cargo manifest and lockfile diffs are empty.
`serde_json` and `md-5` were already workspace dependencies and are used only by
the shared module's `#[cfg(test)]` harness. No G2P crate, Unicode crate, PyO3,
subprocess, HTTP, native/FFI, model asset, runtime router, or unsafe code was
added.

The seam remains an independent Rust library with `bridge_dependencies: []`
and `default_owner: legacy`
(`rewrite-in-rust/dependencies/hfa_dictionary_g2p_core.yaml:20`). Static routing
search found no production caller of `HfaDictionaryG2p`; Python callers still
construct `DictionaryG2P` in `InferenceBase.get_dataset`, and `hfa_api.py` still
selects that legacy route. The manifest preserves `current_owner: legacy` and
an explicit Python rollback at `rewrite-in-rust/manifest.yaml:1452` and line
1478.

## Fixture Audit

The JSONL artifact contains 30 strict JSON cases with 30 unique non-empty case
IDs and an `expect` object on every row. It contains 23 successful constructor
cases, 7 constructor-error cases, 26 conversion calls, 2 shared assertion
errors, and 19 ordered warnings. The cases cover the dependency record's
required parser maps, duplicate and extra-tab behavior, Python stripping,
literal-space empty tokens, CRLF/lone-CR translation, missing and edge-SP
warnings, nullable/empty/string language, malformed/empty/invalid-UTF-8 and
filesystem paths, repeated calls, file snapshotting, and recovery after a
conversion error (`rewrite-in-rust/fixtures/hfa_dictionary_g2p_core.jsonl:1`).

The Python checker imports the real legacy class, constructs real temporary
files, asserts Python 3.12 plus the UTF-8 default encoding, records warnings,
normalizes only the temporary root in errors, and compares every result to the
shared table (`rewrite-in-rust/bootstrap/check_hfa_dictionary_g2p_core.py:18`,
line 58, and line 97). Rust consumes the same JSONL rather than a translated
fixture and supplements it with structured load-context, repeat/recovery, and
10,000-entry/input scaling tests
(`rewrite-in-rust/rust/crates/v2m-core/src/hfa_g2p.rs:1026`, line 1103, line
1160, and line 1185).

## Checks

- Structured YAML/JSONL audit: passed. Manifest state is `reimplemented`,
  inventory is `confirmed`, current owner is `legacy`, dependency boundary is
  `confirmed`, bridge dependencies are empty, and all 30 fixture IDs are unique.
- `env UV_CACHE_DIR=/tmp/uv-cache-hfa-dictionary-dep-review uv run python rewrite-in-rust/bootstrap/check_hfa_dictionary_g2p_core.py`:
  passed; validated all 30 legacy fixtures.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_dictionary_g2p_core -- --nocapture`:
  passed all 4 focused tests.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_phoneme_mora_g2p_core -- --nocapture`:
  passed all 3 prerequisite shared-contract tests.
- `env UV_CACHE_DIR=/tmp/uv-cache-hfa-dictionary-dep-review uv run python scripts/audit_vendored_sources.py`:
  passed; 135 Python packages, 41 native-extension packages, 269 foreign-runtime
  native binaries, and 0 third-party binary artifacts.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml`: passed 112
  `v2m-core` tests, 5 bridge tests, and doc tests.
- `cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check`:
  passed.
- `cargo clippy --manifest-path rewrite-in-rust/rust/Cargo.toml --all-targets --all-features -- -D warnings`:
  passed.
- `RUSTDOCFLAGS='-D warnings' cargo doc --manifest-path rewrite-in-rust/rust/Cargo.toml --no-deps`:
  passed.
- `env UV_CACHE_DIR=/tmp/uv-cache-hfa-dictionary-dep-review uv run python -m py_compile inference/HubertFA/tools/g2p.py`:
  passed.
- Cargo dependency diff and static production-routing audit: passed; no Cargo
  manifest/lock change, new crate, bridge, or runtime route was found.
- `git diff --check`: passed before this report was written.

## Residual Risk

This role does not replace the required behavior or error/tracing reviews. The
contract is deliberately pinned to Python 3.12 on Linux with a UTF-8 locale;
non-UTF-8 platforms, broader OS error variants, and full Python warning/error
reconstruction remain promotion decisions. The public fixture seam accepts
UTF-8 strings, nullable/string language, and filesystem paths rather than
arbitrary Python duck-typed objects. Dictionary iteration order is not part of
the selected conversion payload; a future bridge must not serialize the
internal Rust `HashMap` as a new compatibility promise without a separate
record and fixtures.

Python warning-filter presentation, path payload validation, caller routing,
and rollback application remain unproved and legacy-owned. These residuals do
not require another dependency, a wider unit, or a different seam.

## Promotion Note

This dependency/bootstrap review passes the implementation boundary with one
low-severity control-plane follow-up. The boundary remains confirmed. The
coordinator should not mark `hfa_dictionary_g2p_core` verified or treat this
required role as cleanly closed until the dependency record status is normalized
to `done`. After that fix, this role does not block a coordinator state update,
provided fresh independent `stage_behavior_reviewer` and
`error_tracing_reviewer` reports also pass. This report does not approve
production routing, change runtime ownership, or modify the manifest.
