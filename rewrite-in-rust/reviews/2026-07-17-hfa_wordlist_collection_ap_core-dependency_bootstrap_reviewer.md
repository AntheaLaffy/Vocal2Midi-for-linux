# hfa_wordlist_collection_ap_core - dependency_bootstrap_reviewer

Date: 2026-07-17
Decision: pass

Unit: `hfa_wordlist_collection_ap_core`
Role: `dependency_bootstrap_reviewer`

## Findings

No dependency/bootstrap findings.

## Boundary And Prerequisite

The unit boundary is still the decoder/pre-aggregation phase of
`inference/HubertFA/tools/align_word.py::WordList`. The dependency record
correctly separates collection/log state, append/overlap behavior, interval
subtraction, AP policy, projections, and prefix cleanup from
`fill_small_gaps`, `add_SP`, and `check`
([dependency record](/home/fuurin/code/Vocal2Midi-for-linux/rewrite-in-rust/dependencies/hfa_wordlist_collection_ap_core.yaml:4),
[bootstrap](/home/fuurin/code/Vocal2Midi-for-linux/rewrite-in-rust/bootstrap/hfa_wordlist_collection_ap_core.md:3)).
The caller lifecycle supports that split: `AlignmentDecoder.decode` constructs
and consumes `WordList` projections
([decoder.py](/home/fuurin/code/Vocal2Midi-for-linux/inference/HubertFA/tools/decoder.py:109)),
while `HubertFA.infer` adds AP words and clears prefixes before duplicate-pass
selection and aggregation, then invokes finalization later
([infer_base.py](/home/fuurin/code/Vocal2Midi-for-linux/inference/HubertFA/tools/infer_base.py:199)).

`hfa_word_model_core` is the verified prerequisite in the manifest and the
collection module uses its canonical `Phoneme`/`Word` types. The collection
record explicitly owns one heterogeneous `WordListEntry` representation and
one ordered log buffer for the later finalizer to extend
([bootstrap](/home/fuurin/code/Vocal2Midi-for-linux/rewrite-in-rust/bootstrap/hfa_wordlist_collection_ap_core.md:47)).
The planned `hfa_wordlist_finalize_core` record repeats this reuse requirement
and does not introduce a parallel interval or collection model
([manifest](/home/fuurin/code/Vocal2Midi-for-linux/rewrite-in-rust/manifest.yaml:1361)).

## Dependency And Source Audit

The selected behavior is standard-library list/string/float comparison and
the verified local model; NumPy, librosa, model execution, audio I/O, export,
and ONNX remain caller-owned. The seam is an independent Rust library with no
bridge dependencies and keeps Python as the runtime owner
([dependency record](/home/fuurin/code/Vocal2Midi-for-linux/rewrite-in-rust/dependencies/hfa_wordlist_collection_ap_core.yaml:20)).
No new crate was added for this unit: `v2m-core`'s `encoding_rs`, `md-5`, and
`serde_json` dependencies are unchanged from the pre-unit `9d2903d` baseline;
HFA production code itself uses the standard library, with `serde_json` and
`md-5` confined to the existing fixture test harness.

CPython provenance is explicit and sufficient for the pinned contract. The
dependency/bootstrap records link CPython v3.12.13 `Objects/listobject.c` and
`Objects/listsort.txt`, and the implementation documents the natural-run,
binary-insertion, powersort, gallop, and merge schedule. The generated
Unicode table identifies Python 3.12.13 / Unicode 15.0 and its generation
predicate (`not chr(cp).isprintable()`)
([generated table](/home/fuurin/code/Vocal2Midi-for-linux/rewrite-in-rust/rust/crates/v2m-core/src/python_15_nonprintable.rs:1));
the repository acknowledgement names CPython/Python Software Foundation and
links the tagged source
([ACKNOWLEDGEMENTS.md](/home/fuurin/code/Vocal2Midi-for-linux/ACKNOWLEDGEMENTS.md:22)).
The full-scalar digest fixture independently verifies the generated data.

