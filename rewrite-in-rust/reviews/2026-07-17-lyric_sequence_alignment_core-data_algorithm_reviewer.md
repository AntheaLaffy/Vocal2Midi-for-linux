# lyric_sequence_alignment_core - data_algorithm_reviewer

Date: 2026-07-17
Decision: pass-with-followups

## Findings

- Severity: low
- Location: rewrite-in-rust/fixtures/lyric_sequence_alignment_core.jsonl:13
- Issue: The fixture table proves the main parity path, but it does not fully prove every algorithmic claim made for the scan-window data path. The direct scan fixtures cover first-tie retention and inclusive overlap threshold, but not duplicate-token `Counter` min-count overlap, no-candidate `ScanResult` shape, the 30%-or-10 pruning boundary, or the zero-distance early break.
- Evidence: Python implements frequency overlap, stable approximate sorting, pruning, strict edit-distance replacement, and zero-distance break in `inference/LyricFA/tools/sequence_aligner.py:237`, `inference/LyricFA/tools/sequence_aligner.py:250`, `inference/LyricFA/tools/sequence_aligner.py:253`, `inference/LyricFA/tools/sequence_aligner.py:263`, and `inference/LyricFA/tools/sequence_aligner.py:266`. Rust mirrors these in `rewrite-in-rust/rust/crates/v2m-core/src/lyric_sequence.rs:245`, `rewrite-in-rust/rust/crates/v2m-core/src/lyric_sequence.rs:264`, `rewrite-in-rust/rust/crates/v2m-core/src/lyric_sequence.rs:267`, `rewrite-in-rust/rust/crates/v2m-core/src/lyric_sequence.rs:274`, and `rewrite-in-rust/rust/crates/v2m-core/src/lyric_sequence.rs:277`. Existing fixtures at `rewrite-in-rust/fixtures/lyric_sequence_alignment_core.jsonl:13` and `rewrite-in-rust/fixtures/lyric_sequence_alignment_core.jsonl:14` exercise only two direct scan-window cases.
- Required fix: Add focused fixture rows for repeated-token overlap counts, no-candidate scan output, a candidate set large enough to force the 30% branch instead of the 10-minimum branch, and a direct scan case that proves zero-distance early exit if `scan_windows` remains a public helper.

- Severity: low
- Location: rewrite-in-rust/rust/crates/v2m-core/src/lyric_sequence.rs:298
- Issue: Public helper arguments are not guarded against arithmetic overflow before bridge exposure. Normal fixture and internal `find_best_match` calls stay in range, but direct public calls can overflow `best_start + window_size`; `determine_window_size` can also overflow `input_len + extra_window` for adversarial `extra_window` values.
- Evidence: `build_match_from_alignment` computes `window_end` with unchecked addition at `rewrite-in-rust/rust/crates/v2m-core/src/lyric_sequence.rs:298`, and later combines `best_start + win_idx` at `rewrite-in-rust/rust/crates/v2m-core/src/lyric_sequence.rs:314`. `determine_window_size` computes `input_len + extra_window` at `rewrite-in-rust/rust/crates/v2m-core/src/lyric_sequence.rs:602`. Python uses unbounded integers and forgiving slices in `inference/LyricFA/tools/sequence_aligner.py:279` and `inference/LyricFA/tools/sequence_aligner.py:291`.
- Required fix: Before promotion or bridge payload acceptance, either keep these helpers private to validated internal callers or replace these additions with checked/saturating arithmetic and document the public error behavior.

No medium, high, or critical findings were found. The DP matrix/backtracking state, memory-swapped LCS and edit-distance implementations, strict edit-distance update, stable approximate sort, zero-distance break, match/list `Option` shapes, and highlighter text-token indexing match the legacy algorithm for the exercised compatibility surface.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_lyric_sequence_alignment_core.py`: passed
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml lyric_sequence`: passed, 2 lyric sequence tests run
- `cargo clippy --manifest-path rewrite-in-rust/rust/Cargo.toml --all-targets --all-features -- -D warnings`: passed
- `git diff --check`: passed

## Residual Risk

The fixture table is strong for the primary end-to-end alignment, match, helper, and highlighter behavior, but still leaves some scan-window internals proven by source inspection rather than fixture rows. The Rust API also exposes helper parameters that are broader than the current legacy production path; bridge planning still needs payload validation and error-shape decisions before untrusted callers can reach them.

## Promotion Note

This data/algorithm review does not block promotion of the current reimplementation. Promote with follow-up tracking for the scan-window fixture additions and public-helper arithmetic validation before any production bridge or Python-facing API is introduced.
