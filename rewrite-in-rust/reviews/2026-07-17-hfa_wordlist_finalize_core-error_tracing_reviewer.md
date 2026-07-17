# hfa_wordlist_finalize_core - error_tracing_reviewer

Date: 2026-07-17
Decision: pass

Unit: `hfa_wordlist_finalize_core`
Role: `error_tracing_reviewer`

## Findings

No error-tracing findings.

The Rust error surface keeps the relevant failures structured and exactly
projectable. `HfaWordListErrorKind` distinguishes `ValueError`,
`AttributeError`, `IndexError`, and Rust-integrity `BorrowError`, while the
concrete kind and message remain private and the supported projections are
`exception_type()` plus `Display`
(`rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:356`). Empty-list and
empty-phoneme mutation failures become exact `IndexError: list index out of
range` values through checked lookup and `From<HfaWordMutationError>` at lines
387, 424, and 796. String-valued invalid entries become exact legacy
`AttributeError: 'str' object has no attribute '<attribute>'` values at line
380. Generated SP constructor failures retain `HfaWordError`'s exact legacy
`ValueError` message before being projected into the local compatibility log
(`rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:116`, line 653, line 673,
and line 686). The shared table proves reachable exact `IndexError`,
`AttributeError`, and constructor `ValueError` log projections at
`rewrite-in-rust/fixtures/hfa_wordlist_finalize_core.jsonl:1`, line 8, lines 13
through 24, line 44, and line 50.

Legacy broad catches are preserved for semantic failures without swallowing a
Rust integrity failure. `fill_small_gaps` catches structured index/attribute/
mutation errors as `ERROR in fill_small_gaps: ...`, but immediately returns a
`BorrowError` (`rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:539`).
`add_sp` applies the same outer rule at line 612, while each reachable
`Word::new` failure is handled at its legacy inner boundary as `ERROR: ...` at
lines 653 through 690 rather than being mislabeled `ERROR in add_SP: ...`.
This matches the legacy nested `except ValueError` and outer `except Exception`
boundaries at `inference/HubertFA/tools/align_word.py:217` and line 235. The
Rust-only integrity regression at `rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:2540`
forces all three finalizer operations through a private borrow conflict and
proves exact `BorrowError`, no compatibility log entry, and retained list
state.

Partial mutation and replacement boundaries retain diagnosable legacy state.
`fill_small_gaps_inner` performs leading, trailing, then interior work through
canonical handles, so an invalid middle entry can leave the earlier trailing
repair visible before the outer error (`rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:571`;
fixture line 44). `add_sp_inner` clones the source handles and builds a private
candidate vector, but sends validated-append warnings directly to the shared
outer log. It assigns `self.entries = candidates` only after all candidate
construction succeeds, then runs `check()` (`rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:646`,
line 694). Consequently a pre-replacement outer error retains the original
entries while preserving candidate warnings (fixture line 50), invalid
first/middle/last entries do not install partial candidates (lines 17 through
19), and reachable local constructor failures continue building, replace the
entries, and are followed by the exact check warning (lines 23 through 24).
Empty/overlap append warnings and the original-last trailing boundary are also
proved at lines 16, 21, and 45 through 47.

`check()` cleanly separates compatibility-invalid state from Rust-integrity
failure. The first business defect appends one exact warning and returns
`Ok(false)`; an empty or valid list returns `Ok(true)`
(`rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:699`). An inaccessible
canonical handle instead returns `BorrowError` from `snapshot()` without
adding a misleading warning. `add_sp_inner` deliberately ignores only the
boolean result at line 695; it still propagates `BorrowError` with `?`.
Fixture lines 25 through 39 prove false/true results, first-failure order, and
repeated warnings, while the Rust integrity test proves the distinct error
path.

The current safe public finalizer API cannot manufacture a borrow conflict or
retain a borrow guard. `WordHandle` keeps `Rc<RefCell<Word>>` private and exposes
only owned snapshots, identity comparison, and closed fallible mutations; its
private accessors use `try_borrow` and `try_borrow_mut`
(`rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:251`). `entries()` can
expose cloned handles but not their `Rc`, `RefCell`, `Ref`, or `RefMut`. The
forced-conflict test is possible only because the in-module test can access the
private field. Finalizer empty and invalid positions use checked `get`, check's
phoneme indexing is guarded by the prior empty test, and the generated
full-span phoneme `expect` in `Word::new` follows the already-proved strict
word interval (`rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:116`). No
safe representable finalizer input exposes a panic path; `BorrowError` remains
a defensive integrity channel for future internal changes.

