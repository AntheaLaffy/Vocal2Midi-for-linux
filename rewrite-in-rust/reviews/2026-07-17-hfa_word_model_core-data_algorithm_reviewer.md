# hfa_word_model_core - data_algorithm_reviewer

Date: 2026-07-17
Decision: pass

Unit: `hfa_word_model_core`
Role: `data_algorithm_reviewer`

## Findings

No data representation, numeric, or algorithm findings.

Evidence:

- The canonical representation is appropriately narrow: `Phoneme` is three owned fields and `Word` owns its interval, text, and `Vec<Phoneme>` (`rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:66`, `rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:99`). No `WordList` or parallel interval type appears in the module. The next unit is required to introduce the sole collection/log storage over these types, and finalization must extend that same collection (`rewrite-in-rust/records/0066-split-hfa-word-lifecycle.md:46`, `rewrite-in-rust/manifest.yaml:1329`, `rewrite-in-rust/manifest.yaml:1353`). This avoids conversion layers and preserves one owner for local Word invariants.
- Constructor numeric behavior matches Python's operation order. Python first evaluates `max(0.0, start)`, which turns negative values, negative infinity, negative zero, and NaN into positive `0.0`, while retaining positive infinity; Rust's clamp encodes the same branches before the strict `start < end` test (`inference/HubertFA/tools/align_word.py:11`, `inference/HubertFA/tools/align_word.py:28`, `rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:72`, `rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:106`, `rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:248`). Shared fixtures cover NaN on both sides, both infinities, negative zero, reversed/equal intervals, initialized phonemes, exact errors, and infinite duration (`rewrite-in-rust/fixtures/hfa_word_model_core.jsonl:1`, `rewrite-in-rust/fixtures/hfa_word_model_core.jsonl:2`, `rewrite-in-rust/fixtures/hfa_word_model_core.jsonl:3`, `rewrite-in-rust/fixtures/hfa_word_model_core.jsonl:4`).
- Boundary comparison and short-circuit order are preserved. Python `move_start` evaluates `0 <= new_start` before indexing the first phoneme; Rust returns the same warning for negative/negative-infinity/NaN inputs before accessing the vector, then indexes for nonnegative values. Python `move_end` must access the last phoneme in its first comparison, and Rust does the same (`inference/HubertFA/tools/align_word.py:91`, `inference/HubertFA/tools/align_word.py:102`, `rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:200`, `rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:225`). Fixtures now cover nonempty special floats plus empty-phoneme warning and `IndexError` branches through both log and warning sinks (`rewrite-in-rust/fixtures/hfa_word_model_core.jsonl:8`, `rewrite-in-rust/fixtures/hfa_word_model_core.jsonl:9`, `rewrite-in-rust/fixtures/hfa_word_model_core.jsonl:10`, `rewrite-in-rust/fixtures/hfa_word_model_core.jsonl:14`).
- Duration and boundary synchronization are direct and equivalent. Duration is raw `end - start`; successful append replaces `Word.end` with the appended phoneme end; successful moves update the Word and the first/last phoneme edge together (`inference/HubertFA/tools/align_word.py:41`, `inference/HubertFA/tools/align_word.py:62`, `inference/HubertFA/tools/align_word.py:91`, `rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:145`, `rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:170`, `rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:200`). Neither implementation silently revalidates public fields after construction.
- Add and append preserve raw IEEE comparison semantics and diagnostic order. Both test zero length first. Add then checks inclusive containment; append checks exact adjacency and grows the end. Rejections leave state unchanged and deliver one exact message either as `WARNING: ...` in the supplied list or as a `UserWarning` projection (`inference/HubertFA/tools/align_word.py:45`, `inference/HubertFA/tools/align_word.py:62`, `rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:150`, `rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:170`, `rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:256`). The shared table verifies normal branches and ordering; additional read-only probes manually mutated public fields to NaN and reversed intervals and confirmed Python follows the same branches visible in Rust: NaN containment rejects, reversed contained phonemes can be stored, and exactly adjacent invalid phonemes can propagate a reversed or NaN `Word.end`.
- Exact constructor float rendering handles NaN, infinities, negative zero, ordinary decimal output, and Python-style signed/zero-padded exponents (`rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:265`). Cross-language fixtures assert the special values and ordinary error strings, while the targeted Rust regression covers `1e20`, `1e-7`, and `-0.0` (`rewrite-in-rust/rust/crates/v2m-core/src/hfa_word.rs:546`).
- Complexity is appropriate. Constructors, duration, comparisons, edge mutations, and warning selection are O(1) apart from text allocation/formatting; phoneme insertion is amortized O(1). No scan, sorting, model computation, or collection algorithm has leaked into this prerequisite unit. Later WordList algorithms can operate directly on the canonical owned values.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_hfa_word_model_core.py`: passed all 14 fixture cases.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_word -- --nocapture`: passed; both selected tests passed.
- `UV_CACHE_DIR=/tmp/vocal2midi-uv-cache uv run python -c <mutated-public-field and alias probe>`: passed; exercised NaN/reversed Phoneme fields, invalid append/end propagation, invalid edge fields, and Python object-alias visibility without model execution.
- `cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check`: passed.
- `cargo clippy --manifest-path rewrite-in-rust/rust/Cargo.toml --all-targets --all-features -- -D warnings`: passed.
- `git diff --check`: passed.
- Static type/caller search: the Rust module defines only canonical `Phoneme`/`Word`; production decoder, infer, and API callers remain on the Python types.

## Residual Risk

Python stores the exact Phoneme object supplied to add/append, so an external alias can observe later boundary moves or mutate the stored object. Rust deliberately takes owned `Phoneme` values, and no current decoder/infer/API caller relies on external identity after insertion. A future Python bridge must explicitly choose copy/identity semantics instead of claiming object-identity parity implicitly.

The review does not attempt arbitrary Python duck-typed values, NumPy scalar string formatting, or exhaustive proof over every finite `f64` rendering. WordList overlap/AP/gap/SP/check behavior remains in the two later canonical-owner units.

## Promotion Note

This data/algorithm role does not block coordinator state update for `hfa_word_model_core`. The report does not by itself justify production promotion; bridge ownership and alias/error/warning mapping remain future promotion decisions.
