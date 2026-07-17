# hfa_phoneme_mora_g2p_core - dependency_bootstrap_reviewer

Date: 2026-07-17
Decision: pass

Unit: `hfa_phoneme_mora_g2p_core`
Role: `dependency_bootstrap_reviewer`

## Findings

No dependency/bootstrap findings.

The manifest boundary is **confirmed**. This unit should not be split, merged,
deferred, or replaced. Record 0074 correctly replaces the former mixed
`hfa_g2p_export_config_core` candidate with six independently testable and
rollbackable units (`rewrite-in-rust/records/0074-split-hfa-g2p-config-export-lifecycle.md:14`):

1. pure Base/raw-phoneme/Japanese-mora G2P;
2. dictionary file parsing, lookup, and warning policy;
3. PyYAML-backed config loading and filesystem validation;
4. HTK content/path planning and cumulative prediction state;
5. textgrid 1.6.1 serialization/path planning;
6. export membership/default/order dispatch.

The structured manifest audit found all six replacement IDs, found no remaining
old mixed-unit entry, and confirmed inventory states
`confirmed, confirmed, provisional, confirmed, confirmed, confirmed`. Keeping
only `hfa_config_file_contract_core` provisional is justified: its arbitrary
PyYAML `safe_load` value/error surface still needs a maintained Rust parser and
documented compatibility limits. The other five boundaries have distinct
inputs, effects, dependencies, callers, and reversal points; combining them
would hide rather than reduce those contracts.

The selected first writer route is the minimum pure seam. Legacy
`InferenceBase.get_dataset` selects `PhonemeG2P` or
`JapanesePhonemeMoraG2P` at `inference/HubertFA/tools/infer_base.py:155`, invokes
the chosen object only after reading a lab string at line 175, and separately
owns wav discovery, dataset mutation, warning/error presentation, and later
model execution. `hfa_api.py` merely chooses `phoneme` versus
`ja_mora_phoneme` from language and caller intent at
`inference/API/hfa_api.py:127`. The Rust unit therefore correctly accepts only
UTF-8 text plus nullable language and returns the three ordered arrays or the
shared Base-contract error; it does not absorb mode selection, paths, files,
dataset state, cancellation, decoder/model work, or API/CLI routing.

Capability coverage is complete for that seam. The dependency record names
`BaseG2P.__call__`, `PhonemeG2P`, and `JapanesePhonemeMoraG2P` separately and
maps them to `apply_base_g2p_contract`, `phoneme_g2p`, and
`japanese_phoneme_mora_g2p`
(`rewrite-in-rust/dependencies/hfa_phoneme_mora_g2p_core.yaml:3`). The public
Rust output retains phonemes, words, and signed phoneme-to-word indexes; the
structured error retains exact `IndexError` and empty-message `AssertionError`
projections (`rewrite-in-rust/rust/crates/v2m-core/src/hfa_g2p.rs:17`). The
implementation covers Base SP boundary/consecutive validation and language
prefixing, raw literal-space behavior, Python strip controls, Japanese control
tokens, mora grouping, canonicalization, fallback order, output ordering, and
index construction without adding a parallel contract.

No dependency is missing from the pure implementation. Although legacy
`g2p.py` imports `pathlib` and `warnings`, source inspection shows both are used
only by `DictionaryG2P` beginning at
`inference/HubertFA/tools/g2p.py:177`; the three selected classes use built-in
string/list/assertion behavior only. Production `hfa_g2p.rs` imports only
`std::error` and `std::fmt`; `serde_json` appears solely in the existing test
harness. Cargo manifest and lockfile diffs are empty, and the seam has no bridge
dependency.

The wider repository dependency evidence supports, rather than leaks into, the
re-cut. `requirements.txt` and `pyproject.toml` declare PyYAML and textgrid;
`uv.lock` pins PyYAML 6.0.3 and textgrid 1.6.1 at lines 3701 and 4074.
`third_party/sources/manifest.json` maps those exact versions to existing
`third_party/sources/pyyaml-6.0.3` and
`third_party/sources/textgrid-1.6.1` trees. PyYAML remains assigned to the
provisional config unit, while the pinned textgrid writer source is assigned to
the TextGrid unit. Neither belongs in raw-phoneme/mora G2P. HTK rendering uses
standard numeric/string/path behavior in its own unit, and dispatch composes
exporters only after their separate gates.

The hand-written replacement choice is appropriate. The behavior is a small,
repository-specific set of tables and ordered string transforms whose primary
source is the local `g2p.py`; adding a general G2P, Japanese frontend, model, or
FFI package would enlarge the semantic surface without covering the observed
contract. `apply_base_g2p_contract` is public and reusable by the following
dictionary unit, preserving the record's implementation order without joining
dictionary file/warning ownership to this unit. No unsafe code, model asset,
network, filesystem, subprocess, PyO3, or runtime router is introduced.

