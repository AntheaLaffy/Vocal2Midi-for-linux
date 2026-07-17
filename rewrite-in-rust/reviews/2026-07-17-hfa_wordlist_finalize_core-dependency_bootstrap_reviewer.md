# hfa_wordlist_finalize_core - dependency_bootstrap_reviewer

Date: 2026-07-17
Decision: pass-with-followups

Unit: `hfa_wordlist_finalize_core`
Role: `dependency_bootstrap_reviewer`

## Findings

No critical, high, or medium dependency/bootstrap findings.

- Severity: low
- Location: `rewrite-in-rust/dependencies/hfa_wordlist_finalize_core.yaml:2`
- Issue: The dependency record still declares `status: planned`, and all four
  Rust capability descriptions still say `planned`, while the manifest is
  `status: reimplemented` and record 0072 documents the implemented API.
- Evidence: The stale declarations are at dependency-record lines 2, 6, 10,
  14, and 18. The current state is recorded at
  `rewrite-in-rust/manifest.yaml:1370`, and the implemented methods are
  documented at `rewrite-in-rust/records/0072-implement-hfa-wordlist-finalize-core.md:18`
  and present at `rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:548`,
  line 622, and line 707. Other completed HFA prerequisite records use
  `status: done`.
- Required fix: After this read-only review, normalize the dependency record to
  its completed dependency/bootstrap state and describe the existing Rust API.
  Do not change the confirmed boundary or runtime owner.

- Severity: low
- Location: `rewrite-in-rust/bootstrap/hfa_wordlist_finalize_core.md:109`
- Issue: The bootstrap says the checker retains an optional control-flow hook
  for the unreachable leading-SP constructor failure, but the checker contains
  no fault-injection or monkeypatch hook.
- Evidence: The only optional checker mode is `--generate` at
  `rewrite-in-rust/bootstrap/check_hfa_wordlist_finalize_core.py:242`.
  Record 0071 correctly calls the leading constructor failure an excluded probe
  at `rewrite-in-rust/records/0071-bootstrap-hfa-wordlist-finalize-core.md:50`,
  and record 0072 explicitly says no fault injection was introduced at
  `rewrite-in-rust/records/0072-implement-hfa-wordlist-finalize-core.md:39`.
  Over the selected `f64` seam, entering the branch requires
  `first.start > 0`, so `Word(0, first.start, ...)` necessarily has a valid
  strictly increasing interval. A replace-then-error from `check()` likewise
  requires monkeypatching and is absent from both shared harnesses.
- Required fix: Remove the claim that a hook is retained, or explicitly state
  that both fault-injection paths are documented but excluded. No fixture,
  checker, or Rust production fault-injection API is needed.

## Boundary Decision

The current manifest boundary is **confirmed**. It should not be split, merged,
deferred, or replaced. The earlier split from `hfa_word_interval_core` remains
justified by the real caller lifecycle:

- decoder/pre-aggregation collection, AP insertion, and prefix clearing occur
  before duplicate-pass selection in `inference/HubertFA/tools/infer_base.py:199`;
- aggregate `Phoneme`/`Word` boundaries are reconstructed at lines 217 through
  232;
- only then are `fill_small_gaps`, `add_SP`, and the accumulated log consumed at
  lines 233 through 237.

