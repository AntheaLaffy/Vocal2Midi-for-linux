# hfa_dictionary_g2p_core - dependency_bootstrap_reviewer rerun

Date: 2026-07-17
Decision: pass

Unit: `hfa_dictionary_g2p_core`
Role: `dependency_bootstrap_reviewer`
Writer: `/root/hfa_dictionary_writer`
Reviewer: `/root/hfa_dict_dep_review`

## Findings

No dependency/bootstrap findings.

The initial review's low-severity control-plane finding is closed. The dependency
record now uses `status: done` at
`rewrite-in-rust/dependencies/hfa_dictionary_g2p_core.yaml:2`, which is one of
the dependency schema's allowed states at `rewrite-in-rust/dependencies/README.md:16`.
The manifest correctly retains its separate migration state:
`status: reimplemented`, `inventory_status: confirmed`, and
`current_owner: legacy` at `rewrite-in-rust/manifest.yaml:1450`. Record 0077
also distinguishes the completed dependency record from the unverified manifest
unit at `rewrite-in-rust/records/0077-implement-hfa-dictionary-g2p-core.md:62`.

## Boundary Decision

The manifest boundary remains **confirmed**. It should not be split, merged,
deferred, or replaced.

The 13 added cases close constructor error-projection gaps inside the existing
dictionary snapshot seam. Eight additions expand invalid UTF-8 coverage and five
expand missing-path filename representation coverage; neither set introduces a
new lifecycle phase. `DictionaryG2P.__init__` remains the legacy owner of text
open/read/decode and dictionary parsing at `inference/HubertFA/tools/g2p.py:177`,
while `InferenceBase.get_dataset` still owns path selection, wav/lab discovery,
dataset mutation, and caller error handling at
`inference/HubertFA/tools/infer_base.py:155`.

The Rust changes stay in the same independent `v2m-core::hfa_g2p` object.
`HfaDictionaryG2p::from_path` retains one-time filesystem snapshot and UTF-8
decode state, `parse_dictionary` retains row parsing, and `convert` retains
lookup, warning, word-index, and shared Base-contract behavior
(`rewrite-in-rust/rust/crates/v2m-core/src/hfa_g2p.rs:273`, line 330, and line
693). Explicit decode `start`/exclusive-`end` fields and filename representation
are refinements of the already-selected structured constructor error payload,
not new config, model, export, presentation, or bridge responsibilities.

## Dependency And Seam Audit

Capability coverage remains complete and internally consistent. The dependency
record maps Linux UTF-8 dictionary snapshotting, row parsing, lookup/index
construction, and ordered warning policy to concrete legacy and Rust owners at
`rewrite-in-rust/dependencies/hfa_dictionary_g2p_core.yaml:3`. Its error mapping
now names Python 3.12/Unicode 15 filename repr and explicit decode
start/exclusive-end/leading-byte/reason fields at line 26. Bootstrap records the
same supported platform, representation policy, decode span semantics,
43-case harness, repeated-call behavior, exclusions, and rollback at
`rewrite-in-rust/bootstrap/hfa_dictionary_g2p_core.md:16`.

The expanded diagnostic compatibility does not require a new dependency. Rust
still uses standard-library filesystem, IO, UTF-8, collections, path, error, and
formatting APIs for production dictionary work. Filename rendering reuses the
crate-private `python_15_nonprintable::string_repr` at
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_g2p.rs:746`; the helper remains
`pub(crate)` and was already the verified HFA Word representation primitive at
`rewrite-in-rust/rust/crates/v2m-core/src/python_15_nonprintable.rs:738` and
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:1607`. Record 0077
explicitly documents this reuse at line 58. No parallel repr table, external
Unicode crate, G2P crate, generated runtime dependency, native/FFI code, unsafe
code, PyO3, subprocess, HTTP, or runtime router was introduced.

Cargo workspace, lockfile, and `v2m-core` manifest diffs remain empty. The seam
still declares `bridge_dependencies: []` and `default_owner: legacy` at
`rewrite-in-rust/dependencies/hfa_dictionary_g2p_core.yaml:20`. Static routing
search found no production caller of `HfaDictionaryG2p`; Python
`InferenceBase.get_dataset` and `hfa_api.py` still select and invoke legacy
`DictionaryG2P`. Manifest rollback remains keeping both Python dictionary
selection and conversion as runtime owners at
`rewrite-in-rust/manifest.yaml:1479`.

## Fixture Audit

The shared JSONL contains 43 strict JSON cases with 43 unique non-empty case IDs
and an `expect` object on every row. The distribution is 23 successful
constructors, 4 malformed/empty `IndexError` cases, 9 `UnicodeDecodeError`
cases, 6 missing-path `FileNotFoundError` cases, and 1 directory
`IsADirectoryError` case. Successful instances execute 26 conversions with 19
ordered warnings and 2 shared assertion errors.

