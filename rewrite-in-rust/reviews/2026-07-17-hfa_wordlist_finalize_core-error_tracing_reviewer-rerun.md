# hfa_wordlist_finalize_core - error_tracing_reviewer rerun

Date: 2026-07-17
Decision: pass

Unit: `hfa_wordlist_finalize_core`
Role: `error_tracing_reviewer`

## Findings

No error-tracing findings. The post-fix index-wise scans and two-pass `check()`
preserve the structured error, compatibility-log, and partial-state behavior
approved by the original error/tracing review.

The exception taxonomy and exact projections are unchanged.
`HfaWordListErrorKind` distinguishes `ValueError`, `AttributeError`,
`IndexError`, and Rust-integrity `BorrowError`; `exception_type()` and `Display`
remain the supported type/message projections
(`rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:356`). Checked entry
lookup returns exact `IndexError: list index out of range`, string-valued invalid
entries return exact `'str' object has no attribute '<attribute>'`, and local
Word mutation failures map to the same `IndexError` at lines 380, 387, 424, and
799. Reachable generated-SP `Word::new` failures retain the exact legacy
`ValueError` message through `HfaWordError`.

The semantic-catch versus integrity-propagation boundary is still explicit.
`fill_small_gaps` logs every non-borrow semantic failure as
`ERROR in fill_small_gaps: ...` but returns `BorrowError` without adding a
compatibility diagnostic (`rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:539`).
`add_sp` applies the same outer rule at line 610. Its reachable generated-Word
constructor failures are caught locally as `ERROR: ...` at lines 651 through
688, matching the nested legacy `except ValueError`; index and attribute
failures reach the outer `ERROR in add_SP: ...` boundary. Thus a Rust borrow
conflict is not mislabeled as a legacy semantic exception, while exact legacy
`IndexError`, `AttributeError`, and `ValueError` messages remain observable at
their proper local or outer boundary.

Removing the whole source clone strengthens invalid-first short-circuiting
without changing its error result. `fill_small_gaps_inner` fixes only the entry
count and immediately resolves entry zero before reading the last or any
interior entry (`rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:571`).
`add_sp_inner` likewise resolves entry zero before candidate construction or
tail access at line 644. `check()`'s first iteration obtains only entry zero and
returns `Ok(false)` immediately for an invalid variant at line 705. Static
inspection found no `self.entries.clone()` or production finalizer
`snapshot()` call. The 10,000-tail regression at line 2568 confirms the exact
fill/add outer errors, exact check warning, retained 10,001-entry source after
add, and successful early results for all three operations.

Candidate diagnostics and the replacement boundary remain correct after the
index-wise rewrite. `add_sp_inner` retains the original source collection while
building `candidates`, but validated append writes warnings directly to the
persistent outer log. Only after leading/interior/trailing construction
completes does line 692 assign `self.entries = candidates`; line 693 then runs
`check()` and ignores only its boolean (`?` still propagates `BorrowError`).
Fixture line 50 and the targeted legacy probe prove that an empty-candidate
warning survives the later outer `IndexError` while the original entries remain
unchanged. Fixture lines 17 through 19 prove invalid first/middle/last outer
`AttributeError` without partial replacement. Lines 23 and 24 prove that local
constructor `ValueError` is logged as `ERROR: Word Invalid: ...`, construction
continues, replacement occurs, and the subsequent check warning is retained.

The new two-pass `check()` preserves first-failure order and keeps business
warnings distinct from integrity errors. Pass one validates every entry, Word,
and phoneme in list order and returns one exact warning plus `Ok(false)` on the
first defect (`rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:705`). Only
after every internal invariant succeeds does pass two compare adjacent Word
end/start values at line 778. Fixture line 36 proves a later Word-internal empty-
phoneme failure beats an earlier cross-Word gap; fixture line 35 proves the gap
warning when all internal checks pass. Both pass-one `handle.read` and pass-two
handle reads use `?`, so borrow conflicts return structured `BorrowError`
instead of `false` or a warning. The fixed entry count cannot become stale
during this synchronous `&mut self` method: no callback or public borrow guard
can mutate entries between `len()` and indexed access.

The integrity regression remains valid against the post-fix paths. It maps
`HfaWordMutationError` to exact `IndexError`, then privately forces fill,
add-SP, and check borrow conflicts and asserts `BorrowError`, an empty log, and
one retained Word entry
(`rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:2543`). Production
`WordHandle` access still uses only private `try_borrow`/`try_borrow_mut` and
does not expose a `Ref`, `RefMut`, `Rc`, or `RefCell` at lines 251 through 324;
safe external callers cannot manufacture this conflict. The in-module test is
therefore appropriate defensive integrity evidence rather than a public fault-
injection surface.

