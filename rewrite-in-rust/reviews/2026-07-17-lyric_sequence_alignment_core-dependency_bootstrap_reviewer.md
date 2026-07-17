# lyric_sequence_alignment_core - dependency_bootstrap_reviewer

Date: 2026-07-17
Decision: pass-with-followups

## Findings

- Severity: low
- Location: rewrite-in-rust/dependencies/lyric_sequence_alignment_core.yaml:2
- Issue: The dependency/bootstrap record still reports `status: active` while the manifest has this unit at `status: reimplemented` with a confirmed inventory and complete dependency/bootstrap artifacts. The content of the record is sufficient for this dependency review, but the stale status leaves an ambiguous closure signal for coordinator handoff.
- Evidence: The manifest marks `lyric_sequence_alignment_core` as reimplemented and confirmed (`rewrite-in-rust/manifest.yaml:1151`) and lists the fixture, checker, dependency record, bootstrap record, boundary record, Rust module, and targeted Cargo test as verification evidence (`rewrite-in-rust/manifest.yaml:1165`). The dependency record itself has confirmed inventory impact, a library seam with no bridge dependencies, kept-legacy exclusions, hand-written replacement rationale, and verification commands (`rewrite-in-rust/dependencies/lyric_sequence_alignment_core.yaml:20`, `rewrite-in-rust/dependencies/lyric_sequence_alignment_core.yaml:27`, `rewrite-in-rust/dependencies/lyric_sequence_alignment_core.yaml:30`, `rewrite-in-rust/dependencies/lyric_sequence_alignment_core.yaml:36`, `rewrite-in-rust/dependencies/lyric_sequence_alignment_core.yaml:41`).
- Required fix: Normalize the dependency record status after review, or document that `active` is intentional for dependency records. This reviewer did not update manifest or artifact state.

No boundary, seam, kept-legacy, or hand-written replacement blocker was found.

## Boundary Decision

The manifest unit boundary is confirmed. It should not be split, merged, deferred, or replaced for this dependency/bootstrap gate.

The selected source, `inference/LyricFA/tools/sequence_aligner.py`, imports only `collections.Counter`, `enum.IntEnum`, and typing helpers (`inference/LyricFA/tools/sequence_aligner.py:1`). It implements pure token-list behavior: edit-distance alignment and backtracking, LCS, exact/windowed best-match search, edit distance, wrapper lyric return shape, difference counting, and highlighter rendering (`inference/LyricFA/tools/sequence_aligner.py:21`, `inference/LyricFA/tools/sequence_aligner.py:101`, `inference/LyricFA/tools/sequence_aligner.py:121`, `inference/LyricFA/tools/sequence_aligner.py:196`, `inference/LyricFA/tools/sequence_aligner.py:301`, `inference/LyricFA/tools/sequence_aligner.py:316`, `inference/LyricFA/tools/sequence_aligner.py:323`).

The only Python caller found in this repository is `lyric_matcher.py`, which constructs `SequenceAligner`, shares it with `SmartHighlighter`, calls `find_best_match_and_return_lyrics`, and calls `calculate_difference_count` (`inference/LyricFA/tools/lyric_matcher.py:8`, `inference/LyricFA/tools/lyric_matcher.py:25`, `inference/LyricFA/tools/lyric_matcher.py:59`, `inference/LyricFA/tools/lyric_matcher.py:189`). Keeping the highlighter and difference counter in the same unit is justified because they compose the same aligner behavior and do not add external dependency risk.

## Dependency And Seam Assessment

Kept-legacy capability coverage is accurate for this role. `LyricMatcher` file IO, language processor/G2P behavior, lab/JSON persistence, pipeline printing/control flow, model execution, GUI/Web/CLI routing, and production bridge wiring remain out of scope (`rewrite-in-rust/bootstrap/lyric_sequence_alignment_core.md:19`, `rewrite-in-rust/dependencies/lyric_sequence_alignment_core.yaml:36`). The adjacent caller evidence supports that exclusion: `lyric_matcher.py` owns file reads/writes, `ProcessorFactory`, JSON persistence, folder globbing, missing-lyric/no-match handling, and display output (`inference/LyricFA/tools/lyric_matcher.py:7`, `inference/LyricFA/tools/lyric_matcher.py:28`, `inference/LyricFA/tools/lyric_matcher.py:67`, `inference/LyricFA/tools/lyric_matcher.py:117`, `inference/LyricFA/tools/lyric_matcher.py:177`, `inference/LyricFA/tools/lyric_matcher.py:236`).

