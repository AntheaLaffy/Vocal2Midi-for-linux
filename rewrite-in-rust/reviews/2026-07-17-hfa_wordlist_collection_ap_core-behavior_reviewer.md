# hfa_wordlist_collection_ap_core - behavior_reviewer

Date: 2026-07-17
Decision: fail

Unit: `hfa_wordlist_collection_ap_core`
Role: `behavior_reviewer` (manifest `stage_behavior_reviewer` evidence)

## Findings

### 1. Mixed NaN starts can sort into a different observable order

- Severity: medium
- Location: `rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:589`
- Issue: Rust replaces Python's `list.sort(key=lambda w: w.start)` with an
  insertion loop. These algorithms are equivalent for totally ordered finite
  keys, but not for mixed finite and NaN keys because comparison order affects
  the result. This violates the selected AP stable-sorting contract and changes
  subsequent entry, phoneme, and interval projection order.
- Evidence: legacy Python sorts after both no-overlap and overlap AP paths at
  `inference/HubertFA/tools/align_word.py:195` and
  `inference/HubertFA/tools/align_word.py:213`; Rust uses adjacent insertion
  comparisons at `rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:597`.
  A public-path probe seeded raw starts `[1.0, -1.0, NaN]` and added a
  non-overlapping AP Word at `[0.6, 0.9]`. Python produced start order
  `[-1.0, 0.6, 1.0, NaN]`; applying the Rust loop produced
  `[-1.0, 1.0, NaN, 0.6]`. The shared NaN fixture only covers the simpler
  `[NaN, 1.0, 10.0]` arrangement at
  `rewrite-in-rust/fixtures/hfa_wordlist_collection_ap_core.jsonl:24`, where
  the two algorithms happen to agree.
- Required fix: add this mixed-key AP case to the shared fixture and make the
  supported Rust sort behavior match the legacy Python result. If exact
  CPython behavior for non-total keys is intentionally unsupported, narrow the
  public contract and bootstrap claim explicitly before rerunning this gate.

### 2. Exact append diagnostics do not escape nonprintable Unicode like Python

- Severity: medium
- Location: `rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:653`
- Issue: `python_string_repr` escapes a Unicode character only when Rust
  `char::is_control()` is true. Python `repr` also escapes nonprintable Unicode
  separators and format characters. As a result, append rejection logs can
  contain raw invisible characters where the legacy dataclass repr contains
  visible `\u` escapes, violating the exact-diagnostic contract at
  `rewrite-in-rust/manifest.yaml:1344`.
- Evidence: an empty-phoneme Word whose text is
  `line\u2028zero\u200bwidth` logs Python repr bytes containing
  `5c7532303238` and `5c7532303062`, the literal strings `\u2028` and
  `\u200b`. Both characters report `is_control() == false` in Rust, and an
  isolated execution of the implementation's helper emitted raw UTF-8
  `e280a8` and `e2808b`. The current escaping fixture at
  `rewrite-in-rust/fixtures/hfa_wordlist_collection_ap_core.jsonl:2` covers
  quotes, backslashes, and newline only.
- Required fix: implement Python-compatible escaping for all nonprintable
  Unicode code points, including correct `\x`, `\u`, and `\U` widths, and add
  shared append-warning cases for U+2028 and U+200B before rerunning this gate.

No other behavior mismatch was found in the reviewed surface. All 28 shared
cases agree for append/raw bypass, string-valued invalid entries, overlap and
touching boundaries, log order/clear, interval validation and subtraction over
finite and special floats, AP branch-local minimum handling, default and
explicit thresholds, aliasing, caught-error partial state, repeated calls,
projections, and prefix partial mutation.

The unit boundary remains intact. Rust defines collection/AP/prefix behavior
but no `fill_small_gaps`, `add_SP`, or `check` implementation; those stay in
the planned finalization unit. Production decoder and inference callers still
import and invoke Python `WordList` at
`inference/HubertFA/tools/decoder.py:5` and
`inference/HubertFA/tools/infer_base.py:10`. The Rust module is only exported by
the independent workspace at
`rewrite-in-rust/rust/crates/v2m-core/src/lib.rs:17`, so rollback remains the
current runtime state.

Writer/reviewer separation is preserved. I wrote dependency/bootstrap fixture
artifacts for this unit but did not write its Rust production implementation or
Rust tests. This review did not modify production code, fixtures, bootstrap or
dependency artifacts, the manifest, or migration records.

## Checks

- `UV_CACHE_DIR=/tmp/uv-cache uv run python rewrite-in-rust/bootstrap/check_hfa_wordlist_collection_ap_core.py`: passed all 28 shared cases.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_wordlist_collection`: passed; 1 selected Rust fixture-table test passed, 100 were filtered out, and 0 bridge tests were selected.
- `cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check`: passed.
- `git diff --check`: passed before this report was written.
- Targeted Python/Rust nonprintable-Unicode repr probe: failed parity for U+2028 and U+200B as described above.
- Targeted mixed finite/NaN AP sort probe: failed parity as described above.
- Static boundary/routing search: no finalization-method creep and no production Rust caller routing found.

## Residual Risk

The proof remains limited to string-valued invalid entries and `f64` interval
values. It does not establish arbitrary Python duck-typed list contents,
custom comparison/property side effects, a future Python/Rust bridge, decoder
numeric behavior, finalization, model/audio/export IO, or runtime warning
presentation. The two findings also show that the shared table currently
under-samples Python repr and non-total sort behavior despite claiming those
surfaces.

## Promotion Note

This role blocks recording the manifest `stage_behavior_reviewer` requirement
as passed. Keep `hfa_wordlist_collection_ap_core` at `reimplemented` until both
parity gaps are fixed or explicitly removed from the accepted contract and an
independent behavior rerun passes. This report does not update the manifest.
