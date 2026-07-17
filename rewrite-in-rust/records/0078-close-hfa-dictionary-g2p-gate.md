# 0078 - Close HFA Dictionary G2P Gate

Date: 2026-07-17

## Context

Record 0074 split dictionary-backed G2P from pure phoneme/mora conversion,
config parsing, and export behavior. Record 0077 implemented the confirmed
dictionary unit behind the independent Rust library seam and fixed the
multi-byte decode and filename-representation gaps found by the first
independent reviews.

The current passing evidence is:

- dependency/bootstrap rerun:
  `rewrite-in-rust/reviews/2026-07-17-hfa_dictionary_g2p_core-dependency_bootstrap_reviewer-rerun.md`
- behavior rerun:
  `rewrite-in-rust/reviews/2026-07-17-hfa_dictionary_g2p_core-behavior_reviewer-rerun.md`
- error/tracing rerun:
  `rewrite-in-rust/reviews/2026-07-17-hfa_dictionary_g2p_core-error_tracing_reviewer-rerun.md`

All three required roles returned `pass` with no findings. The historical
`pass-with-followups` and failed reports remain as audit evidence.

## Decision

Accept `hfa_dictionary_g2p_core` as verified for the current legacy-owned,
no-bridge Rust library seam.

The verified unit preserves:

- one-time Linux UTF-8 dictionary snapshots, Python universal newlines,
  whole-file/row stripping, tab-field selection, literal-space phone tokens,
  duplicate replacement, and all-or-nothing malformed-row errors;
- literal-space input lookup, missing-word omission, accepted-word index
  advancement, exact edge-SP and missing-word warning order/text, interior SP,
  language prefixing, output order, and shared Base assertions;
- structured open/read/decode/parse context, CPython single- versus multi-byte
  decode spans, truncated sequence ends, exact supported UTF-8 filename repr,
  warning retention across assertion errors, repeated calls, and recovery;
- the shared `HfaG2pOutput`, `HfaG2pError`, and Python 3.12/Unicode-15 string
  representation primitives without another crate, bridge, or runtime route.

The shared cross-runtime table contains 43 cases. Independent rerun probes
matched nine decode behavior cases and sixteen decode error/tracing cases,
including invalid continuation and truncated three- and four-byte sequences.

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

The shared Python checker passes all 43 cases. The focused Rust gate passes four
dictionary tests, the prerequisite G2P gate passes three tests, and the full
workspace passes 113 `v2m-core` tests plus five bridge tests.

## Residual Risk

Exact filename projection is limited to Linux paths representable as UTF-8.
Promotion must choose and fixture a non-UTF-8 path payload policy, reconstruct
Python warnings and exceptions without parsing display strings, define
diagnostic display/retention, route callers, and prove rollback. Dictionary map
iteration order is not part of the selected conversion payload.

## Reversal

Rollback remains keeping Python `DictionaryG2P` and
`InferenceBase.get_dataset` dictionary selection as runtime owners. No Python
caller imports the Rust converter, and no GUI, Web, CLI, model, config, or export
path changed.
