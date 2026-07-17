# hfa_dictionary_g2p_core - error_tracing_reviewer rerun

Date: 2026-07-17
Decision: pass

Unit: `hfa_dictionary_g2p_core`
Role: `error_tracing_reviewer`

## Findings

No findings.

The initial high-severity decode finding is resolved. `Decode` now retains an
explicit Python start and exclusive end, leading byte, reason, original
`Utf8Error`, operation, and path at
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_g2p.rs:114`. Construction derives
`end` from `start + error_len` for ordinary invalid spans and from the input
length for truncated spans at lines 300 through 314. The compatibility
projection at lines 167 through 185 uses CPython's singular
`byte 0x.. in position N` form for one byte and plural
`bytes in position N-M` form for longer invalid or truncated spans.

I independently compiled a temporary probe that imports the current source and
compared it with Python 3.12.13. All 16 tested invalid inputs matched in start,
exclusive end, leading byte, reason, and complete message: invalid `0xff`, a
continuation byte used as a start, immediate and later invalid continuations,
two- and three-byte overlong sequences, a surrogate, an out-of-range scalar, an
invalid `f5` lead, and one-/two-/three-byte truncated sequences. In particular,
`e2 82 41` produced `bytes in position 5-6: invalid continuation byte`, while
truncated `e2 82`, `f0 90`, and `f0 90 80` retained ends 7, 7, and 8 and exact
`unexpected end of data` projections. Fixture lines 25 through 33 now bind all
of the initially missing error shapes, and the structured test at
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_g2p.rs:1107` directly inspects a
single-byte error, the two-byte invalid-continuation span, and the three-byte
truncated span.

The initial filename-representation finding is also resolved for the declared
Linux UTF-8 path boundary. `python_io_message()` routes valid UTF-8 paths through
the shared Python 3.12/Unicode-15 `string_repr` implementation at
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_g2p.rs:746`. That implementation
selects quotes, escapes backslash and control characters, applies the pinned
Unicode-15 printability table, and retains printable non-ASCII text at
`rewrite-in-rust/rust/crates/v2m-core/src/python_15_nonprintable.rs:737`.
Independent current-Rust/Python probes matched exact `FileNotFoundError`
messages for plain, apostrophe, newline, tab, backslash, printable Chinese, and
post-Unicode-15 U+323B0 filenames. Fixture lines 34 through 39 bind the first six
supported path classes, and the shared repr unit test additionally checks quote,
control, nonprintable-width, printable Unicode, and U+323B0 behavior.

Structured context remains intact outside compatibility strings. I/O errors
retain `PathBuf`, `io::Error`, kind/raw OS source, and open/read operation;
decode errors retain source plus the fields above; malformed rows retain parse
operation, path, zero-based row index, and field count
(`rewrite-in-rust/rust/crates/v2m-core/src/hfa_g2p.rs:108`). `Error::source()`
returns the underlying I/O or UTF-8 error at line 198. The shared base error
retains exact `IndexError`/`AssertionError` type/message and is returned beside
already-produced warnings in `HfaDictionaryG2pConversion` at line 259.

Warnings remain ordered data, not process logs. Missing-word warnings retain the
raw input position and word; edge-SP warnings also retain the dictionary-phone
position; both expose exact `UserWarning` category/message and convert operation
at `rewrite-in-rust/rust/crates/v2m-core/src/hfa_g2p.rs:208`. The values can
contain lyric text and paths, but static inspection found no stdout, stderr,
tracing, logging-framework, file, bridge, or telemetry sink. A future bridge
must classify these compatibility values before display or external retention.

Repeated/error-recovery behavior remains sound. The converter snapshots the
file once, retains warnings emitted before a later assertion, remains usable
after that error, and repeats identical subsequent results and warnings, as
proved at `rewrite-in-rust/rust/crates/v2m-core/src/hfa_g2p.rs:1201`. The unit
stays inside its confirmed library boundary: production
`inference/HubertFA/tools/infer_base.py:13` and line 162 still import and create
the legacy Python `DictionaryG2P`; no Rust route or owner switch exists. Writer
and reviewer separation is intact. This rerun preserves the initial fail report
and changes only this report.

## Checks

- `env UV_CACHE_DIR=/tmp/uv-cache-hfa-dict-error-rerun uv run python rewrite-in-rust/bootstrap/check_hfa_dictionary_g2p_core.py`: passed all 43 current Python 3.12/Linux UTF-8 golden cases.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_dictionary_g2p_core -- --nocapture`: passed all 4 focused Rust tests; 109 tests were filtered out.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_dictionary_g2p_core_retains_structured_load_context -- --nocapture`: passed explicit I/O, single/multi-byte decode, truncated-decode, and malformed-row context assertions.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_dictionary_g2p_core_repeats_and_recovers_after_assertion_errors -- --nocapture`: passed warning retention, recovery, and repeated-call assertions.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml python_15_string_repr_covers_quotes_controls_and_unicode -- --nocapture`: passed the shared Python 3.12/Unicode-15 repr test.
- Independent Python 3.12.13/current-Rust decode probe: all 16 invalid-continuation, overlong, surrogate, out-of-range, invalid-lead, and truncated cases matched exactly.
- Independent Python 3.12.13/current-Rust path probe: plain, apostrophe, newline, tab, backslash, Chinese UTF-8, and U+323B0 cases matched exactly.
- Independent non-UTF-8 Rust path probe: retained the original path bytes, `open` operation, `NotFound` source kind, and `FileNotFoundError` type without panic; its fallback display is not treated as a Python compatibility projection.
- Static sink/routing inspection: no Rust diagnostic sink or production routing change was found; Python remains runtime owner.
- `cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check`: passed.
- `git diff --check`: passed before this report was written.

## Residual Risk

Exact filename compatibility is deliberately limited to Linux paths
representable as UTF-8, as recorded in
`rewrite-in-rust/bootstrap/hfa_dictionary_g2p_core.md:31`. A direct non-UTF-8
probe confirmed that Rust retains the original path bytes and source context,
but Python itself has distinct messages for a bytes path (`b'...\\xff'`) and a
surrogate-decoded string path (`'...\\udcff'`). Promotion must choose and test
one payload/encoding policy rather than labeling the Rust fallback display as
exact compatibility.

The shared empty-message `InvalidSilenceLayout` projection still does not name
the offending word or SP pair. The conversion type makes the operation implicit
and retains prior warnings; promotion may add separate non-compatibility trace
context but must not change the Python projection.

There is no production bridge. Python warning/error reconstruction, sensitive
diagnostic display and retention policy, caller routing, non-UTF-8 platform
policy, and rollback execution remain promotion work.

## Promotion Note

This error/tracing rerun passes and does not block coordinator verification of
`hfa_dictionary_g2p_core`. The coordinator may combine this report with the
fresh passing dependency/bootstrap and behavior reruns before updating the unit
state. This report does not approve production routing or a runtime-owner
change; rollback remains keeping Python `DictionaryG2P` and
`InferenceBase.get_dataset` dictionary selection as runtime owners.
