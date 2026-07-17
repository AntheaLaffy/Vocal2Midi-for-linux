# hfa_wordlist_finalize_core - behavior_reviewer rerun

Date: 2026-07-17
Decision: pass

Unit: `hfa_wordlist_finalize_core`
Role: `behavior_reviewer` (manifest `stage_behavior_reviewer` evidence)

## Findings

No behavior parity findings in the current post-fix implementation.

The exact interior-equality gap is now effective. Legacy Python repairs when
`0 < current.start - previous.end <= gap_length` at
`inference/HubertFA/tools/align_word.py:229`. Fixture
`fill_interior_touch_equal_above` now uses exactly representable values
`2.125 - 2.0 == 0.125` and a separate `3.25 - 3.0 == 0.25` above-threshold case
at `rewrite-in-rust/fixtures/hfa_wordlist_finalize_core.jsonl:7`. Its golden
result requires the equality predecessor and last phoneme to end at `2.125`
while leaving the `0.25` gap unchanged. Rust retains the inclusive comparison at
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:597`. A separate public-API
Python/Rust differential produced byte-identical output and f64 bits
`4001000000000000` for both repaired endpoints, independently proving the
equality branch rather than relying only on the shared fixture runner.

The index-wise production iteration preserves finalizer behavior after removing
whole-source clones. `fill_small_gaps` fixes `entry_count`, reads only the current
canonical handle, and still performs negative-start, strict leading, trailing,
then interior mutations at
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:571`. `add_sp` fixes the
source count, reads source entries by index, sends candidate append diagnostics
to the one outer log, obtains the trailing boundary from the original last
source handle, and commits only after construction at lines 644 through 693.
`check` validates entries and Word internals in its first index pass, then Word
adjacency in its second pass at lines 705 through 796, preserving Python's exact
first-failure order.

The unchanged behavior surface remains exercised by the same 53-row table.
`fill_small_gaps` cases cover strict leading/trailing/interior thresholds,
leading -> trailing -> interior order, helper-warning continuation, partial
mutation before errors, aliases, repetition, and NaN or signed infinity at
fixture lines 1 through 12 and 40 through 44 and 48 through 49. `add_SP` cases at
lines 13 through 24 and 45 through 47 and 50 through 53 cover shared candidate
logs, temporary warnings before outer errors, source/generated identity,
implicit empty/overlap discard, original-last trailing behavior, invalid-entry
partial state, reachable constructor `ValueError`, repetition, custom text,
aliases, and IEEE values. `check` lines 26 through 39 retain every return,
message, first-failure branch, crossing-order case, and repeated warning.

An independent public differential reconstructed those behaviors without the
fixture parser. Ten composed scenarios, including 11 check-order subcases,
matched byte for byte across Python and Rust with the unchanged SHA-256
`ac5a0f9f0573e9ddc2ead01b311823b2201f876018ba5198bbe4238ce039b797`.
It independently covers ordered and partial fill mutation, helper warnings,
candidate warning plus outer error, overlap discard with original-last trailing,
source identity, custom same/larger-wav repetition, local `ValueError`, positive
infinity, persistent clear/extend logs, and all check failure levels. This
confirms the index-wise implementation did not change alias, log, replacement,
or short-circuit semantics.

