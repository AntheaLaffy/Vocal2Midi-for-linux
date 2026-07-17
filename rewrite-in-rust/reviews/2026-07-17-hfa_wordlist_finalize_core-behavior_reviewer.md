# hfa_wordlist_finalize_core - behavior_reviewer

Date: 2026-07-17
Decision: pass

Unit: `hfa_wordlist_finalize_core`
Role: `behavior_reviewer` (manifest `stage_behavior_reviewer` evidence)

## Findings

No behavior parity findings.

`fill_small_gaps` preserves the legacy evaluation and mutation contract. Python
checks the negative raw start, strict leading repair, trailing repair, and then
interior gaps at `inference/HubertFA/tools/align_word.py:217`; Rust follows the
same order at `rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:548`. The
leading tests remain strict on both sides, trailing remains inclusive, and
interior remains `0 < gap <= gap_length`. The shared table covers equality and
neighboring counterexamples, partial state before an invalid entry or empty-phone
error, helper-warning continuation, repeated aliases/calls, and NaN or signed
infinities at `rewrite-in-rust/fixtures/hfa_wordlist_finalize_core.jsonl:1`
through line 12 and lines 40 through 44 and 48 through 49. An independent public
probe additionally matched leading -> trailing -> interior mutation, a trailing
helper warning followed by both interior repairs, negative-start plus trailing
partial mutation before an invalid middle entry, NaN gap suppression, and
negative-infinity wav warning behavior byte for byte.

`add_sp` preserves candidate construction and replacement semantics. Legacy
shares the original log with its temporary list, validates every candidate
through `append`, takes the trailing start from the original last entry, replaces
only after candidate construction completes, and ignores the final `check()`
boolean at `inference/HubertFA/tools/align_word.py:235`. Rust writes candidate
warnings into the same persistent buffer through the canonical append helper at
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:512`, retains the source
handles and original-last read at lines 646 through 695, and likewise ignores
the boolean. Fixture lines 13 through 24 and 45 through 47 and 50 through 53
cover leading/interior/trailing/custom insertion, identity, implicit empty and
overlap discard, temporary warning plus outer error ordering, invalid first/
middle/last partial state, reachable local constructor `ValueError`, repetition,
aliases, and NaN or signed-infinity wav values. Independent public cases matched
all of candidate-warning-then-outer-error, overlap discard with original-last
trailing placement, exact source identity, custom same/larger-wav repetition,
local `ValueError` plus subsequent check warning, and positive-infinity SP.

`check` preserves return values, exact first-failure order, messages, and repeat
accumulation. Python validates entry type, word order, empty phones, first edge,
last edge, each phone order/adjacency, and only then word adjacency at
`inference/HubertFA/tools/align_word.py:284`; Rust performs those checks in the
same sequence at `rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:707`.
Fixture lines 26 through 39 cover every branch, crossing defects, NaN invalidity,
finite-to-positive-infinity validity, and duplicate warnings. The independent
public probe exercised 11 check scenarios and matched all returns and exact logs,
including internal-before-later-gap and repeated word-gap failures.

Caller-compatible clear/extend behavior remains on the canonical collection.
`clear_entries` retains diagnostics and `raw_extend` bypasses append validation
at `rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:461`, matching Python
list `clear()`/`extend()` used by `inference/API/hfa_api.py:75`. Fixture line 25
and an independent public probe preserve the log, raw replacement, source
identity, and subsequent check result. `infer_base.py` still calls Python
`fill_small_gaps`, `add_SP`, and `log` in order at
`inference/HubertFA/tools/infer_base.py:233`; `hfa_api.py` still repairs the same
Python prediction objects after inference at line 83.

The two runtimes consume one fixture artifact rather than duplicated tables.
The Python checker imports the legacy classes and reads the JSONL at
`rewrite-in-rust/bootstrap/check_hfa_wordlist_finalize_core.py:20` and line 23;
the Rust test embeds that exact path at
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:1686`, executes it at line
2319, and asserts a 53-case count at line 2537. Regeneration under Python 3.12.13
was byte-identical to the checked table with SHA-256
`01244c584f3f7dd5c120be706fc6a2b5f118830dc92c24a08eabf62fe8718b3e`.
A separate audit compared all 600 finite numeric JSON paths: all input paths had
identical Python/Rust f64 bits. Current `serde_json` decoding rounds two derived
expected-duration tokens by one ULP, but public Rust duration results for both
values matched Python bit for bit; the shared comparator's tolerance accepts
those parser-only expected-value shifts.