The nine byte fixtures distinguish invalid start, continuation start, overlong,
surrogate, out-of-range, consumed multi-byte invalid continuation, and truncated
three-/four-byte spans. The six missing-path fixtures include the plain path and
five custom names covering apostrophe quote selection, newline, tab, backslash,
and printable non-ASCII. Those cases appear contiguously at
`rewrite-in-rust/fixtures/hfa_dictionary_g2p_core.jsonl:25` through line 39 and
match the capability list in the dependency record at line 31.

The Python checker still imports the real legacy class, creates actual temporary
files, asserts Python 3.12 plus the Linux UTF-8 default encoding, permits an
optional `path_name`, and compares exact normalized exceptions to the shared
table (`rewrite-in-rust/bootstrap/check_hfa_dictionary_g2p_core.py:18`, line 58,
and line 97). Rust consumes that same JSONL and now constructs the same custom
filenames at `rewrite-in-rust/rust/crates/v2m-core/src/hfa_g2p.rs:1025`. Focused
structured tests separately assert singular and multi-byte invalid-continuation
spans plus truncated-end retention at lines 1107 through 1179. The established
repeat/recovery and 10,000-entry/input regression tests remain intact.

## Checks

- Structured YAML/JSONL audit: passed. Dependency state is `done`, boundary is
  `confirmed`, bridge dependencies are empty, manifest state remains
  `reimplemented/confirmed/legacy`, all 43 case IDs are unique, and every case
  has an expected result.
- Fixture distribution audit: passed; 9 byte cases, 5 custom path names, 23
  successful constructors, 20 constructor errors, 26 conversion calls, and 19
  warnings.
- `env UV_CACHE_DIR=/tmp/uv-cache-hfa-dictionary-dep-rerun uv run python rewrite-in-rust/bootstrap/check_hfa_dictionary_g2p_core.py`:
  passed; validated all 43 legacy fixtures.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_dictionary_g2p_core -- --nocapture`:
  passed all 4 focused tests.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_phoneme_mora_g2p_core -- --nocapture`:
  passed all 3 prerequisite shared-contract tests.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml python_15_string_repr_covers_quotes_controls_and_unicode -- --nocapture`:
  passed the shared Python 3.12/Unicode 15 repr test.
- `env UV_CACHE_DIR=/tmp/uv-cache-hfa-dictionary-dep-rerun uv run python scripts/audit_vendored_sources.py`:
  passed; 135 Python packages, 41 native-extension packages, 269 foreign-runtime
  native binaries, and 0 third-party binary artifacts.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml`: passed 113
  `v2m-core` tests, 5 bridge tests, and doc tests.
- `cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check`:
  passed.
- `cargo clippy --manifest-path rewrite-in-rust/rust/Cargo.toml --all-targets --all-features -- -D warnings`:
  passed.
- `RUSTDOCFLAGS='-D warnings' cargo doc --manifest-path rewrite-in-rust/rust/Cargo.toml --no-deps`:
  passed.
- `env UV_CACHE_DIR=/tmp/uv-cache-hfa-dictionary-dep-rerun uv run python -m py_compile inference/HubertFA/tools/g2p.py`:
  passed.
- Cargo dependency diff and static production-routing audit: passed; no Cargo
  manifest/lock change, new crate, bridge, or runtime route was found.
- `git diff --check`: passed before this report was written.

## Residual Risk

This role does not substitute for the required fresh behavior and error/tracing
reruns. The compatibility policy remains deliberately limited to Python 3.12 on
Linux with a UTF-8 locale and UTF-8-representable paths. Non-UTF-8 platform
defaults, non-UTF-8 Unix path bytes, broader OS error variants, Python warning
filter/presentation semantics, and bridge-side exception reconstruction remain
promotion decisions. The structured original `PathBuf`, `io::Error`, and
`Utf8Error` are retained so those future policies do not need to parse the
compatibility message.

Dictionary iteration order is not part of the selected conversion payload. A
future bridge must not serialize the internal Rust `HashMap` as a new
compatibility promise without a separate record and fixtures. These residuals
do not require another dependency, a wider unit, or a different seam.

## Promotion Note

This clean dependency/bootstrap rerun supersedes the initial
`pass-with-followups` report for gate status while preserving it as historical
evidence. The boundary remains confirmed, and this role no longer blocks the
coordinator from updating `hfa_dictionary_g2p_core` state. The coordinator may
treat `dependency_bootstrap_reviewer` as passed, but should mark the unit
`verified` only after fresh independent `stage_behavior_reviewer` and
`error_tracing_reviewer` reruns also pass. This report does not approve
production routing, change runtime ownership, or modify the manifest.
