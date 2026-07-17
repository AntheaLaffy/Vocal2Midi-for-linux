# hfa_dictionary_g2p_core - error_tracing_reviewer

Date: 2026-07-17
Decision: fail

Unit: `hfa_dictionary_g2p_core`
Role: `error_tracing_reviewer`

## Findings

- Severity: high
- Location: `rewrite-in-rust/rust/crates/v2m-core/src/hfa_g2p.rs:167`
- Issue: The advertised exact `UnicodeDecodeError` compatibility projection is
  wrong for invalid spans longer than one byte, and the structured decode error
  does not retain enough information to reconstruct a truncated multibyte span.
  For `error_len > 1`, `compatibility_message()` still renders singular
  `byte 0x..` and plural `positions`; Python renders plural `bytes`, omits the
  hexadecimal byte, and retains singular `position`. For truncated input,
  `Utf8Error::error_len()` is `None`, but lines 177 through 184 treat that as a
  one-byte span, losing the actual end position. This contradicts the exact
  decode position/reason and compatibility-projection contract in
  `rewrite-in-rust/dependencies/hfa_dictionary_g2p_core.yaml:25` and
  `rewrite-in-rust/bootstrap/hfa_dictionary_g2p_core.md:61`.
- Evidence: A temporary Rust probe imported the current `hfa_g2p.rs` directly
  and was compared with Python 3.12.13. For bytes `e2 82 41` after the valid
  `word\t` prefix, Rust returned
  `'utf-8' codec can't decode byte 0xe2 in positions 5-6: invalid continuation byte`,
  while Python returned
  `'utf-8' codec can't decode bytes in position 5-6: invalid continuation byte`.
  For truncated `f0 90 80`, Rust returned a one-byte `position 5` projection,
  while Python returned `bytes in position 5-7`. The same lost range occurred
  for truncated `e2 82` and `f0 90`. Invalid-start, immediate invalid-
  continuation, overlong, surrogate, and out-of-range probes retained the
  correct start and reason; the blocker is specifically multi-byte message and
  end-range fidelity. The current golden table covers only the single-byte
  `0xff` case at `rewrite-in-rust/fixtures/hfa_dictionary_g2p_core.jsonl:25`, so
  its passing result does not exercise this branch.
- Required fix: Retain an explicit Python decode end position (or equivalent
  input length/span) in `HfaDictionaryG2pLoadError::Decode`. Render one-byte
  spans as `byte 0x.. in position N`, and longer spans as
  `bytes in position N-M`, without a hexadecimal byte. Add shared Python/Rust
  fixtures for late invalid continuation and one-, two-, and three-byte
  truncated sequences, plus structured assertions for operation, path, source,
  start, end, byte, and reason. Keep the overlong, surrogate, and out-of-range
  probes as reason regressions.

- Severity: medium
- Location: `rewrite-in-rust/rust/crates/v2m-core/src/hfa_g2p.rs:748`
- Issue: `python_io_message()` does not implement Python's filename `repr`
  semantics for all valid UTF-8 paths, despite exposing a Python compatibility
  message. It always chooses single quotes and escapes only backslash and
  apostrophe; Python may choose double quotes and escapes control characters.
  `to_string_lossy()` would also erase non-UTF-8 path bytes if such a `Path` is
  admitted. The original `PathBuf`, `io::Error`, kind, and raw OS error remain
  available, so the structured source is diagnosable; only the exact projection
  is incomplete.
- Evidence: Direct current-Rust/Python 3.12.13 missing-path probes matched for a
  plain path and backslash. For `/tmp/missing-'`, Rust projected a single-quoted
  path with `\'`, while Python chose a double-quoted filename. For filenames
  containing newline or tab, Rust embedded the control character while Python
  projected `\n` or `\t`. Existing fixture line 26 uses only
  `<TMP>/dictionary.txt` and cannot detect these cases.
