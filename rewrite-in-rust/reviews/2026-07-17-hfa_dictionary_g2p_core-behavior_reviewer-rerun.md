# hfa_dictionary_g2p_core - behavior_reviewer rerun

Date: 2026-07-17
Decision: pass

Unit: `hfa_dictionary_g2p_core`
Role: `behavior_reviewer` (manifest `stage_behavior_reviewer` evidence)
Writer: `/root/hfa_dictionary_writer`

## Findings

No behavior parity findings in the post-fix implementation.

The blocker in the initial behavior report is closed. Decode failures now
retain explicit Python start and exclusive-end positions at
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_g2p.rs:114`, derive truncated
sequence ends from the remaining input at line 300, and render CPython's
one-byte hexadecimal form versus multi-byte ranged form at line 166. An
independent probe linked the compiled public `HfaDictionaryG2p::from_path` API
and replayed the original eight-case matrix against Python 3.12.13 expected
messages. All eight matched, including the three formerly failing truncated
sequences (`e2 82`, `f0 9f 92`, and `61 e2 82`). A ninth adversarial case,
`e2 82 41`, also matched CPython's multi-byte invalid-continuation range.

The durable table now owns the missing branches rather than relying on the
isolated probe. Fixture lines 25 through 33 cover invalid-start, immediate
continuation, overlong, surrogate, out-of-range, multi-byte invalid-
continuation, and two-/three-byte truncated spans. The same 43-row table is
executed through the real legacy Python `DictionaryG2P` by
`rewrite-in-rust/bootstrap/check_hfa_dictionary_g2p_core.py:58` and through the
Rust public constructor/converter at
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_g2p.rs:1025`; both pass exact
exception, dictionary, output, index, prefix, and warning projections.

The remaining behavior surface also remains exact. Source inspection and
fixture lines 1 through 24 and 41 through 43 confirm whole-file and row
stripping, universal newline translation, required tab fields, ignored later
fields, duplicate replacement, literal-space phone/input empty tokens,
accepted interior `SP`, edge/missing `UserWarning` order and exact text,
accepted-word index advancement, nullable/empty language prefixes, output
order, constructor snapshotting, stable repeated calls, and recovery after a
conversion assertion. The focused Rust suite independently checks structured
load fields, warning retention across assertion failure, later reuse, and the
10,000-entry/token path. Fixture lines 34 through 40 also preserve supported
Linux UTF-8 filename error projections for plain, quote, control, backslash,
non-ASCII, and directory paths.

Scope, ownership, and rollback remain intact. Static search found
`HfaDictionaryG2p` only in the independent `v2m-core::hfa_g2p` module and its
tests. Production `InferenceBase` still imports and constructs legacy Python
`DictionaryG2P` at `inference/HubertFA/tools/infer_base.py:13` and line 162;
`inference/API/hfa_api.py:134` still reaches that legacy route. The manifest
remains `status: reimplemented` and `current_owner: legacy` at
`rewrite-in-rust/manifest.yaml:1450`, with rollback preserving
`DictionaryG2P` and `InferenceBase.get_dataset` at line 1479.

Writer/reviewer separation is preserved. This rerun did not overwrite the
historical failed report or modify production code, fixtures,
dependency/bootstrap artifacts, records, or the manifest. It adds only this
rerun report; independent probe artifacts were isolated under `/tmp`.

## Checks

- `UV_CACHE_DIR=/tmp/uv-cache-hfa-dict-behavior-rerun uv run python rewrite-in-rust/bootstrap/check_hfa_dictionary_g2p_core.py`:
  passed; validated all 43 Python 3.12/Linux UTF-8 golden cases.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_dictionary_g2p_core -- --nocapture`:
  passed 4 focused tests: exact 43-row fixture parity, structured load context,
  repeated/error recovery, and the 10,000-entry/token path.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_phoneme_mora_g2p_core`:
  passed 3 prerequisite shared-output tests.
- Independent compiled-public-API invalid UTF-8 matrix: passed all original 8
  cases plus one multi-byte invalid-continuation case, 9/9 exact type/message
  matches against Python 3.12.13.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml`: passed 113
  `v2m-core` tests, 5 bridge tests, and doc tests.
- `cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check`:
  passed.
- `UV_CACHE_DIR=/tmp/uv-cache-hfa-dict-behavior-rerun uv run python -m py_compile inference/HubertFA/tools/g2p.py`:
  passed.
- `git diff --check`: passed before this report was written.
- Static scope/routing search: passed; no Rust production bridge, Python caller
  switch, dataset/config/export/model/frontend expansion, or rollback change was
  found.

## Residual Risk

The independent nine-case decode matrix and durable fixtures cover each
relevant UTF-8 sequence-length/error class, but do not enumerate every invalid
byte string. The structured `Utf8Error`, explicit start/end positions, reason
branches, and shared fixtures make the remaining risk low.

Non-UTF-8 platform defaults and non-UTF-8 paths remain outside the supported
Linux UTF-8 compatibility projection. Bridge payload validation, Python
warning/exception reconstruction, warning-filter presentation, caller routing,
and applying rollback remain promotion concerns and are still legacy-owned.
Dictionary iteration order is not part of the selected conversion payload; a
future bridge must not make the internal Rust `HashMap` a new compatibility
promise without separate fixtures.

## Promotion Note

This rerun passes the manifest `stage_behavior_reviewer` requirement and this
role no longer blocks a coordinator state update. The coordinator may mark the
unit `verified` only after confirming fresh passing evidence for every other
required role, especially the post-fix `error_tracing_reviewer`, and the current
dependency/bootstrap gate. This report does not update the manifest, change
runtime ownership, or approve production routing.
