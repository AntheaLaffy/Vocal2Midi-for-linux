# 0077 - Implement HFA Dictionary G2P Core

Date: 2026-07-17

## Context

Record 0074 split the HFA lifecycle and placed dictionary-backed G2P after the
verified shared phoneme/mora output contract. Dependency expansion confirmed a
standard-library-only unit covering `DictionaryG2P.__init__`, `_g2p`, and the
shared `BaseG2P.__call__` assertions without dataset discovery, config parsing,
model execution, export, or a Python/Rust bridge.

The accepted 43-case Python 3.12/Linux UTF-8 JSONL table fixes constructor
snapshotting, universal newlines, whole-file and row parsing, duplicate
replacement, literal-space phone/input tokens, warning order, output indexes,
structured constructor failures, repeated calls, and recovery after conversion
errors.

## Decision

Extend the existing `v2m-core::hfa_g2p` module and reuse `HfaG2pOutput`,
`HfaG2pError`, and `apply_base_g2p_contract`. The implementation adds:

- `HfaDictionaryG2p`, which reads the path once, decodes UTF-8, applies Python
  universal-newline behavior, parses all rows before construction succeeds, and
  exposes only a borrowed dictionary snapshot plus immutable conversion;
- last-value duplicate replacement, ignored tab fields after index 1,
  Python-compatible stripping, and literal ASCII-space token preservation;
- `HfaDictionaryG2pConversion`, which retains ordered structured warnings beside
  either the shared output or shared assertion error, including warnings emitted
  before a later base-contract failure;
- `HfaDictionaryG2pWarning`, retaining the warning variant, raw input token
  position, edge-phone position where applicable, exact `UserWarning` category,
  and exact compatibility message without emitting logs;
- structured open/read, UTF-8 decode, and malformed-row load errors retaining
  operation, dictionary path, I/O source/kind, decode start/exclusive-end/leading
  byte/reason, or row/field position plus the exact Python exception projection.

The Rust fixture harness consumes the same 43-case JSONL as the real-class
Python checker. Focused Rust tests also inspect structured error fields, retain
warnings across an assertion error, prove later successful and repeated calls,
and exercise a 10,000-entry dictionary with 10,000 ordered input tokens. No
Python import, caller, production route, or bridge changed.

Initial independent behavior and error/tracing reviews found that the original
single `0xff` fixture did not cover CPython's multi-byte `UnicodeDecodeError`
projection. Rust lost truncated sequence ends and rendered a hexadecimal
singular `byte` with plural `positions`; Python renders the full span as
`bytes in position N-M`. The error/tracing review also found that missing-path
messages did not use Python filename `repr` for apostrophe and control-character
paths.

The writer fix stores explicit Python `start` and exclusive `end` fields, extends
truncated spans to the input end, and uses CPython's singular/plural message
shape. Eight added decode cases cover continuation-start, overlong, surrogate,
out-of-range, a two-byte consumed invalid continuation, and truncated three- and
four-byte sequences. Five missing-path cases cover apostrophe quote selection,
newline, tab, backslash, and printable non-ASCII. Filename projection reuses the
existing Python 3.12/Unicode 15 string-repr implementation shared with HFA Word
diagnostics. Exact filename projection remains scoped to Linux paths
representable as UTF-8; the structured path/source remains available outside
that promotion-time policy. The dependency record uses its own schema's `done`
state while the manifest correctly remains `reimplemented`.

## State

The manifest unit is `reimplemented`, not `verified`. Python
`DictionaryG2P` and `InferenceBase.get_dataset` remain the only runtime owners.

## Verification

```bash
uv run python rewrite-in-rust/bootstrap/check_hfa_dictionary_g2p_core.py
uv run python -m py_compile inference/HubertFA/tools/g2p.py
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_dictionary_g2p_core
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_phoneme_mora_g2p_core
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml
cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check
cargo clippy --manifest-path rewrite-in-rust/rust/Cargo.toml --all-targets --all-features -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --manifest-path rewrite-in-rust/rust/Cargo.toml --no-deps
uv run python scripts/audit_vendored_sources.py
git diff --check
```

## Residual Risk

Fresh independent dependency/bootstrap, stage behavior, and error/tracing review
reruns are still required. A promotion record must define non-UTF-8 path payloads,
non-UTF-8 platform policy, Python I/O/decode/assertion reconstruction, warning
presentation, caller routing, and rollback before Rust can own runtime behavior.

## Reversal

Keep Python `DictionaryG2P` and `InferenceBase.get_dataset` dictionary selection
as runtime owners. Removing the uncalled Rust dictionary types restores the
pre-unit state without changing application, GUI, Web, CLI, model, or export
behavior.
