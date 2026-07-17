# hfa_dictionary_g2p_core - behavior_reviewer

Date: 2026-07-17
Decision: fail

Unit: `hfa_dictionary_g2p_core`
Role: `behavior_reviewer` (manifest `stage_behavior_reviewer` evidence)
Writer: `/root/hfa_dictionary_writer`

Writer/reviewer separation is preserved. This review did not modify production
code, fixtures, dependencies/bootstrap artifacts, records, or the manifest.

## Findings

- Severity: medium
- Location: `rewrite-in-rust/rust/crates/v2m-core/src/hfa_g2p.rs:166`
- Issue: truncated multi-byte UTF-8 dictionary contents do not preserve the
  exact Python 3.12 `UnicodeDecodeError` message. When Rust reports
  `Utf8Error::error_len() == None`, `compatibility_message` substitutes a
  one-byte length at line 177 and always renders one offending byte at lines
  186-188. CPython instead reports the complete truncated byte range with the
  plural `bytes` form. This contradicts the unit's invalid-UTF-8 constructor
  error contract and prevents exact Python exception reconstruction.
- Evidence: an independent probe called the real Python 3.12.13 decoder and the
  compiled public Rust `HfaDictionaryG2p::from_path` API for eight invalid UTF-8
  patterns. Five single-byte-range cases matched, but all three truncated
  multi-byte cases failed:

  | Bytes after `word\t` | Python 3.12.13 | Rust |
  | --- | --- | --- |
  | `e2 82` | `'utf-8' codec can't decode bytes in positions 5-6: unexpected end of data` | `'utf-8' codec can't decode byte 0xe2 in position 5: unexpected end of data` |
  | `f0 9f 92` | `'utf-8' codec can't decode bytes in positions 5-7: unexpected end of data` | `'utf-8' codec can't decode byte 0xf0 in position 5: unexpected end of data` |
  | `61 e2 82` | `'utf-8' codec can't decode bytes in positions 6-7: unexpected end of data` | `'utf-8' codec can't decode byte 0xe2 in position 6: unexpected end of data` |

  The durable table has only the passing single invalid-start-byte case
  (`0xff`) at `rewrite-in-rust/fixtures/hfa_dictionary_g2p_core.jsonl:25`, so
  both fixture harnesses can pass while this branch remains wrong.
- Required fix: retain the full Python invalid span (`start` and exclusive
  `end`) when constructing `HfaDictionaryG2pLoadError::Decode`; for an
  incomplete Rust sequence, derive `end` from the remaining input bytes.
  Render CPython's singular byte/hex form only for a one-byte span and its
  plural ranged form for longer spans. Add golden cases for truncated
  three-byte and four-byte sequences and for an invalid continuation after a
  valid multi-byte prefix, then rerun the real Python fixture checker, focused
  Rust tests, and this independent adversarial comparison.

No other behavior finding was identified. Source inspection and the shared
30-case table agree on whole-file/row stripping, universal newlines, required
tab fields, ignored extra fields, duplicate replacement, literal-space phone
and input tokens, accepted interior `SP`, edge/missing warning order and exact
text, word-index advancement, nullable/empty language prefixes, output arrays,
constructor snapshotting, repeated calls, and recovery after conversion-level
assertion errors.

The unit stayed within its confirmed boundary. Static search found the Rust
dictionary types only in the independent `v2m-core::hfa_g2p` module and its
tests. `inference/HubertFA/tools/infer_base.py` and
`inference/API/hfa_api.py` still route to legacy Python, `current_owner` remains
`legacy`, and the rollback remains keeping `DictionaryG2P` and
`InferenceBase.get_dataset` as runtime owners.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_hfa_dictionary_g2p_core.py`:
  passed; validated all 30 current Python golden cases.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_dictionary_g2p_core -- --nocapture`:
  passed 4 focused tests, including the shared fixture table, structured load
  context, repeated/error recovery, and the 10,000-entry/token path.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_phoneme_mora_g2p_core`:
  passed 3 prerequisite shared-output tests.
- Independent compiled-public-API invalid UTF-8 matrix: failed 3 of 8 cases;
  truncated `e2 82`, `f0 9f 92`, and `61 e2 82` produced the mismatches above.
  Invalid start, overlong, surrogate, above-maximum, and immediate bad-
  continuation representatives matched Python's one-byte error projection.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml`: passed 112
  `v2m-core` tests, 5 bridge tests, and doc tests.
- `cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check`:
  passed.
- `uv run python -m py_compile inference/HubertFA/tools/g2p.py`: passed.
- `git diff --check`: passed before this report was written.
- Static scope/routing search: passed; no Python production import, bridge,
  dataset route, model execution, config, export, or frontend change was found
  for this unit.

## Residual Risk

The adversarial decode matrix is representative rather than exhaustive. After
the identified span-formatting defect is fixed, a broader table should cover
every UTF-8 sequence-length class and invalid position so the durable fixture
gate, rather than an isolated reviewer probe, owns this contract.

Non-UTF-8 platform defaults, non-UTF-8 paths, bridge payload validation,
Python warning/exception reconstruction, and warning-filter presentation remain
explicit promotion concerns outside this independent library unit.

## Promotion Note

This behavior review blocks the manifest `stage_behavior_reviewer` gate. The
coordinator cannot update `hfa_dictionary_g2p_core` from `reimplemented` to
`verified` until the invalid-UTF-8 error-range mismatch is fixed and an
independent behavior rerun passes. Python remains the runtime owner and the
documented rollback route remains intact. The separately required
dependency/bootstrap and error/tracing reviews must also pass before any state
update or promotion decision.