The Python and Rust gates still consume one table. The legacy checker reads and
executes the JSONL at
`rewrite-in-rust/bootstrap/check_hfa_wordlist_finalize_core.py:152`; Rust embeds
the same path and asserts exactly 53 rows at
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:2521`. Regeneration under
Python 3.12.13 was byte-identical to the checked fixture with SHA-256
`44efe90a4b5c15df77bc0f05b692c85a04686866c181a202a03e101bc29e6701`.
An f64 path audit found no Python/Rust input-bit differences across all 600 JSON
numeric paths, including the new `0.125` values. The two previously documented
one-ULP `serde_json` shifts remain confined to derived expected `dur` tokens;
public duration results match Python exactly.

Prerequisite and ownership behavior is unchanged. The Python model fixture and
45-row collection/AP fixture both pass in their legacy checkers and Rust table
tests. `clear_entries`/`raw_extend` retain canonical caller-compatible list and
log behavior. Static search found no `discard_empty`, `get_text`, duplicate
finalizer state, production `v2m_core::hfa_word` route, or owner switch. The
manifest remains `status: reimplemented` and `current_owner: legacy` at
`rewrite-in-rust/manifest.yaml:1370`.

Writer/reviewer separation is preserved. This rerun did not change production
code, tests, fixtures, checker, bootstrap/dependency artifacts, records, or the
manifest. It adds only this report; public differential clients remained under
`/tmp`. The original behavior report is retained as earlier review history.

## Checks

- `UV_CACHE_DIR=/tmp/uv-cache uv run python --version`: passed; Python 3.12.13.
- `UV_CACHE_DIR=/tmp/uv-cache uv run python rewrite-in-rust/bootstrap/check_hfa_wordlist_finalize_core.py`: passed all 53 current legacy cases.
- Legacy `--generate` plus `cmp`: passed; regenerated JSONL was byte-identical with SHA-256 `44efe90a4b5c15df77bc0f05b692c85a04686866c181a202a03e101bc29e6701`.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_wordlist_finalize -- --nocapture`: passed 3 finalizer tests, including the 53-row table, integrity errors, and 10,000-entry valid/invalid-first regression.
- Exact `0.125` public Python/Rust differential: passed byte for byte; SHA-256 `1e7251d27f529c2cc2e322927b494c870b1a558caeb98dcc56a670f493c0a637`.
- Independent composed public differential: passed 10 scenarios and 11 check-order subcases byte for byte; SHA-256 `ac5a0f9f0573e9ddc2ead01b311823b2201f876018ba5198bbe4238ce039b797`.
- Independent JSON numeric-path audit: 600 numeric paths checked; 0 input-bit mismatches, including equality seed/call values.
- `UV_CACHE_DIR=/tmp/uv-cache uv run python rewrite-in-rust/bootstrap/check_hfa_word_model_core.py`: passed the prerequisite legacy model table.
- `UV_CACHE_DIR=/tmp/uv-cache uv run python rewrite-in-rust/bootstrap/check_hfa_wordlist_collection_ap_core.py`: passed the prerequisite 45-case legacy collection table.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_word_model_follows_parity_fixture_table`: passed the model regression.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_wordlist_collection_follows_parity_fixture_table`: passed the 45-case collection regression.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml`: passed 105 `v2m-core` tests, 5 bridge tests, and doc tests.
- `cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check`: passed.
- `cargo clippy --manifest-path rewrite-in-rust/rust/Cargo.toml --all-targets --all-features -- -D warnings`: passed.
- `RUSTDOCFLAGS='-D warnings' cargo doc --manifest-path rewrite-in-rust/rust/Cargo.toml --no-deps`: passed.
- `git diff --check`: passed before this report was written.
- Static scope/routing search: passed; no out-of-scope finalizer API or production Rust route was found.

## Residual Risk

The shared Rust comparator still accepts ordinary finite differences up to
`1e-12`; the exact equality and composed public probes close the changed critical
branches, but the shared table alone remains branch/parity evidence rather than
a bit-exact proof of every finite output. Arbitrary Python invalid objects,
warning presentation, Python/Rust alias transfer through `hfa_api.py`, bridge
payloads, decoder/aggregation and model/audio/export behavior, production
routing, and owner-switch rollback remain outside this library seam.

## Promotion Note

This rerun passes the manifest `stage_behavior_reviewer` requirement and does
not block coordinator state update. The coordinator must still obtain and
evaluate current passing evidence for every separately required review,
including the post-fix data/algorithm gate, before marking the unit verified.
This report does not update the manifest or approve production owner promotion.