## Fixture Coverage

The declared 45 cases are present with 45 unique case IDs. They cover raw
heterogeneous seed/extend state, append and overlap diagnostics, log lifecycle,
interval subtraction and special floats, AP branch/evaluation order and
thresholds, aliasing, projections/prefix partial mutation, and invalid-entry
errors. They also include the CPython 3.12.13 equal-key/NaN/infinity matrices,
repeated calls, 65/127/257-entry merge/gallop corpora, and the Unicode 15.0
full-scalar printability digest
([dependency record](/home/fuurin/code/Vocal2Midi-for-linux/rewrite-in-rust/dependencies/hfa_wordlist_collection_ap_core.yaml:24)).
The prerequisite model fixture/checker also passes.

## Ownership, Pin, And Rollback

The project pins Python to `>=3.12,<3.13`, `uv.lock` to `==3.12.*`, and
`.python-version` to `3.12`; the executed environment is Python 3.12.13.
The manifest remains `status: reimplemented`, `current_owner: legacy`, and
the rollback keeps Python `WordList` construction and pre-aggregation AP/prefix
behavior as runtime owners
([manifest](/home/fuurin/code/Vocal2Midi-for-linux/rewrite-in-rust/manifest.yaml:1334),
[bootstrap](/home/fuurin/code/Vocal2Midi-for-linux/rewrite-in-rust/bootstrap/hfa_wordlist_collection_ap_core.md:127)).
Static routing search found no production Python caller importing the Rust
module. Promotion still needs an explicit bridge payload, warning/error
mapping, ownership switch, and rollback record, as the dependency record
requires.

## Checks

- `UV_CACHE_DIR=/tmp/uv-cache uv run python --version`: passed, Python 3.12.13.
- `UV_CACHE_DIR=/tmp/uv-cache uv run python rewrite-in-rust/bootstrap/check_hfa_wordlist_collection_ap_core.py`: passed all 45 cases.
- `UV_CACHE_DIR=/tmp/uv-cache uv run python rewrite-in-rust/bootstrap/check_hfa_word_model_core.py`: passed.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_wordlist_collection`: passed (1 selected test).
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml word_handles_preserve_aliases_without_exposing_borrow_guards`: passed.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml`: passed (102 `v2m-core` and 5 bridge tests).
- `cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check`: passed.
- `cargo clippy --manifest-path rewrite-in-rust/rust/Cargo.toml --all-targets --all-features -- -D warnings`: passed.
- `RUSTDOCFLAGS="-D warnings" cargo doc --manifest-path rewrite-in-rust/rust/Cargo.toml --no-deps`: passed.
- `UV_CACHE_DIR=/tmp/uv-cache uv run python scripts/audit_vendored_sources.py`: passed; no source-audit errors or binary artifacts.
- YAML parse of collection, verified model prerequisite, and planned finalizer dependency records: passed.
- Static caller/routing and Cargo-baseline audit: passed; no unit-specific dependency or production-route creep.

## Residual Risk

The CPython sort contract is intentionally tied to 3.12.13 comparison
scheduling, and the repr table is tied to Unicode 15.0. A Python/Unicode
upgrade must regenerate the table and rerun the corpus/digest checks. Rust
`String` cannot represent Python lone surrogates, and invalid entries are
currently the string-valued compatibility shape used by the fixtures rather
than arbitrary Python duck-typed objects. A future bridge must define those
conversions plus diagnostic mapping and preserve the same canonical
heterogeneous entry/log storage for finalization.

## Promotion Note

This dependency/bootstrap role does not block coordinator state update for
`hfa_wordlist_collection_ap_core`. The coordinator may record this required
review as passed; behavior, data/algorithm, and error-tracing reviews remain
independent gates, and this report does not update the manifest or approve
production owner promotion.