The seam is correctly an independent Rust library surface. The bootstrap record specifies `v2m-core::lyric_sequence`, legacy Python as runtime owner, and no bridge dependencies (`rewrite-in-rust/bootstrap/lyric_sequence_alignment_core.md:50`). The Rust module documents that Python remains runtime owner for language processors, G2P dictionaries, file IO, persistence, model execution, GUI/Web/CLI callers, and production routing (`rewrite-in-rust/rust/crates/v2m-core/src/lyric_sequence.rs:1`). The module is exposed as `pub mod lyric_sequence` in `v2m-core` (`rewrite-in-rust/rust/crates/v2m-core/src/lib.rs:17`) and uses only `std::collections::HashMap` in production code (`rewrite-in-rust/rust/crates/v2m-core/src/lyric_sequence.rs:8`).

The hand-written replacement is justified. The dependency and bootstrap records explain that no package-level replacement is needed because this unit has only standard-library Python imports and one local caller (`rewrite-in-rust/dependencies/lyric_sequence_alignment_core.yaml:27`, `rewrite-in-rust/dependencies/lyric_sequence_alignment_core.yaml:30`, `rewrite-in-rust/bootstrap/lyric_sequence_alignment_core.md:24`). `pyproject.toml` and requirements files contain model/UI/web/numeric dependencies but no sequence-alignment dependency used by this source (`pyproject.toml:7`, `requirements.txt:7`, `requirements-web.txt:3`).

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_lyric_sequence_alignment_core.py`: passed.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml lyric_sequence`: passed; 2 lyric sequence tests passed, 0 failed.
- `git diff --check`: passed.
- `cargo tree --manifest-path rewrite-in-rust/rust/Cargo.toml -p v2m-core`: inspected; no PyO3, subprocess, HTTP, ONNX, model-runtime, or async bridge dependency appeared.
- `rg -n "SequenceAligner|SmartHighlighter|calculate_difference_count|find_best_match_and_return_lyrics|sequence_aligner" -g '*.py' ...`: inspected; only `sequence_aligner.py` and `lyric_matcher.py` matched outside the bootstrap checker.
- `rg -n "rapidfuzz|Levenshtein|editdistance|edlib|difflib|sequence|alignment|lyric|pypinyin|g2p|python-mecab|janome|pykakasi" pyproject.toml requirements.txt requirements-linux.txt requirements-web.txt uv.lock`: inspected; no package-level sequence-alignment dependency match.
- Current diff/unit additions inspected with `git status --short`, `git diff --name-only`, targeted `git diff -- rewrite-in-rust/manifest.yaml rewrite-in-rust/rust/crates/v2m-core/src/lib.rs`, and `git ls-files` for the unit artifacts.

## Residual Risk

This review did not perform behavior or data/algorithm review. Fixture parity currently covers empty input/reference, exact match, insert/delete/substitute tie behavior, long input, no overlap, approximate window matching, scan-window tie retention, inclusive overlap threshold, empty alignment result output, wrapper return shape, difference counts, and highlighter cases (`rewrite-in-rust/fixtures/lyric_sequence_alignment_core.jsonl:1`, `rewrite-in-rust/bootstrap/check_lyric_sequence_alignment_core.py:76`, `rewrite-in-rust/rust/crates/v2m-core/src/lyric_sequence.rs:638`, `rewrite-in-rust/rust/crates/v2m-core/src/lyric_sequence.rs:721`). Broader fuzz/property coverage for DP tie behavior and candidate-pruning edge cases remains outside this dependency/bootstrap role.

Production bridge payload validation, logging text, and Python-facing error mapping remain intentionally undefined until a promotion record introduces a runtime route (`rewrite-in-rust/bootstrap/lyric_sequence_alignment_core.md:108`).

## Promotion Note

This dependency/bootstrap role confirms the unit boundary and does not block on dependency, seam, kept-legacy, or hand-written replacement grounds. The coordinator should normalize the dependency record status follow-up and should not mark the unit verified until the required behavior and data/algorithm review roles pass.
