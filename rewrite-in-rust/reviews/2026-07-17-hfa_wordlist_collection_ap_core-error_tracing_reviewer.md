# hfa_wordlist_collection_ap_core - error_tracing_reviewer

Date: 2026-07-17
Decision: pass

Unit: `hfa_wordlist_collection_ap_core`
Role: `error_tracing_reviewer`

## Findings

No error-tracing findings.

`HfaWordListError` retains distinct `ValueError`, `AttributeError`, and
Rust-integrity `BorrowError` kinds at
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:350`. The concrete kind and
message fields are private, while the supported exception projection is
`exception_type()` and the supported message projection is `Display` at lines
391 through 404. The fixture adapter uses exactly those two public projections
at lines 1623 through 1629. The shared table proves both validation-order
`ValueError` messages at
`rewrite-in-rust/fixtures/hfa_wordlist_collection_ap_core.jsonl:5` and line 17,
and the exact invalid-string `AttributeError` type/message pairs at line 27.
This avoids parsing `Debug` output or collapsing exception kinds at the future
bridge boundary.

Legacy caught-versus-uncaught ordering is preserved. Standalone interval
validation raises before any subtraction, raw-interval validation precedes
remove-interval validation, and Rust returns the corresponding structured
`ValueError` in the same order
(`inference/HubertFA/tools/align_word.py:156`,
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:498`). Legacy `add_AP`
wraps its complete body in `except Exception` and appends `ERROR in add_AP: ...`
(`inference/HubertFA/tools/align_word.py:181`); Rust converts the selected
legacy semantic failures to the same persistent log and returns `Ok(())` at
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:575`, line 586, and line
602. In contrast, legacy projection and prefix methods do not catch invalid
entry attribute failures (`inference/HubertFA/tools/align_word.py:268`), and
Rust returns exact `AttributeError` values from `phoneme_texts`, `intervals`, and
`clear_language_prefix` at
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:621`, line 642, and line
658.

Append and AP diagnostic mutation order also matches. Rejected empty or
overlapping `append` calls log once without insertion at
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:474`. A no-overlap AP is
inserted before sorting, so a later invalid-entry sort error leaves the inserted
handle stored and logs the error; the shared proof is
`rewrite-in-rust/fixtures/hfa_wordlist_collection_ap_core.jsonl:23`. An invalid
entry encountered during overlap subtraction is caught before fragment
insertion (line 13), while a fully consumed residual defers the invalid-entry
failure until sorting (line 14). Prefix cleanup mutates earlier Words before an
invalid entry returns uncaught `AttributeError`, as proved at line 28. The
targeted legacy probe reproduced all four state/error shapes and their exact
messages.

The former public `RefCell` panic surface is closed. `WordHandle` owns a private
`Rc<RefCell<Word>>` and exposes only owned snapshots, identity comparison, and
closed fallible mutations at
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:251`. Both private accessors
use `try_borrow`/`try_borrow_mut`, never panicking `borrow`/`borrow_mut`, at lines
310 through 324. Safe external callers cannot obtain the inner `Rc`, `RefCell`,
`Ref`, or `RefMut`; every public operation completes its internal borrow before
returning. Consequently the current safe public API cannot manufacture a
`BorrowError`. If an in-module conflict is introduced later, ordinary methods
propagate it through `?`, and `add_ap` explicitly returns it rather than
misclassifying it as a caught legacy business error at lines 558 through 560
and 602 through 604. It is therefore neither leaked by a reachable current
path nor silently swallowed into the WordList log.

Exact logs remain an ordered, caller-readable in-memory buffer: append joins
legacy dataclass repr text with the original warning, AP uses the legacy error
prefix, and `clear_log` resets without affecting entries
(`rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:409`, line 444, and line
474). The 45-case table covers log accumulation/clearing, exact quote and
backslash escaping, Unicode printability, error order, and partial mutation at
`rewrite-in-rust/fixtures/hfa_wordlist_collection_ap_core.jsonl:1`, line 2,
line 4, lines 13 through 14, line 23, line 33, and line 42. These compatibility
logs can contain user-supplied Word and phoneme text, but the Rust unit does not
send them to stdout, files, telemetry, or a bridge. It adds no path, token,
credential, model, or environment detail beyond the legacy in-memory surface.

Writer/reviewer separation is intact. This review did not modify production
code, fixtures, dependency/bootstrap artifacts, the manifest, or records.

## Checks

- `env UV_CACHE_DIR=/tmp/uv-cache-hfa-error-review uv run python rewrite-in-rust/bootstrap/check_hfa_wordlist_collection_ap_core.py`: passed all 45 legacy fixture rows (exit 0; the checker is silent on success).
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_wordlist_collection -- --nocapture`: passed; 1 selected fixture-table test passed and 101 were filtered out in `v2m-core`.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml word_handles_preserve_aliases_without_exposing_borrow_guards -- --nocapture`: passed; 1 selected public alias/fallible-handle test passed and 101 were filtered out.
- Targeted legacy caught/uncaught and partial-mutation probe: passed; caught AP invalid-entry text was exactly `ERROR in add_AP: 'str' object has no attribute 'start'`, the no-overlap insertion survived its sort failure, all three invalid projections returned the expected `AttributeError`, and prefix cleanup retained the earlier `a` mutation while leaving the later `ja/b` unchanged.
- Static borrow search over `hfa_word.rs`: inspected; Word access uses only private `try_borrow` and `try_borrow_mut`, with no panicking `borrow()` or `borrow_mut()` call.
- Static production-routing search: inspected; `inference/HubertFA/tools/decoder.py:5` and `inference/HubertFA/tools/infer_base.py:10` still import the legacy Python types, while the manifest remains `status: reimplemented` and `current_owner: legacy` at `rewrite-in-rust/manifest.yaml:1334`.
- `cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check`: passed.
- `git diff --check`: passed after this report was written.

## Residual Risk

The current public API deliberately makes borrow conflicts unreachable, so the
focused test proves public alias usability rather than forcing the private
`BorrowError` branch. Any future in-module API that exposes a borrow guard or
runs caller code while a borrow is held must add a forced-conflict regression
and preserve propagation rather than logging it as a legacy `add_AP` error.

Only string-valued invalid entries are represented. A production bridge must
define exception messages for broader Python object types rather than assuming
the current `'str' object ...` projection applies universally. It must also
decide where exact lyric/phoneme-bearing logs may be displayed or retained;
redacting the current in-memory compatibility value would break parity, but
forwarding it to external telemetry without classification would expose user
content.

The Rust unit has no production bridge. Python warning/log presentation,
decoder and inference failures, finalization, model/audio/export IO, and
cross-language traceback/context propagation remain outside this review.

## Promotion Note

This error-tracing role does not block coordinator state update. The coordinator
may record `error_tracing_reviewer` as passed for the current independent,
legacy-owned library seam. This report does not approve production routing and
does not by itself mark the unit verified; the coordinator must separately
resolve or supersede every other required review, including the existing
data/algorithm failure report.
