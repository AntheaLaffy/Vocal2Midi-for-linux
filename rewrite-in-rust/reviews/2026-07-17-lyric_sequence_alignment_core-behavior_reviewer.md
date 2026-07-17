# lyric_sequence_alignment_core - behavior_reviewer

Date: 2026-07-17
Decision: pass

## Findings

No findings.

Evidence:

- Edit-distance DP and backtrack parity are preserved. Python selects substitute first and only changes to delete/insert on strictly lower cost at `inference/LyricFA/tools/sequence_aligner.py:48`, then backtracks the stored operations at `inference/LyricFA/tools/sequence_aligner.py:63`. Rust mirrors the strict tie behavior at `rewrite-in-rust/rust/crates/v2m-core/src/lyric_sequence.rs:77` and the same backtrack output shape at `rewrite-in-rust/rust/crates/v2m-core/src/lyric_sequence.rs:518`. Fixture lines 1-5 cover empty insert/delete rows, substitute tie retention, delete-over-insert custom costs, and mixed insertion/substitution.
- Exact match, LCS/window prefilter, inclusive overlap threshold, and long-input/no-window reason strings match the legacy behavior. Python exact-match and reason handling are at `inference/LyricFA/tools/sequence_aligner.py:129` and `inference/LyricFA/tools/sequence_aligner.py:142`; Rust mirrors them at `rewrite-in-rust/rust/crates/v2m-core/src/lyric_sequence.rs:138` and `rewrite-in-rust/rust/crates/v2m-core/src/lyric_sequence.rs:153`. Python rejects candidate windows only when `coverage < OVERLAP_THRESHOLD` at `inference/LyricFA/tools/sequence_aligner.py:240`; Rust uses the same strict-less-than check at `rewrite-in-rust/rust/crates/v2m-core/src/lyric_sequence.rs:251`, so equality is accepted. Fixture lines 8-15 cover exact match, no overlap, approximate first tie, long-input success/failure reasons, first-tie scan retention, inclusive threshold equality, and empty alignment helper output.
- Candidate sorting, pruning, and first-tie retention match. Python sorts by approximate distance, keeps `max(10, int(0.3 * total))`, and updates the selected start only on strict edit-distance improvement at `inference/LyricFA/tools/sequence_aligner.py:250`. Rust uses the same retained count and strict improvement rule at `rewrite-in-rust/rust/crates/v2m-core/src/lyric_sequence.rs:264`. Fixture lines 10, 13, and 14 exercise first-tie and threshold behavior.
- Public return shapes and helper outputs match the Python caller surface. Python `find_best_match` returns `(matched_text, start, end, matched_phonetic_list, matched_text_list, reason)` at `inference/LyricFA/tools/sequence_aligner.py:121`; `find_best_match_and_return_lyrics` returns `(matched_text, matched_phonetic, start, end, reason)` at `inference/LyricFA/tools/sequence_aligner.py:301`; `calculate_difference_count` counts zipped mismatches plus length delta at `inference/LyricFA/tools/sequence_aligner.py:316`. Rust exposes equivalent result fields and difference counting at `rewrite-in-rust/rust/crates/v2m-core/src/lyric_sequence.rs:375`, `rewrite-in-rust/rust/crates/v2m-core/src/lyric_sequence.rs:399`, and `rewrite-in-rust/rust/crates/v2m-core/src/lyric_sequence.rs:434`. Fixture lines 16-19 cover return-lyrics shape, LCS, edit distance, and difference count.
- SmartHighlighter parenthesized output and text-index behavior match. Python advances the text index only for non-gap match tokens and omits empty highlight slots at `inference/LyricFA/tools/sequence_aligner.py:346`; Rust mirrors that behavior at `rewrite-in-rust/rust/crates/v2m-core/src/lyric_sequence.rs:475` and joins non-empty slots at `rewrite-in-rust/rust/crates/v2m-core/src/lyric_sequence.rs:510`. Fixture lines 20-24 cover empty match phonetic, substitution, ASR extra/delete, match extra/insert, and short text-token indexing.
- Rollback and runtime ownership remain legacy Python, with no production lyric bridge introduced. The manifest keeps `current_owner: legacy` and rollback to `SequenceAligner`/`SmartHighlighter` at `rewrite-in-rust/manifest.yaml:1151`; the bootstrap record states runtime owner is legacy Python and bridge dependencies are none at `rewrite-in-rust/bootstrap/lyric_sequence_alignment_core.md:50`; the boundary record states no production bridge was introduced at `rewrite-in-rust/records/0055-confirm-lyric-sequence-alignment-boundary.md:78`. `rewrite-in-rust/rust/crates/v2m-core/src/lib.rs:17` only exports the independent Rust module, and `rg -n "lyric_sequence|SequenceAligner|SmartHighlighter|calculate_difference_count|v2m_core|v2m-core" inference application web_server.py tests` found only the existing legacy Python imports/callers.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_lyric_sequence_alignment_core.py`: passed.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml lyric_sequence`: passed, 2 tests passed in `v2m-core`; 0 matching tests in `v2m-quant-bridge`.
- `git diff --check`: passed.

## Residual Risk

The parity proof is fixture-bound. It covers the public behaviors named by the unit record, but it is not an exhaustive proof over arbitrary token alphabets, custom cost combinations, `max_window_scale` values, or malformed future bridge payloads. Runtime promotion still needs its own payload validation, logging text, and Python-facing error mapping before any production caller routes lyric alignment through Rust.

## Promotion Note

This behavior review does not block coordinator state update from the behavior-parity perspective. The coordinator should keep Python as the runtime owner and should only mark broader unit verification after the required independent review roles for this unit are satisfied.
