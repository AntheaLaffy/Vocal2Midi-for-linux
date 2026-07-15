# game_note_word_alignment - dependency_bootstrap_reviewer

Date: 2026-07-15
Decision: pass

## Findings

No findings for this dependency bootstrap review.

Dependency coverage, kept-legacy decisions, seam choice, hand-written replacement choice, fixture inventory, and production bridge/import boundaries are adequately documented and verified for this role.

## Evidence

- Boundary confirmed: `rewrite-in-rust/manifest.yaml:112` lists `game_note_word_alignment` as `reimplemented` with `inventory_status: confirmed`, current owner `legacy`, and rollback to `inference.game.alignment_utils.align_notes_to_words` at `rewrite-in-rust/manifest.yaml:128`.
- Re-cut decision is documented: `rewrite-in-rust/records/0005-split-game-alignment-unit.md:26` splits the provisional GAME alignment unit into `game_phone_word_parsing` and `game_note_word_alignment`; `rewrite-in-rust/records/0005-split-game-alignment-unit.md:31` rejects a Rust NumPy/ndarray dependency for this unit and chooses a narrow `Vec<f64>` implementation after fixtures capture tolerance and tie behavior.
- Capability scope is narrow: `rewrite-in-rust/dependencies/game_note_word_alignment.yaml:4` maps only `align_notes_to_words`; `rewrite-in-rust/dependencies/game_note_word_alignment.yaml:33` keeps phone parsing in `game_phone_word_parsing`; `rewrite-in-rust/dependencies/game_note_word_alignment.yaml:35` keeps GAME ONNX inference, NumPy array preparation, librosa note conversion, HFA word extraction, and production API integration legacy-owned.
- Adjacent phone parsing remains separate: `rewrite-in-rust/dependencies/game_phone_word_parsing.yaml:4` covers `validate_phones`, `parse_words`, and `merge_consecutive_uv_words`, with no bridge dependencies at `rewrite-in-rust/dependencies/game_phone_word_parsing.yaml:8`.
- Seam is appropriate: `rewrite-in-rust/bootstrap/game_note_word_alignment.md:46` chooses an independent Rust library seam, legacy runtime owner, and no bridge dependencies; `rewrite-in-rust/bootstrap/game_note_word_alignment.md:55` explicitly forbids PyO3, subprocess, CLI, HTTP, NumPy, ndarray, ONNX Runtime, and runtime-router code for this unit.
- Python source behavior uses only small NumPy operations for this unit: `inference/game/alignment_utils.py:113` and `inference/game/alignment_utils.py:123` use `np.cumsum`, `np.abs`, and `np.argmin`; `pyproject.toml:17` declares `numpy<2.0.0`, `uv.lock:926` pins `numpy` to `1.26.4`, and `third_party/sources/manifest.json:475` vendors `third_party/sources/numpy-1.26.4`.
- Source audit coverage is clean: `third_party/source_audit.json:2` reports 135 installed Python packages and `third_party/source_audit.json:7` reports all 269 foreign runtime native binaries covered, with zero third-party binary artifacts at `third_party/source_audit.json:8` and no errors at `third_party/source_audit.json:24`.
- Fixture coverage matches the dependency record: `rewrite-in-rust/fixtures/game_note_word_alignment.tsv:2` through `rewrite-in-rust/fixtures/game_note_word_alignment.tsv:16` cover empty inputs, exact boundaries, slur reset, snapping, no-snap behavior, repeated note merge, rest insertion, apply-word-UV behavior, monotonic clamp, and `argmin` tie behavior.
- Rust implementation follows the hand-written replacement decision: `rewrite-in-rust/rust/crates/v2m-core/src/game/note_word.rs:21` uses local cumulative boundary vectors, `rewrite-in-rust/rust/crates/v2m-core/src/game/note_word.rs:30` scans nearest boundaries, and `rewrite-in-rust/rust/crates/v2m-core/src/game/note_word.rs:120` implements `cumsum_with_zero` directly. Cargo manifests add no third-party crate dependencies.
- Production bridge/import scan found no production changes: `git diff -- inference application gui web_server.py web_task_manager.py` and `git ls-files -o --exclude-standard inference application gui web_server.py web_task_manager.py` produced no output. The live caller still imports the Python function at `inference/API/game_api.py:14` and calls it at `inference/API/game_api.py:289`.
- Rust dependency/bridge scan found no forbidden Rust bridge/dependency additions: `rg -n "pyo3|maturin|ndarray|numpy|onnxruntime|Command::new|std::process|subprocess|http|router" rewrite-in-rust/rust/Cargo.toml rewrite-in-rust/rust/crates/v2m-core/Cargo.toml rewrite-in-rust/rust/crates/v2m-core/src` only matched the local test name `nearest_boundary_index_uses_first_minimum_like_numpy_argmin` at `rewrite-in-rust/rust/crates/v2m-core/src/game/note_word.rs:227`.

## Checks

- `uv run python scripts/audit_vendored_sources.py`: passed. Output: `Source audit passed: 135 Python packages, 41 native-extension packages, 269 foreign runtime native binaries, 0 third_party binary artifacts.`
- `uv run python rewrite-in-rust/bootstrap/check_game_note_word_alignment.py`: passed with no output.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml note_word`: passed. 2 tests passed, 0 failed, 17 filtered out.
- `git diff -- inference application gui web_server.py web_task_manager.py`: passed. No production Python diff output.
- `git ls-files -o --exclude-standard inference application gui web_server.py web_task_manager.py`: passed. No untracked production Python bridge/import files.
- `rg -n "pyo3|maturin|ndarray|numpy|onnxruntime|Command::new|std::process|subprocess|http|router" rewrite-in-rust/rust/Cargo.toml rewrite-in-rust/rust/crates/v2m-core/Cargo.toml rewrite-in-rust/rust/crates/v2m-core/src`: passed. No forbidden dependency or bridge matches; only the local NumPy-argmin test name matched.

## Residual Risk

This review does not prove full Python/Rust behavior parity or numeric edge-case sufficiency beyond dependency/bootstrap scope. The unit still needs the requested behavior/data-algorithm review coverage before promotion because `align_notes_to_words` owns timeline snapping, clamping, and note-slicing semantics.

## Boundary Decision

The manifest unit boundary remains confirmed. It should not be split, merged, deferred, or replaced for dependency/bootstrap reasons.

## Promotion Note

This role does not block coordinator state update. Coordinator can record dependency bootstrap review as passed, while leaving behavior/data-algorithm promotion gates to their separate review roles.
