# hfa_wordlist_collection_ap_core - behavior_reviewer rerun

Date: 2026-07-17
Decision: pass

Unit: `hfa_wordlist_collection_ap_core`
Role: `behavior_reviewer` (manifest `stage_behavior_reviewer` evidence)

## Findings

No behavior parity findings.

The two prior sorting blockers are closed. The shared table now contains both
exact counterexamples at
`rewrite-in-rust/fixtures/hfa_wordlist_collection_ap_core.jsonl:29` and line
30, equal finite keys and NaN/infinity placement matrices at lines 31 and 34
through 40, and repeated AP sorting at lines 32 and 41. The 65-, 127-, and
257-entry corpora at lines 43 through 45 exercise minrun boundaries, powersort
collapse, both merge directions, and galloping beyond the short insertion path.
All produce the CPython 3.12.13 orders in both the legacy checker and Rust test.

Rust now follows the concrete CPython comparison schedule rather than passing a
non-total NaN comparator to a generic sort. Natural run detection and stable
binary insertion are at
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:731`; minrun, power, and
pending-run collapse are at lines 813 through 856; pre-merge trimming and
merge selection are at lines 858 through 896; and the left/right gallops and
merges are at lines 903 through 1255. These branches use the same directional
raw `<` tests as CPython 3.12.13
[`Objects/listobject.c`](https://github.com/python/cpython/blob/v3.12.13/Objects/listobject.c),
without deriving a false total equivalence for NaN.

An independent public-API differential generated 760 additional lists across
lengths 2 through 300. The 58,080 entries included repeated finite keys, NaN,
positive infinity, and negative infinity and were sorted by a non-overlapping
`add_AP` call. Python 3.12.13 and Rust produced the same ordered-label FNV-64
digest, `7e4f438916c9030d`. This independently covers run, merge, and gallop
layouts beyond the writer-selected corpora.

The exact repr blocker is also closed. `python_string_repr` now selects
Python-style `\x`, `\u`, and `\U` widths from the generated Unicode 15
nonprintable table at
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:1291`; the private 712
range table is in
`rewrite-in-rust/rust/crates/v2m-core/src/python_15_nonprintable.rs:1`. The
shared full-scalar proof at
`rewrite-in-rust/fixtures/hfa_wordlist_collection_ap_core.jsonl:42` matches
Python's 1,112,064 scalar values, 963,066 nonprintable values, and MD5
`f9d22b381b01c21d615f1d6436fec3d3`. The explicit append case at line 33 covers
control, space, format, separator, BMP/non-BMP noncharacters, and a printable
U+10000. A separate public append probe over U+0085, U+00A0, U+200B, U+2028,
U+FFFF, U+10000, and U+10FFFF produced byte-identical Python and Rust logs.

The public alias panic blocker is closed. `WordHandle` keeps its
`Rc<RefCell<Word>>` private and exposes only owned snapshots, identity checks,
and complete fallible mutations at
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:251`. Its `read` and
`write` closures are private and use fallible borrows at lines 310 through 324;
neither source handles nor handles cloned from `WordList::entries()` can retain
a public `Ref` or `RefMut`. The focused source/entry alias test at lines 1981
through 1999 passes and confirms mutations remain visible through both handles
without a guard or panic surface.

The original 28 collection/AP cases remain unchanged in coverage and pass.
They continue to prove raw append bypass, invalid entries, overlap/touching,
logs, interval validation and special floats, AP branches and thresholds,
aliasing, caught-error partial state, repeated calls, projections, and prefix
partial mutation.

Scope and ownership remain correct. No Rust `fill_small_gaps`, `add_SP`, or
`check` implementation was found; finalization remains the planned next unit.
Production decoder and inference callers still import Python `WordList` at
`inference/HubertFA/tools/decoder.py:5` and
`inference/HubertFA/tools/infer_base.py:10`. The manifest remains
`status: reimplemented` and `current_owner: legacy` at
`rewrite-in-rust/manifest.yaml:1334`. Rust only exports the independent module
at `rewrite-in-rust/rust/crates/v2m-core/src/lib.rs:17`.

Writer/reviewer separation remains intact. I wrote earlier dependency/bootstrap
fixture artifacts but did not write this Rust production implementation or its
Rust tests. This rerun did not modify production code, fixtures, bootstrap or
dependency artifacts, the manifest, or migration records.

## Checks

- `UV_CACHE_DIR=/tmp/uv-cache uv run python --version`: passed; Python 3.12.13.
- `UV_CACHE_DIR=/tmp/uv-cache uv run python rewrite-in-rust/bootstrap/check_hfa_wordlist_collection_ap_core.py`: passed all 45 shared cases.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_wordlist_collection`: passed; 1 selected fixture-table test passed and 101 were filtered out.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml word_handles_preserve_aliases_without_exposing_borrow_guards`: passed; 1 selected alias-safety test passed.
- `cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check`: passed.
- `git diff --check`: passed before this report was written.
- Independent public sort differential: passed; Python and Rust matched across 760 cases and 58,080 entries with FNV-64 `7e4f438916c9030d`.
- Independent public repr differential: passed; Python and Rust warning-log UTF-8 hex matched for the seven requested control/format/separator/BMP/non-BMP boundaries.
- Static boundary/routing search: passed; no finalizer scope creep and no production Rust caller route found.

## Residual Risk

The sort compatibility contract is deliberately pinned to CPython 3.12.13's
comparison schedule. A Python interpreter upgrade must regenerate and rerun the
NaN corpus. The Unicode proof covers Unicode 15 scalar values; Rust `String`
cannot represent Python lone surrogates. Invalid entries remain string-valued,
and the safe handle mutation facade covers the selected fixture/caller fields
rather than arbitrary Python duck typing. A future bridge must define those
conversions, diagnostic mapping, and rollback before routing production calls.
Finalization, decoder math, model/audio/export IO, and runtime presentation
remain outside this behavior gate.

## Promotion Note

This behavior role no longer blocks coordinator state update. The coordinator
may record the manifest `stage_behavior_reviewer` requirement as passed using
this rerun, but must still evaluate the independently required dependency,
data/algorithm, and error-tracing evidence before marking the unit verified.
This report does not update the manifest or approve production owner promotion.