Scope and rollback remain intact. No `discard_empty` or `get_text` implementation
or call was found, no second interval/entry/log representation was introduced,
and no production path references `v2m_core::hfa_word`. The only Rust exposure is
the independent workspace module at
`rewrite-in-rust/rust/crates/v2m-core/src/lib.rs:17`. The manifest remains
`status: reimplemented` and `current_owner: legacy` at
`rewrite-in-rust/manifest.yaml:1370`; Python runtime ownership and the rollback
route are unchanged.

Writer/reviewer separation is preserved. This review did not write the Rust
implementation, production tests, fixture/checker, bootstrap/dependency record,
manifest, or migration records. It adds only this review report; independent
probes and the stricter comparator experiment were isolated under `/tmp`.

## Checks

- `UV_CACHE_DIR=/tmp/uv-cache uv run python --version`: passed; Python 3.12.13.
- `UV_CACHE_DIR=/tmp/uv-cache uv run python rewrite-in-rust/bootstrap/check_hfa_wordlist_finalize_core.py`: passed all 53 shared cases.
- Legacy `--generate` plus `cmp`: passed; regenerated JSONL was byte-identical with SHA-256 `01244c584f3f7dd5c120be706fc6a2b5f118830dc92c24a08eabf62fe8718b3e`.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_wordlist_finalize -- --nocapture`: passed; both finalizer tests passed, including direct fixture consumption and structured integrity errors.
- Independent public Python/Rust differential under `/tmp`: passed; 10 composed scenarios, including 11 check-order subcases, produced byte-identical output with SHA-256 `ac5a0f9f0573e9ddc2ead01b311823b2201f876018ba5198bbe4238ce039b797`.
- Independent JSON numeric-path audit: 600 paths checked; 0 input bit mismatches and 2 derived expected-duration parser shifts. Direct public duration probes matched Python at bits `3fdccccccccccccc` and `3fb9999999999999`.
- Temporary bit-strict fixture-copy experiment: identified the documented expected-value parser shift at fixture line 2; follow-up path and public-API probes showed the Rust implementation result equals Python and the shared input is unchanged.
- `UV_CACHE_DIR=/tmp/uv-cache uv run python rewrite-in-rust/bootstrap/check_hfa_word_model_core.py`: passed the prerequisite legacy model table.
- `UV_CACHE_DIR=/tmp/uv-cache uv run python rewrite-in-rust/bootstrap/check_hfa_wordlist_collection_ap_core.py`: passed the prerequisite 45-case legacy collection table.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_word_model_follows_parity_fixture_table`: passed the existing model gate.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_wordlist_collection_follows_parity_fixture_table`: passed the existing 45-case collection gate.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml`: passed 104 `v2m-core` tests, 5 bridge tests, and doc tests.
- `cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check`: passed.
- `cargo clippy --manifest-path rewrite-in-rust/rust/Cargo.toml --all-targets --all-features -- -D warnings`: passed.
- `RUSTDOCFLAGS='-D warnings' cargo doc --manifest-path rewrite-in-rust/rust/Cargo.toml --no-deps`: passed.
- `git diff --check`: passed before this report was written.
- Static scope/routing search: passed; no `discard_empty`, `get_text`, duplicate finalizer storage, or production Rust route was found.

## Residual Risk

The shared Rust fixture comparator intentionally accepts finite numeric
differences up to `1e-12`; the audit above closes the two known parser-induced
expected-duration shifts and the independent public differential covers exact
critical boundary values, but the shared table alone is not a bit-exact proof for
every future finite output. The seam also remains limited to f64/string fixtures,
string-valued invalid entries, and a single-threaded safe handle facade. Arbitrary
Python objects, warning-filter presentation, Python/Rust alias transfer across
`hfa_api.py`, decoder and aggregation behavior, audio/model/export IO, bridge
payloads, production routing, and owner-switch rollback remain unproved and must
be defined before runtime promotion.

## Promotion Note

This behavior role does not block coordinator state update. The coordinator may
record the manifest `stage_behavior_reviewer` requirement as passed using this
report, but must still evaluate the separately required dependency/bootstrap,
data/algorithm, and error/tracing evidence before marking the unit verified.
This report does not update the manifest or approve production owner promotion.