The current 53-row table remains coherent after the equality repair. Fixture
line 7 now uses exact binary `0.125` equality and a separate `0.25` above-
threshold gap, and the same current artifact passes both the legacy checker and
Rust parity test. Dependency status is now `done` and names the implemented
APIs (`rewrite-in-rust/dependencies/hfa_wordlist_finalize_core.yaml:1`); the
bootstrap accurately states that the leading constructor failure is
mathematically unreachable and adds no checker or production fault-injection
hook (`rewrite-in-rust/bootstrap/hfa_wordlist_finalize_core.md:109`).

Diagnostics still contain caller-controlled Word, phoneme, and custom SP text,
but the Rust unit only stores, joins, and clears the in-memory buffer. Static
inspection found no stdout/stderr, file, tracing, logging-framework, or
telemetry sink and no added path, token, credential, environment, model, or
audio context. No production route was introduced:
`inference/HubertFA/tools/decoder.py:5` and
`inference/HubertFA/tools/infer_base.py:10` still import the legacy Python
types, and finalization calls at `inference/HubertFA/tools/infer_base.py:233`
remain Python-owned. This rerun modified no production code, tests, fixtures,
checker, bootstrap/dependency artifacts, records, control-plane files, or the
manifest; it adds only this report and retains the original error review as
history.

## Checks

- `env UV_CACHE_DIR=/tmp/uv-cache-hfa-finalizer-error-rerun uv run python rewrite-in-rust/bootstrap/check_hfa_wordlist_finalize_core.py`: passed all 53 current legacy rows (silent exit 0).
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_wordlist_finalize -- --nocapture`: passed all 3 focused tests: the 53-row parity table, structured integrity errors, and 10,000-entry valid/invalid-first regression.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_wordlist_finalize_preserves_structured_integrity_errors -- --nocapture`: passed the focused `IndexError` mapping and fill/add/check `BorrowError` propagation assertions.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_wordlist_finalize_scales_and_short_circuits_invalid_first -- --nocapture`: passed the 10,000-tail invalid-first and long-valid regression.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_wordlist_collection -- --nocapture`: passed the canonical 45-row collection prerequisite.
- Targeted legacy error/state probe: passed. Invalid-first fill/add produced exact outer `AttributeError` logs without replacement; a candidate empty warning survived a later outer `IndexError`; a local SP constructor failure produced `ERROR: Word Invalid: ...`, continued to replacement, and emitted the check warning; a later internal failure beat a prior Word gap, while a pure adjacency failure emitted the gap warning.
- Static post-fix inspection: passed. No whole `self.entries` clone or production finalizer Word snapshot remains; handle access uses private fallible borrows, and no external diagnostic sink was found.
- Static routing inspection: passed. Production decoder/inference paths still import and invoke only the legacy Python `WordList`; no Rust bridge or owner switch exists.
- `cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check`: passed.
- `git diff --check`: passed after this rerun report was written.

## Residual Risk

The exact invalid-object projection remains fixture-bound to string-valued
entries. A future bridge must define arbitrary Python object conversion and
must preserve the distinction among caught semantic logs, `check()` warning
plus false, and propagated Rust integrity errors.

`BorrowError` is intentionally unreachable through the safe public facade, so
the integrity test can force only an in-module conflict. Pass-two borrow
propagation is established by the same private fallible accessor and `?`
control flow rather than a public runtime reproduction. Any future callback,
reentrant bridge, exposed borrow guard, or concurrent owner model must add
boundary tests for both check passes and specify whether add-SP replacement may
already have occurred.

There is still no production bridge. Python warning presentation, traceback
context, Python/Rust identity across `hfa_api.py` clear/extend repair, routing,
and rollback remain outside this library review. Exact compatibility logs carry
user lyric/phoneme content and require an explicit display, retention, and
telemetry policy before an owner switch.

## Promotion Note

This post-fix error/tracing rerun passes and does not block coordinator
verification of `hfa_wordlist_finalize_core`. The coordinator may combine this
report with the current passing dependency/bootstrap, behavior, and
data/algorithm reruns before updating unit state. This report does not approve
production routing, change runtime ownership, or modify the manifest.
