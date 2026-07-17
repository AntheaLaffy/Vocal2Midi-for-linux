# hfa_phoneme_mora_g2p_core - dependency_bootstrap_reviewer rerun

Date: 2026-07-17
Decision: pass

Unit: `hfa_phoneme_mora_g2p_core`
Role: `dependency_bootstrap_reviewer`

## Findings

No dependency/bootstrap findings. The Unicode 15 compatibility fix closes the
fixture risk recorded by the initial dependency review's residual section
without changing the confirmed unit boundary.

The post-fix lowercase behavior belongs inside the existing pure G2P seam.
Legacy `JapanesePhonemeMoraG2P` applies Python 3.12 `str.lower()` to accepted
UTF-8 mora tokens at `inference/HubertFA/tools/g2p.py:103`; unknown/case-token
behavior was already part of this unit's manifest policy. Pinning that operation
to Python 3.12's Unicode 15 tables therefore refines an existing deterministic
string capability. It does not add dictionary lookup, a Japanese frontend,
file/model access, warning presentation, config, export, or caller routing.
The six-unit re-cut from record 0074 remains valid, and
`hfa_phoneme_mora_g2p_core` remains the minimum first pure route rather than a
candidate for another split or merge.

The implementation remains a local, standard-library production seam.
`python_15_lowercase` scans the input string, delegates ordinary chunks to
Rust's contextual lowercase implementation, and preserves characters whose
case mappings were introduced after Unicode 15
(`rewrite-in-rust/rust/crates/v2m-core/src/hfa_g2p.rs:377`). Its compatibility
predicate is a closed local table at line 391: eight explicit scalar values,
U+10D50..U+10D65 (22 values), and U+16EA0..U+16EB8 (25 values), for exactly 55
members. Production `hfa_g2p.rs` still imports only `std::error` and `std::fmt`.
No Unicode crate, generated runtime dependency, FFI, filesystem, model asset,
network, subprocess, PyO3, or unsafe code was added.

The 55-member set and the complete mapping surface now have durable,
independent references. The historical failed behavior/data reports preserve
the Python-15 versus Rust-17 discovery and representative public mismatches.
Record 0075 records the 55-scalar fix and all-scalar requirement at
`rewrite-in-rust/records/0075-implement-hfa-phoneme-mora-g2p-core.md:26`.
The source table expresses the complete compatibility set directly, while the
legacy checker independently hashes Python's actual lowercase output for every
valid scalar from U+0000 through U+10FFFF, excluding surrogates
(`rewrite-in-rust/bootstrap/check_hfa_phoneme_mora_g2p_core.py:52`). It includes
the code point bytes, UTF-8 lowercase bytes, and a delimiter in the digest, and
reports `unicodedata.unidata_version` plus scalar count.

Fixture line 22 pins that reference to Unicode `15.0.0`, 1,112,064 valid scalar
values, and MD5 `463756413147af7de3cf822b56a336b1`. The Rust test independently
iterates the same scalar domain and hashes `python_15_lowercase` into the same
shape at `rewrite-in-rust/rust/crates/v2m-core/src/hfa_g2p.rs:463`. Thus the
four-group examples are not being used as a substitute for complete coverage:
any missing/extra compatibility member or later toolchain casing drift changes
the full digest. MD5 is only a deterministic fixture checksum; Python explicitly
sets `usedforsecurity=False`, and Rust reuses the workspace's existing `md-5`
test dependency.

The expanded 28-row table is adequate and remains genuinely cross-runtime.
The former 25 cases are retained, line 20 exercises representatives from all
four newer-mapping groups through the public mora API, line 21 retains a
context-sensitive Greek final-sigma result, and line 22 is the exhaustive scalar
digest. The resulting distribution is 9 raw-phoneme, 12 mora, 6 Base-contract,
and 1 digest case. The checker imports the real legacy G2P classes and computes
the digest using only Python standard-library `hashlib` and `unicodedata`; Rust
consumes the same JSONL. The focused gate also retains repeated-call and
10,000-token regression tests.

