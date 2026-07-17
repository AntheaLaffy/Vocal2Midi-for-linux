# 0055 - Confirm Lyric Sequence Alignment Boundary

Date: 2026-07-17

## Context

The next manifest unit after the verified slicer batch was
`lyric_sequence_alignment_core`. It was still `planned` and `provisional`, so
dependency/bootstrap discovery was required before writer work.

The selected source is `inference/LyricFA/tools/sequence_aligner.py`. The only
local caller found during discovery is `inference/LyricFA/tools/lyric_matcher.py`.

## Discovery

`sequence_aligner.py` imports only Python standard-library modules:

- `collections.Counter`;
- `enum.IntEnum`;
- `typing.List`, `typing.Tuple`, and `typing.Optional`.

The file implements pure token-list behavior:

- edit-distance DP and backtracking;
- LCS prefiltering;
- exact-match and approximate window matching;
- overlap threshold filtering, including inclusive equality at the threshold,
  and candidate pruning;
- match-result construction and reason strings;
- difference counting;
- parenthesized highlighter rendering.

The fixture gate also covers duplicate-token overlap counting, no-candidate
scan output, the 30%-or-10 candidate pruning branch, and zero-distance early
break behavior. Rust public helper arithmetic uses saturating addition for
window bounds so future bridge planning starts from Python-like forgiving slice
behavior rather than wrapping `usize` arithmetic.

`SmartHighlighter` reuses the same `SequenceAligner.compute_alignment` behavior
as the matcher path. Splitting it out would duplicate fixtures around alignment
output instead of reducing dependency or runtime risk.

## Decision

Confirm the existing `lyric_sequence_alignment_core` unit boundary.

Use a narrow hand-written Rust implementation in `v2m-core::lyric_sequence`
against shared fixtures. Do not add a package-level alignment dependency, PyO3
bridge, subprocess bridge, HTTP bridge, language-processor bridge, model
runtime dependency, or production runtime routing.

Keep these behaviors legacy-owned:

- `LyricMatcher` file IO and `.lab`/JSON persistence;
- Chinese/Japanese G2P and language processor normalization;
- HubertFA, GAME, RMVPE, ASR, and other model execution;
- GUI/Web/CLI routing.

## Verification

Dependency/bootstrap artifacts:

```text
rewrite-in-rust/dependencies/lyric_sequence_alignment_core.yaml
rewrite-in-rust/bootstrap/lyric_sequence_alignment_core.md
rewrite-in-rust/fixtures/lyric_sequence_alignment_core.jsonl
rewrite-in-rust/bootstrap/check_lyric_sequence_alignment_core.py
```

Current Python parity command:

```bash
uv run python rewrite-in-rust/bootstrap/check_lyric_sequence_alignment_core.py
```

Expected Rust command after writer implementation:

```bash
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml lyric_sequence
```

## Reversal

Rollback remains keeping `SequenceAligner`, `SmartHighlighter`, and
`calculate_difference_count` in `inference.LyricFA.tools.sequence_aligner` as
the runtime owners. No production bridge was introduced.