Diagnostics are an ordered, caller-readable in-memory buffer. They include
caller-controlled Word, phoneme, and custom SP text through append warnings,
constructor errors, and final-check warnings, but this Rust unit only stores,
joins, and clears the strings (`rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:470`).
Static inspection found no stdout/stderr, file, tracing, logging-framework, or
telemetry sink and no added path, token, credential, environment, model, or
audio context. A future bridge must classify where lyric-bearing diagnostics
may be displayed or retained; redacting the current in-memory value would break
the exact compatibility contract, while forwarding it to external telemetry
would expose user content.

No production route was introduced. `inference/HubertFA/tools/decoder.py:5`
and `inference/HubertFA/tools/infer_base.py:10` still import the legacy Python
types, and the finalizer calls at `inference/HubertFA/tools/infer_base.py:233`
remain Python methods. The manifest remains `status: reimplemented` and
`current_owner: legacy` at `rewrite-in-rust/manifest.yaml:1368`. Writer/reviewer
separation is intact: this review did not modify production code, fixtures,
checker, bootstrap/dependency artifacts, the manifest, or records.

## Checks

- `env UV_CACHE_DIR=/tmp/uv-cache-hfa-finalizer-error uv run python rewrite-in-rust/bootstrap/check_hfa_wordlist_finalize_core.py`: passed all 53 legacy fixture rows (exit 0; silent on success).
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_wordlist_finalize -- --nocapture`: passed; both the 53-row parity test and structured-integrity test passed, with 102 `v2m-core` tests filtered out.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_wordlist_finalize_preserves_structured_integrity_errors -- --nocapture`: passed; forced `fill`, `add_sp`, and `check` borrow conflicts all projected exact `BorrowError`, left the log empty, and retained the entry.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_wordlist_collection -- --nocapture`: passed the 45-row canonical collection prerequisite, with 103 `v2m-core` tests filtered out.
- Targeted legacy error/state probe: passed. Empty fill logged exact outer `IndexError`; invalid-middle fill retained the trailing mutation before exact `AttributeError`; candidate append warning survived a later outer `IndexError` while original entries remained; local SP construction logged `ERROR: Word Invalid: ...` and continued to the check warning; invalid-entry `check()` returned `false` with one warning.
- Static borrow/panic/sink inspection over finalizer-facing `hfa_word.rs`: inspected. Production handle access uses only private `try_borrow`/`try_borrow_mut`; no finalizer-facing stdout, file, tracing, logging-framework, telemetry, or unguarded index panic surface was found.
- Static routing search: inspected. Only legacy Python `WordList` is imported and called by decoder/inference production paths; no Rust bridge or owner switch exists.
- `cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check`: passed.
- `git diff --check`: passed after this report was written.

## Residual Risk

The exact invalid-object projection is fixture-bound to string-valued entries.
A production bridge must define conversion and exception text for arbitrary
Python objects instead of assuming every invalid value should report `'str'
object ...`. It must also preserve the distinction among caught semantic log
lines, `check()`'s warning plus false result, and propagated Rust integrity
errors.

The current public API deliberately makes borrow conflicts unreachable, so the
forced `BorrowError` evidence is an in-module integrity test rather than a
public reproduction. Any future callback, exposed guard, reentrant bridge, or
internal operation that keeps a borrow alive across finalization must add a
public-boundary regression, preserve propagation, and specify whether entry
replacement may already have occurred.

There is no production bridge. Python warning presentation, arbitrary exception
traceback/context mapping, Python-to-Rust alias ownership through
`hfa_api.py`'s later clear/extend repair, routing, and rollback remain outside
this library review. Exact logs can carry user lyric/phoneme content and require
an explicit display, retention, and telemetry policy before production use.

## Promotion Note

This error-tracing role does not block coordinator state update for the current
independent, legacy-owned library seam. The coordinator may record
`error_tracing_reviewer` as passed after separately resolving every other
required review. This report does not approve production routing and does not
mark or modify the manifest.