The fixture gate is adequate for dependency/bootstrap promotion. The 25 unique
rows divide into 9 raw-phoneme, 10 mora, and 6 injected Base-contract cases.
They cover empty and literal repeated spaces, non-space whitespace, Python
U+001C stripping, exact SP errors, nullable/empty/non-empty language prefixes,
SP/AP/EP filtering, N/cl/I/U normalization, precomposed and separated shapes,
hu/fy/multi-vowel and palatal fallbacks, unknown/case behavior, all output
arrays, and exact index maps
(`rewrite-in-rust/fixtures/hfa_phoneme_mora_g2p_core.jsonl:1`). The checker
imports the real legacy classes; only Base's abstract `_g2p` output is injected
to isolate its public contract
(`rewrite-in-rust/bootstrap/check_hfa_phoneme_mora_g2p_core.py:15`). It also
checks repeated calls, while Rust consumes the same JSONL and adds repeated-call
and 10,000-token scaling regressions at
`rewrite-in-rust/rust/crates/v2m-core/src/hfa_g2p.rs:432`.

No warning, file, model, or effect seam is omitted. Missing-word and edge-SP
warnings are emitted only by `DictionaryG2P` and are explicitly assigned to
`hfa_dictionary_g2p_core`. Dictionary open/parse/path behavior stays there;
wav/lab discovery and dataset mutation stay in `InferenceBase`; YAML and config
file existence stay in the config unit; directory creation/writes and status
printing stay legacy-owned behind HTK/TextGrid planning; export defaults and
fixed call order stay in dispatch. ONNX sessions, decoding, aggregation,
artifact copying, GUI/Web/CLI routing, and production warning presentation are
all explicitly kept legacy.

Documentation and ownership facts are consistent. The dependency status is
`done`, bootstrap and record 0075 name the implemented Rust APIs and the same
25-row gate, and manifest status is `reimplemented` with
`current_owner: legacy` at `rewrite-in-rust/manifest.yaml:1409`. Static routing
search found only the exported Rust module declaration and no production caller
of it. Rollback is concrete: keep the three Python classes as the only runtime
owners and remove the uncalled Rust module without changing application
behavior. Writer/reviewer separation is intact; this review changed no
production code, fixtures, checker, dependency/bootstrap artifact, record,
control-plane file, or manifest.

## Checks

- Structured YAML/JSONL audit: passed. All six replacement units exist, the old mixed ID is absent, inventory states match record 0074, all owners are legacy, dependency status is `done`, bridge dependencies are empty, and all 25 fixture IDs are unique with expected results.
- `env UV_CACHE_DIR=/tmp/uv-cache-hfa-g2p-dep-review uv run python --version`: Python 3.12.13.
- `env UV_CACHE_DIR=/tmp/uv-cache-hfa-g2p-dep-review uv run python rewrite-in-rust/bootstrap/check_hfa_phoneme_mora_g2p_core.py`: validated all 25 legacy fixtures and repeated-call stability.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_phoneme_mora_g2p_core -- --nocapture`: passed all 3 focused tests: shared table, repeated calls, and 10,000-token scaling.
- `env UV_CACHE_DIR=/tmp/uv-cache-hfa-g2p-dep-review uv run python scripts/audit_vendored_sources.py`: passed; 135 Python packages, 41 native-extension packages, 269 foreign-runtime native binaries, and 0 third-party binary artifacts.
- Requirements/lock/source audit: passed. PyYAML 6.0.3 and textgrid 1.6.1 lock entries match their vendored source directories; neither is imported by the selected Rust production seam.
- `env UV_CACHE_DIR=/tmp/uv-cache-hfa-g2p-dep-review uv run python -m py_compile inference/HubertFA/tools/g2p.py inference/HubertFA/tools/infer_base.py inference/API/hfa_api.py`: passed.
- Cargo dependency and static import audit: passed. No Cargo manifest/lock dependency diff; production `hfa_g2p.rs` uses only `std`, and no production Rust route was found.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml`: passed 108 `v2m-core` tests, 5 bridge tests, and doc tests.
- `cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check`: passed.
- `cargo clippy --manifest-path rewrite-in-rust/rust/Cargo.toml --all-targets --all-features -- -D warnings`: passed.
- `RUSTDOCFLAGS='-D warnings' cargo doc --manifest-path rewrite-in-rust/rust/Cargo.toml --no-deps`: passed.
- `git diff --check`: passed after this report was written.

## Residual Risk

The shared table is finite. It covers each dependency-relevant behavior family
but does not exhaust every hard-coded mora onset/join combination, every Unicode
case/whitespace code point, or arbitrary malformed injected Base output shape.
Those table-level and algorithmic questions remain for the independent behavior
and data/algorithm gates; they do not require another dependency or a wider
unit boundary.

The next dictionary unit must actually reuse the public output/Base contract
and independently prove file encoding/parsing, duplicate keys, malformed rows,
literal empty tokens, warning category/order/text, and IO errors. Config must
remain provisional until its YAML parser decision is recorded. HTK/TextGrid
planning must not silently take filesystem ownership, and dispatch must not
absorb API/CLI/model routing.

No production bridge exists. A later owner switch must define Python/Rust
payload validation, exact exception projection, nullable/empty language
encoding, signed index transport, caller mode selection, warning presentation,
rollback, and how lab/file context is attached outside this pure unit.

## Promotion Note

This dependency/bootstrap review passes and does not block coordinator state
update for `hfa_phoneme_mora_g2p_core`. The manifest boundary is confirmed as
the first pure unit of the six-unit re-cut. The coordinator must still obtain
and evaluate the independent behavior and data/algorithm reviews before marking
the unit verified. This report does not approve production routing or modify
runtime ownership or the manifest.
