# 0056 - Close Lyric Sequence Alignment Gate

Date: 2026-07-17

## Context

`lyric_sequence_alignment_core` was confirmed by record 0055 as a fixture-bound
unit for pure token sequence alignment and highlighter behavior from
`inference/LyricFA/tools/sequence_aligner.py`.

The unit keeps language processors, Chinese/Japanese G2P, LyricMatcher file IO,
`.lab`/JSON persistence, summary printing, model execution, GUI/Web/CLI routing,
and production bridge wiring legacy-owned.

The unit now has:

- dependency/bootstrap review:
  `rewrite-in-rust/reviews/2026-07-17-lyric_sequence_alignment_core-dependency_bootstrap_reviewer.md`
- dependency/bootstrap follow-up review:
  `rewrite-in-rust/reviews/2026-07-17-lyric_sequence_alignment_core-dependency_bootstrap_reviewer-rerun.md`
- behavior review:
  `rewrite-in-rust/reviews/2026-07-17-lyric_sequence_alignment_core-behavior_reviewer.md`
- data/algorithm review:
  `rewrite-in-rust/reviews/2026-07-17-lyric_sequence_alignment_core-data_algorithm_reviewer.md`
- data/algorithm follow-up review:
  `rewrite-in-rust/reviews/2026-07-17-lyric_sequence_alignment_core-data_algorithm_reviewer-rerun.md`

The initial dependency and data/algorithm reviews returned
`pass-with-followups`. The coordinator fixed the stale dependency-record status,
added stronger scan-window fixtures, and hardened public helper arithmetic with
saturating addition. Both reruns returned `pass`.

## Decision

Accept `lyric_sequence_alignment_core` as verified.

The verified Rust unit preserves:

- edit-distance DP output and backtracking shape;
- substitute/delete/insert tie behavior;
- LCS helper behavior;
- exact-match shortcut;
- overlap threshold filtering, including equality at the threshold;
- stable candidate sorting, first-tie retention, 30%-or-10 pruning, and
  zero-distance break behavior;
- long-input and no-window reason strings;
- empty-alignment-result helper output;
- wrapper return-lyrics tuple shape;
- difference-count helper behavior;
- `SmartHighlighter` parenthesized output and text-token indexing.

## Verification

```bash
uv run python rewrite-in-rust/bootstrap/check_lyric_sequence_alignment_core.py
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml lyric_sequence
```

Broader checks also passed during coordinator review:

```bash
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml
cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check
cargo clippy --manifest-path rewrite-in-rust/rust/Cargo.toml --all-targets --all-features -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --manifest-path rewrite-in-rust/rust/Cargo.toml --no-deps
git diff --check
```

## Residual Risk

The verified proof is fixture-bound and does not promote Rust into production.
Runtime promotion must still define external payload validation, dependency
wiring, logging text, and Python-facing error mapping. Language processor and
G2P behavior remain adjacent legacy-owned units.

## Reversal

Rollback remains keeping `SequenceAligner`, `SmartHighlighter`, and
`calculate_difference_count` in `inference.LyricFA.tools.sequence_aligner` as
the runtime owners. No production bridge was introduced.
