# hfa_wordlist_finalize_core - dependency_bootstrap_reviewer rerun

Date: 2026-07-17
Decision: pass

Unit: `hfa_wordlist_finalize_core`
Role: `dependency_bootstrap_reviewer`

## Findings

No dependency/bootstrap findings.

The two follow-ups from the prior `pass-with-followups` report are closed:

- `rewrite-in-rust/dependencies/hfa_wordlist_finalize_core.yaml:2` now reports
  `status: done`. Its capability entries point to the implemented
  `WordList::fill_small_gaps`, `fill_small_gaps_default`, `add_sp`,
  `add_sp_default`, and `check` methods at dependency-record lines 6, 10, and
  14, and its verification list names the working finalizer and prerequisite
  Rust test filters at lines 55 and 56.
- `rewrite-in-rust/bootstrap/hfa_wordlist_finalize_core.md:109` now accurately
  states that the leading SP constructor failure is mathematically unreachable
  over accepted numeric inputs and that no shared checker hook, parity case, or
  Rust production API introduces fault injection. The monkeypatched `check()`
  path is explicitly outside the accepted seam at line 113. Static inspection
  confirms the checker's only optional mode remains `--generate` at
  `rewrite-in-rust/bootstrap/check_hfa_wordlist_finalize_core.py:242`.

## Boundary And Prerequisites

The manifest boundary remains **confirmed**. It should not be split, merged,
deferred, or replaced. The caller lifecycle still places this unit after
aggregation: `inference/HubertFA/tools/infer_base.py:217` reconstructs aggregate
Words, then calls `fill_small_gaps`, `add_SP`, and `log` at lines 233 through
237. Decoder/pre-aggregation collection, AP policy, prefix clearing,
multi-pass aggregation, audio/model/export behavior, and warning presentation
remain legacy-owned.

The expanded `inference/API/hfa_api.py` source reference remains necessary and
does not broaden the unit. Its post-infer repair mutates the same Word objects at
line 67 and performs list `clear()`/`extend()` at lines 77 and 78. Rust
`entries()`, `clear_entries`, and `raw_extend` at
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:455` through line 467 can
express the list replacement over retained handles without migrating repair
selection. Cross-language alias ownership remains a promotion concern, not a
new finalizer dependency.

Both ordered prerequisites remain verified:
`hfa_word_model_core` at `rewrite-in-rust/manifest.yaml:1304` and
`hfa_wordlist_collection_ap_core` at line 1336. The finalizer still extends the
same canonical safe `WordHandle`, heterogeneous `WordListEntry`, `WordList`
entry vector, and persistent ordered log at
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:259`, line 338, and line
432. No parallel handle, entry, interval, collection, or log representation was
found.

## Dependency And Fixture Audit

The seam remains an independent Rust library with legacy default ownership and
an empty `bridge_dependencies` list at
`rewrite-in-rust/dependencies/hfa_wordlist_finalize_core.yaml:20`. Cargo
manifest and lockfile diffs are empty. Production HFA code uses only `std`; no
new crate, PyO3, subprocess, CLI, HTTP, runtime router, NumPy, librosa,
TextGrid, ONNX Runtime, model, GUI, Web, or production bridge dependency was
introduced. Static routing search found no production Python caller of the Rust
module.

The shared fixture remains 53 rows with 53 unique case IDs and an `expect`
object on every row: 19 `fill_small_gaps`, 19 `add_SP`, 14 `check`, and one
`clear_extend_check`. The post-fix `fill_interior_touch_equal_above` row at
`rewrite-in-rust/fixtures/hfa_wordlist_finalize_core.jsonl:7` now uses an exact
binary `0.125` equality gap and a distinct `0.25` above-threshold gap. Its
Python golden was regenerated without changing table size. The legacy checker
reads that table at `rewrite-in-rust/bootstrap/check_hfa_wordlist_finalize_core.py:234`,
and the Rust test consumes the same artifact and asserts 53 cases at
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:2521`.

The manifest remains `status: reimplemented`, `current_owner: legacy`, and has
an explicit Python finalizer rollback at `rewrite-in-rust/manifest.yaml:1370`
and line 1401. No runtime owner or route changed.

## Checks

- `UV_CACHE_DIR=/tmp/uv-cache-hfa-finalizer-dep-rerun uv run python --version`:
  passed; Python 3.12.13.
- `uv run python rewrite-in-rust/bootstrap/check_hfa_word_model_core.py`:
  passed all 14 prerequisite cases.
- `uv run python rewrite-in-rust/bootstrap/check_hfa_wordlist_collection_ap_core.py`:
  passed all 45 prerequisite cases.
- `uv run python rewrite-in-rust/bootstrap/check_hfa_wordlist_finalize_core.py`:
  passed all 53 finalizer cases; silent exit 0.
- Structured YAML/JSONL audit: passed; four YAML files parsed, dependency status
  is `done`, all 53 fixture IDs are unique and have `expect`, and the equality
  row has exact `0.125` equal and `0.25` above gaps.
- `uv run python scripts/audit_vendored_sources.py`: passed; 135 Python
  packages, 41 native-extension packages, 269 foreign-runtime native binaries,
  and zero `third_party` binary artifacts.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_wordlist_finalize -- --nocapture`:
  passed 3 focused tests, including the shared table, structured errors, and
  10,000-entry scaling/short-circuit regression.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_wordlist_collection -- --nocapture`:
  passed the prerequisite collection test.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml`: passed 105
  `v2m-core` tests, 5 bridge tests, and doc tests.
- `cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check`:
  passed.
- `cargo clippy --manifest-path rewrite-in-rust/rust/Cargo.toml --all-targets --all-features -- -D warnings`:
  passed.
- `RUSTDOCFLAGS='-D warnings' cargo doc --manifest-path rewrite-in-rust/rust/Cargo.toml --no-deps`:
  passed.
- `uv run pytest tests/test_web_api.py -q -p no:cacheprovider`: passed, 53
  tests.
- `git diff --check`: passed before this report was written.
- Cargo dependency diff and static production-routing audit: passed; no crate,
  lockfile, bridge, or runtime-route creep.

## Residual Risk

The compatibility contract remains pinned to CPython 3.12.13 and an `f64` /
string fixture payload. Arbitrary Python invalid objects, warning-filter
presentation, Python/Rust alias lifetime across `hfa_api.py`, and bridge payload
mapping remain unproved. `WordHandle` is deliberately single-threaded, and
`clear_entries`/`raw_extend` prove list replacement mechanics only. A future
owner switch must either preserve Python object identity through repair or keep
that repair wholly legacy-owned.

## Promotion Note

This dependency/bootstrap rerun passes cleanly and no longer blocks coordinator
verification of `hfa_wordlist_finalize_core`. The coordinator may use this
rerun, rather than the historical `pass-with-followups` report, as the required
dependency/bootstrap evidence. Other review roles remain independent gates.
This report does not update the manifest or approve a production owner switch.
