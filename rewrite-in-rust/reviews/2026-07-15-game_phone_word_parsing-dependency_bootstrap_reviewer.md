# game_phone_word_parsing - dependency_bootstrap_reviewer

Date: 2026-07-15
Decision: pass

## Findings

No findings after checking full role scope.

The manifest unit boundary is confirmed. `game_phone_word_parsing` should remain
split from `game_note_word_alignment`; it should not be merged, deferred, or
replaced. The prior missing true empty `parse_words([], [], [])` fixture is now
present at `rewrite-in-rust/fixtures/game_parse_words.tsv:7`, distinct from the
zero-length word-span case at `rewrite-in-rust/fixtures/game_parse_words.tsv:6`.

## Evidence

- `rewrite-in-rust/records/0005-split-game-alignment-unit.md:13` records the
  dependency split between pure phone/word list helpers and NumPy-style note
  timeline alignment.
- `rewrite-in-rust/dependencies/game_phone_word_parsing.yaml:8` keeps the seam
  as a library unit with no bridge dependencies, and
  `rewrite-in-rust/dependencies/game_phone_word_parsing.yaml:25` selects a
  narrow hand-written replacement against `inference/game/alignment_utils.py`.
- `rewrite-in-rust/bootstrap/game_phone_word_parsing.md:21` keeps
  `align_notes_to_words`, GAME ONNX inference, NumPy array preparation, librosa
  note conversion, and production API integration legacy-owned.
- `rewrite-in-rust/bootstrap/game_phone_word_parsing.md:31` documents that the
  selected helpers do not call NumPy and should not add an ndarray/NumPy
  equivalent dependency.
- `rewrite-in-rust/rust/Cargo.toml` and
  `rewrite-in-rust/rust/crates/v2m-core/Cargo.toml` contain no PyO3, NumPy,
  ndarray, subprocess, HTTP, or runtime-router dependency.
- `rewrite-in-rust/rust/crates/v2m-core/src/lib.rs:3` states the Rust crate is
  not wired into the Python runtime, and the scan for bridge/import changes found
  no production Python caller importing the Rust phone/word implementation.

## Checks

- `uv run python scripts/audit_vendored_sources.py`: passed; source audit
  reported 135 Python packages, 41 native-extension packages, 269 covered foreign
  runtime native binaries, 0 third-party binary artifacts, and no errors.
- `uv run python rewrite-in-rust/bootstrap/check_game_phone_word_parsing.py`:
  passed with no output.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml phone_word`:
  passed; 4 tests passed, 0 failed, 13 filtered out.
- `rg -n 'name = "numpy"|numpy' uv.lock pyproject.toml requirements*.txt`:
  confirmed project NumPy remains locked at `numpy-1.26.4` in `uv.lock`, while
  this unit's dependency record and bootstrap keep NumPy out of the Rust seam.
- `rg -n 'phone_word|v2m_core|pyo3|subprocess|rewrite-in-rust|parse_words|validate_phones|merge_consecutive_uv_words' ...`:
  inspected production and rewrite paths, excluding review and target output;
  no production Rust bridge/import changes were found.
- `git status --short` and targeted `git diff --stat` over production Python
  paths, Rust Cargo manifests, `pyproject.toml`, and `uv.lock`: found no
  production bridge/import or dependency-manifest changes for this unit.

## Residual Risk

This role does not review full Python/Rust behavior parity beyond whether the
dependency/bootstrap evidence supports the chosen capability boundary. Behavior
edge cases such as malformed spans remain behavior-review or promotion-review
concerns, not dependency-bootstrap blockers.

## Promotion Note

This dependency-bootstrap role does not block coordinator state update. The
boundary is confirmed, the split remains justified, no new bridge or heavy
dependency is required, legacy ownership is intact, and the prior fixture
follow-up has been resolved and verified.