Control-plane and dependency facts match the implementation. The dependency
record says `status: done`, names 28 rows and the 1,112,064-scalar contract, and
keeps `bridge_dependencies: []`
(`rewrite-in-rust/dependencies/hfa_phoneme_mora_g2p_core.yaml:1`). Bootstrap
describes the same 55 values, contextual-lowercase preservation, and full digest
at `rewrite-in-rust/bootstrap/hfa_phoneme_mora_g2p_core.md:40`. Record 0075 and
manifest verification both state 28 cases and require independent reruns. The
manifest remains `status: reimplemented`, `inventory_status: confirmed`, and
`current_owner: legacy` at `rewrite-in-rust/manifest.yaml:1409`; rollback still
keeps the three Python classes as runtime owners.

No crate, bridge, or routing change accompanies the fix. Cargo manifest and
lockfile diffs are empty; `md-5` and `serde_json` were already present for test
fixtures. Static routing search found no production caller of `hfa_g2p`; Python
`InferenceBase.get_dataset` and `hfa_api.py` still select and invoke the legacy
classes. The vendored-source audit remains clean, and the Unicode fix needs no
new third-party source because its executable reference is the pinned project
Python runtime itself. Writer/reviewer separation is intact: this rerun changes
no production code, fixture, checker, dependency/bootstrap artifact, record,
control-plane file, or manifest and adds only this report.

## Checks

- Structured Unicode/YAML/JSONL audit: passed. The Rust compatibility predicate expands to exactly 55 scalars: eight singletons, U+10D50..U+10D65, and U+16EA0..U+16EB8; all 28 fixture IDs are unique; the digest row exactly pins Unicode 15.0.0, 1,112,064 scalars, and MD5 `463756413147af7de3cf822b56a336b1`; dependency status/bridge and manifest owner/status facts match.
- `env UV_CACHE_DIR=/tmp/uv-cache-hfa-g2p-dep-rerun uv run python -c 'import sys, unicodedata; ...'`: Python 3.12.13, Unicode 15.0.0.
- `env UV_CACHE_DIR=/tmp/uv-cache-hfa-g2p-dep-rerun uv run python rewrite-in-rust/bootstrap/check_hfa_phoneme_mora_g2p_core.py`: validated all 28 legacy fixtures, including the complete scalar digest, plus repeated-call stability.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_phoneme_mora_g2p_core -- --nocapture`: passed all 3 focused tests; the shared-table test recomputed the complete Rust digest.
- `env UV_CACHE_DIR=/tmp/uv-cache-hfa-g2p-dep-rerun uv run python scripts/audit_vendored_sources.py`: passed; 135 Python packages, 41 native-extension packages, 269 foreign-runtime native binaries, and 0 third-party binary artifacts.
- Cargo dependency/static routing audit: passed. No Cargo manifest or lockfile diff, no new crate, no bridge dependency, and no production Rust G2P caller was found.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml`: passed 108 `v2m-core` tests, 5 bridge tests, and doc tests.
- `cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check`: passed.
- `cargo clippy --manifest-path rewrite-in-rust/rust/Cargo.toml --all-targets --all-features -- -D warnings`: passed.
- `RUSTDOCFLAGS='-D warnings' cargo doc --manifest-path rewrite-in-rust/rust/Cargo.toml --no-deps`: passed.
- `git diff --check`: passed after this report was written.

## Residual Risk

The digest exhausts single-scalar lowercase mappings, while contextual behavior
over arbitrary multi-scalar strings is not exhaustive. The retained Greek final
sigma case proves the critical chunking property exercised by the fix, but the
independent behavior/data reruns must still decide whether broader contextual
corpora are required. This is an algorithm/behavior risk, not evidence for a
new dependency or a wider seam.

The compatibility list is intentionally tied to the active Rust Unicode table
versus Python 3.12 Unicode 15. A future Rust toolchain that adds more lowercase
mappings will fail the all-scalar digest and require updating the local preserved
set or adopting another reviewed Unicode-15 implementation. That maintenance
contract is now explicit and deterministic.

Dictionary files/warnings, dataset/lab IO, model execution, config/export,
warning presentation, payload/error transport, production routing, and rollback
application remain legacy-owned. A future bridge must preserve nullable/empty
language, UTF-8 input, ordered arrays, signed indexes, and exact Base errors.

## Promotion Note

This dependency/bootstrap rerun passes and does not block coordinator
verification of `hfa_phoneme_mora_g2p_core`. The boundary remains confirmed as
the first pure unit of the six-unit re-cut. The coordinator must use fresh
passing behavior and data/algorithm reruns to supersede their historical failed
reports before marking the unit verified. This report does not approve
production routing or modify runtime ownership or the manifest.
