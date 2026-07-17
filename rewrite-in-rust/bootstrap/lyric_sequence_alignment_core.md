# lyric_sequence_alignment_core Bootstrap

## Boundary

`lyric_sequence_alignment_core` covers the pure token alignment helpers in
`inference/LyricFA/tools/sequence_aligner.py`.

The unit covers:

- `SequenceAligner.compute_alignment` edit-distance DP and `_backtrack` output;
- `SequenceAligner.compute_lcs_length`;
- `SequenceAligner.find_best_match` and its exact-match, window-size,
  scan-window, and alignment-result helpers;
- `SequenceAligner.compute_edit_distance`;
- `SequenceAligner.find_best_match_and_return_lyrics`;
- `calculate_difference_count`;
- `SmartHighlighter.highlight_differences`.

The unit explicitly does not cover `LyricMatcher` file IO, language processor
normalization, Chinese/Japanese G2P, `.lab`/JSON persistence, pipeline summary
printing, HubertFA/GAME/RMVPE/model execution, GUI/Web/CLI routing, or a
production Rust bridge.

## Dependency Expansion

The selected source imports only Python standard-library modules:

- `collections.Counter`;
- `enum.IntEnum`;
- `typing.List`, `typing.Tuple`, and `typing.Optional`.

The only local caller found during discovery is
`inference/LyricFA/tools/lyric_matcher.py`, which constructs one
`SequenceAligner`, reuses it inside `SmartHighlighter`, calls
`find_best_match_and_return_lyrics`, and calls `calculate_difference_count`.

Dependency evidence:

- `pyproject.toml`, `requirements*.txt`, and `uv.lock` include heavy model,
  UI, web, and numeric packages, but none are imported by
  `sequence_aligner.py`.
- `third_party/sources/manifest.json`,
  `third_party/sources/MISSING_SOURCES.md`,
  `third_party/native_sources/manifest.json`, and
  `third_party/source_audit.json` confirm vendored dependency source coverage,
  but this unit does not need a Python package or native/FFI replacement.
- `rewrite-in-rust/resources.md` lists LyricFA helpers as Stage 1 candidates
  while keeping model inference and frontend surfaces legacy-owned.

## Seam

Use the default rewrite seam:

- kind: independent Rust library
- crate: `rewrite-in-rust/rust/crates/v2m-core`
- module: `lyric_sequence`
- runtime owner: legacy Python
- bridge dependencies: none

No PyO3, subprocess bridge, CLI bridge, HTTP service, runtime router, language
processor bridge, or model runtime dependency is introduced.

## Fixture Harness

Python/Rust parity uses:

```text
rewrite-in-rust/fixtures/lyric_sequence_alignment_core.jsonl
```

The legacy Python side is checked by:

```bash
uv run python rewrite-in-rust/bootstrap/check_lyric_sequence_alignment_core.py
```

The fixture table covers:

- empty left/right alignment insert/delete base rows;
- substitute tie retention with default costs;
- delete-over-insert tie with custom costs;
- mixed substitution plus insertion backtracking;
- empty input/reference reason strings;
- exact match with text output;
- no-overlap failure;
- approximate first-tie window match;
- long-input success and failure reason text;
- direct scan-window first-tie retention;
- inclusive overlap threshold behavior;
- duplicate-token `Counter` min-count overlap;
- no-candidate scan output shape;
- 30%-or-10 candidate pruning behavior;
- zero-distance early break behavior;
- direct empty-alignment-result helper output;
- return-lyrics tuple shape;
- LCS, edit-distance, and difference-count helper behavior;
- highlighter empty-match, substitute, delete, insert, and short-text-index
  rendering.

The Rust side should be checked by:

```bash
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml lyric_sequence
```

## Repeated-Call Behavior

The selected helpers are deterministic for fixed token lists, costs, window
parameters, and strings. They do not depend on filesystem state, language
processor dictionaries, model state, process state, global caches, audio
decoder state, GUI/Web state, or network state.

## Rollback

Rollback is keeping production imports unchanged:

```text
inference.LyricFA.tools.sequence_aligner.SequenceAligner
inference.LyricFA.tools.sequence_aligner.SmartHighlighter
inference.LyricFA.tools.sequence_aligner.calculate_difference_count
```

No production caller should import Rust lyric sequence helpers until a
promotion record defines payload validation, dependency wiring, logging text,
and Python-facing error mapping.