`inference/API/hfa_api.py` is a necessary expanded source reference, not a new
finalizer dependency. After inference it mutates the same prediction objects
with `move_end` and replaces list contents with `clear()`/`extend()` at lines 67
and 75 through 78. The repair policy remains legacy-owned. Rust
`entries()`, `clear_entries`, and `raw_extend` at
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:455` through line 467 are
sufficient to express the list replacement operation over retained handles;
they do not migrate the repair decision or solve the future Python/Rust alias
bridge.

## Dependency And State Audit

Both ordered prerequisites are verified in the manifest:
`hfa_word_model_core` at `rewrite-in-rust/manifest.yaml:1304` and
`hfa_wordlist_collection_ap_core` at line 1336. Their verified reports and
record 0070 establish the exact reusable state: canonical `Phoneme`/`Word`,
private safe `WordHandle` identity, one heterogeneous `WordListEntry` vector,
and one persistent ordered log. The finalizer extends those exact definitions
at `rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:259`, line 338, and
line 432; no parallel entry, interval, handle, or log representation was found.

The implementation remains a hand-written deterministic library replacement.
`hfa_word.rs` imports only `std` in production at lines 8 through 12; fixture
JSON support is test-only. Cargo manifest and lockfile diffs are empty. No new
crate, PyO3, subprocess, CLI, HTTP, runtime router, NumPy, librosa, TextGrid,
ONNX Runtime, model execution, GUI, Web, or production bridge dependency was
introduced. Static routing search found no production Python import or caller
of the Rust module.

The manifest remains `current_owner: legacy` at
`rewrite-in-rust/manifest.yaml:1372`. Rollback is explicit at line 1401 and in
record 0072: keep Python `WordList.fill_small_gaps`, `add_SP`, and `check` as
runtime owners. Removing the uncalled Rust finalizer restores the pre-unit
state without changing decoder, aggregation, API, export, GUI, Web, or CLI
routing.

## Fixture Audit

The JSONL table contains 53 rows, 53 unique non-empty case IDs, and an `expect`
object on every row. Its distribution is 19 `fill_small_gaps`, 19 `add_SP`, 14
`check`, and one `clear_extend_check` workflow. The Python checker compares
returns, full heterogeneous entry snapshots, source/generated identity labels,
and accumulated logs with case/field context at
`rewrite-in-rust/bootstrap/check_hfa_wordlist_finalize_core.py:152` and line
213. The Rust test consumes the same table and asserts exactly 53 cases at
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:2518`.

The case IDs and expected outputs substantiate the declared reachable branches:
ordered/partial gap mutation and strict boundaries; helper warnings and outer
errors; validated SP discard/replacement and original-last semantics; natural
interior/trailing negative-end constructor errors; pre-existing/shared logs;
source/generated/reused identity; repeated calls; every first-failure check
branch; and NaN, positive/negative infinity, and negative-zero encoding. The two
unreachable monkeypatch/fault-injection paths are not represented, as required.

The fixture contract is pinned to project Python 3.12.13
(`pyproject.toml:6`, `uv.lock:3`, `.python-version:1`) and serializes numeric
inputs as `f64`, including explicit special-float markers. That is a coherent
cross-language seam, but not a claim of parity for arbitrary Python numeric or
duck-typed objects.

## Checks

- `UV_CACHE_DIR=/tmp/uv-cache-hfa-finalizer-dep uv run python --version`:
  passed; Python 3.12.13.
- `uv run python rewrite-in-rust/bootstrap/check_hfa_word_model_core.py`:
  passed all 14 prerequisite cases.
- `uv run python rewrite-in-rust/bootstrap/check_hfa_wordlist_collection_ap_core.py`:
  passed all 45 prerequisite cases.
- `uv run python rewrite-in-rust/bootstrap/check_hfa_wordlist_finalize_core.py`:
  passed all 53 finalizer cases; silent exit 0.
- Structured YAML/JSONL audit: passed; manifest and three HFA dependency YAML
  files parsed, with 53 unique fixture IDs, all `expect` objects, and four known
  fixture kinds.
- `uv run python scripts/audit_vendored_sources.py`: passed; 135 Python
  packages, 41 native-extension packages, 269 foreign-runtime native binaries,
  and zero `third_party` binary artifacts.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_wordlist_finalize -- --nocapture`:
  passed both selected tests, including the shared 53-row table.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml`: passed 104
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

The compatibility table is deliberately tied to CPython 3.12.13 and an `f64`
payload. Python version changes require fixture regeneration and review of
special-float/error formatting. Rust still models invalid entries only as the
string-valued fixture surface, `WordHandle` remains single-threaded, and no
production bridge defines arbitrary Python object conversion, alias lifetime,
warning/error presentation, or the `hfa_api.py` post-infer repair handoff.
`clear_entries`/`raw_extend` prove list replacement mechanics only; promotion
must preserve identity across Python repair or keep that repair wholly
legacy-owned.

## Promotion Note

Dependency expansion, prerequisite coverage, seam choice, fixture capability,
and rollback all pass. The two low-severity declaration follow-ups do not
justify re-cutting the unit or adding dependencies, but the coordinator should
normalize the dependency record and bootstrap wording before treating this role
as a clean promotion artifact. Required behavior and data/algorithm reviews
remain independent gates; the existing error/tracing report is not evaluated by
this role. This report does not update the manifest or approve a production
owner switch.
