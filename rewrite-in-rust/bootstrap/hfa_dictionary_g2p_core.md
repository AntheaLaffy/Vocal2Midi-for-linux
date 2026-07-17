# hfa_dictionary_g2p_core Bootstrap

## Boundary

Cover `DictionaryG2P.__init__`, `DictionaryG2P._g2p`, and the verified
`BaseG2P.__call__` output/prefix contract. Include one-time dictionary file IO,
row parsing, immutable lookup state, ordered warnings, word-index assignment,
and shared SP assertions. These behaviors form one independently verifiable
library object and do not need a Python/Rust runtime bridge.

Exclude dictionary path selection, vocab/config parsing, wav/lab discovery,
dataset mutation, decoder/model execution, alignment, export, and warning or
exception reconstruction at a Python bridge. `InferenceBase.get_dataset` and
`inference/API/hfa_api.py` remain caller evidence only.

## Dependency Expansion

`inference/HubertFA/tools/g2p.py` imports only `pathlib` and `warnings` for this
class. The selected path does not call pyopenjtalk, PyYAML, textgrid, numpy,
librosa, ONNX Runtime, or any native/FFI capability. `pyproject.toml`, all three
requirements views, `uv.lock`, the source manifests, and
`third_party/source_audit.json` therefore justify a standard-library-backed
narrow replacement; no crate, vendored package, or bridge dependency is added.

Python opens the dictionary with locale-default text mode and universal newline
translation. The supported project environment is Python 3.12 on Linux with a
UTF-8 locale. The fixture gate asserts both facts before binding invalid-UTF-8
and OS error messages. A future non-UTF-8 platform would require a separate
compatibility decision rather than silently changing this contract.

Exact filename compatibility projection is likewise fixture-bound to Linux paths
representable as UTF-8. It uses Python 3.12/Unicode 15 string `repr`: quote choice,
backslash and control escaping, and printable non-ASCII retention are observable.
The original `PathBuf` and `io::Error` remain structured for other paths, but a
future bridge must define non-UTF-8 path encoding rather than treating the
fallback display as a compatibility promise.

The exact parser and lookup order is:

1. Read the complete file once, apply Python `str.strip()`, then split on
   newline. Empty and whitespace-only files therefore produce one malformed
   row rather than an empty dictionary.
2. For each row, split on tab, require indexes 0 and 1, ignore later fields,
   strip the key/value, and split phones on a literal ASCII space. Empty phone
   tokens are retained. Duplicate keys replace the value from the earlier row.
3. Strip input text and split on a literal ASCII space. Every missing token,
   including an empty token created by repeated spaces, emits an ordered
   `UserWarning` and does not consume a word index.
4. A dictionary hit enters `word_seq` and consumes an index even when all its
   phones are rejected edge `SP`. Each first/last `SP` emits its own warning;
   interior `SP` is retained. The shared base contract then validates SP layout
   and prefixes non-SP phones when language is not `None`.

## Seam

- crate/module: extend `v2m-core::hfa_g2p`; do not create a second output type
- prerequisite: verified `hfa_phoneme_mora_g2p_core` `HfaG2pOutput` and shared
  base-contract errors
- kind: independent Rust library
- runtime owner: legacy Python
- bridge dependencies: none
- load input: dictionary path plus nullable language
- load output: immutable converter or structured open/read/decode/row error
- convert input: borrowed UTF-8 input text
- convert output: `HfaG2pOutput` plus ordered `HfaDictionaryG2pWarning` values,
  or a shared assertion error

The Rust error must retain operation, path where applicable, source/kind, and a
Python compatibility projection. Malformed rows project to `IndexError` with
`list index out of range`. Decode failures retain an explicit Python start and
exclusive end, leading byte, source, and reason. A one-byte span renders
`byte 0x.. in position N`; a longer invalid or truncated span renders
`bytes in position N-M`, with truncation extending to the input end. Shared SP
failures retain the empty-message `AssertionError`. Rust warnings are data, not
emitted process logs. Each warning must retain the exact Python category and
message so a later bridge can reconstruct presentation without parsing strings.

Loading snapshots the file. Repeated conversions do no filesystem IO, do not
mutate the dictionary, repeat the same ordered warnings/results, and remain
usable after a conversion-level assertion error.

## Fixture Harness

`rewrite-in-rust/fixtures/hfa_dictionary_g2p_core.jsonl` is a static 43-case
Python 3.12 Linux/UTF-8 golden table. Every case supplies an inline dictionary
file (text or hex bytes), a missing/directory path mode and optional filename,
language, zero or more calls, and an exact expected parsed map, output arrays,
ordered warning category/text, or normalized exception type/message. The real
legacy class is checked through temporary files:

```bash
uv run python rewrite-in-rust/bootstrap/check_hfa_dictionary_g2p_core.py
```

The table covers ordinary and Unicode rows; `None`, empty, and string language;
whole-file/key/value trimming; duplicate keys; ignored extra tab fields;
literal repeated phone/input spaces; empty phones and an interior empty key;
CRLF and lone-CR universal newlines; missing and empty input words; edge,
interior, and consecutive SP; exact warning order; empty/whitespace/malformed
and invalid-UTF-8 files; missing/directory paths; repeated calls; constructor
snapshotting after file replacement; and recovery after a conversion error.
Invalid UTF-8 cases distinguish invalid start, immediate and multi-byte invalid
continuation, overlong, surrogate, out-of-range, and two-/three-byte truncated
spans. Missing-path cases bind Python filename `repr` for apostrophe, newline,
tab, backslash, and printable non-ASCII UTF-8 names.

The writer gate is this Python fixture command plus the verified shared-output
prerequisite test:

```bash
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_dictionary_g2p_core
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_phoneme_mora_g2p_core
```

## Rollback

Keep Python `DictionaryG2P` and `InferenceBase.get_dataset` dictionary routing
as runtime owners. No production import changes during implementation. Promotion
must separately define payload validation, path and decode error reconstruction,
warning presentation, caller routing, and rollback.