- Required fix: Before claiming general exact I/O compatibility, either define
  and enforce a narrower accepted path payload in the seam/promotion record, or
  implement and fixture Python-compatible filename representation, including
  quote choice, control escaping, and a deliberate non-UTF-8 path policy.

No additional error/tracing issue was found in the reviewed boundary. Load
errors retain operation and path, I/O errors retain the source and kind through
`Error::source`, decode errors retain the `Utf8Error`, start, current length,
leading byte, and reason, and malformed rows retain zero-based row index and
field count (`rewrite-in-rust/rust/crates/v2m-core/src/hfa_g2p.rs:108`). The
shared base failure remains a structured `HfaG2pError` with exact legacy
`IndexError`/`AssertionError` type and message and is returned beside warnings
in `HfaDictionaryG2pConversion` at line 263.

Warnings are ordered data rather than emitted logs. Each retains the raw input
word position; edge-SP warnings also retain the dictionary phone position, and
both expose exact `UserWarning` category/message projections at
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_g2p.rs:214`. The raw word and
dictionary path can contain user lyric or filesystem data, but static inspection
found no stdout, stderr, tracing, logging-framework, file, bridge, or telemetry
sink. A future bridge must classify those exact compatibility values before
display or telemetry rather than logging them automatically.

Repeated/error-recovery behavior is sound for the current immutable seam. The
converter snapshots the file once, returns warnings emitted before a later base
assertion, remains usable after that assertion, and repeats identical successful
results and warnings (`rewrite-in-rust/rust/crates/v2m-core/src/hfa_g2p.rs:287`,
line 332, and line 1159). The implementation remains within the confirmed
library boundary: production `InferenceBase` still imports and constructs the
legacy Python `DictionaryG2P` at `inference/HubertFA/tools/infer_base.py:13` and
line 162. Writer/reviewer separation is intact; this review changed only this
report.

## Checks

- `env UV_CACHE_DIR=/tmp/uv-cache-hfa-dict-error uv run python rewrite-in-rust/bootstrap/check_hfa_dictionary_g2p_core.py`: passed all 30 current legacy fixture cases.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_dictionary_g2p_core -- --nocapture`: passed all 4 focused Rust tests.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_dictionary_g2p_core_retains_structured_load_context -- --nocapture`: passed the current I/O, decode, and malformed-row field assertions.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_dictionary_g2p_core_repeats_and_recovers_after_assertion_errors -- --nocapture`: passed warning retention, failure recovery, and repeated-call assertions.
- Direct Python 3.12.13/current-Rust UTF-8 probe: invalid start, invalid continuation, overlong, surrogate, out-of-range, and truncated multibyte cases inspected; found the high-severity multi-byte projection mismatch above.
- Direct Python 3.12.13/current-Rust missing-path probe: plain/backslash cases matched; apostrophe/newline/tab cases exposed the medium-severity representation mismatch above.
- Static diagnostic-sink and routing inspection: no Rust log/telemetry sink or production route was found; Python remains the runtime owner.
- `cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check`: passed.
- `git diff --check`: passed before this report was written.

## Residual Risk

The current shared `InvalidSilenceLayout` error intentionally preserves the
legacy empty `AssertionError` message and does not identify the offending
dictionary word or consecutive-SP pair. The conversion wrapper makes the
operation implicit and retains prior warnings, but promotion should decide
whether a separate non-compatibility trace field is needed without changing the
Python projection.

There is no production bridge. Python warning and exception reconstruction,
path payload validation, diagnostic display/retention policy, caller routing,
and rollback execution remain promotion work. This report does not approve any
runtime-owner change.

## Promotion Note

This role blocks coordinator state update. `hfa_dictionary_g2p_core` must remain
`reimplemented`; the coordinator cannot mark it `verified` until a writer fixes
the decode projection/range defect, resolves or explicitly constrains the path
projection, and an independent `error_tracing_reviewer` rerun passes. Rollback
remains keeping Python `DictionaryG2P` and `InferenceBase.get_dataset`
dictionary selection as runtime owners.
