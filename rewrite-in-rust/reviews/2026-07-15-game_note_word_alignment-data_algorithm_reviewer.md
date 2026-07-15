# game_note_word_alignment - data_algorithm_reviewer

Date: 2026-07-15
Decision: pass

## Findings

No findings.

The prior fixture-evidence follow-up is resolved. `fixtures/game_note_word_alignment.tsv:16` now uses an exact binary tie: `word_dur=1.0,1.0`, `note_dur=2.0`, raw internal word boundary `1.0`, note boundaries `0.0,2.0`, and equal distances `1.0,1.0`. The expected output `rest,A` with durations `0.0001,1.9999` proves the first note boundary is selected and then clamped by `_ALIGN_MIN_GAP`.

Rust preserves that behavior because `nearest_boundary_index` updates only on a strictly smaller distance, not on equal distance, matching NumPy `argmin` first-index semantics from `inference/game/alignment_utils.py:123`. The relevant Rust implementation is in `rust/crates/v2m-core/src/game/note_word.rs:131`.

## Checks

- `sed -n '1,240p' /home/fuurin/.claude/skills/vocal2midi-rs-review-gate/SKILL.md`: confirmed this pass is exactly one unit and one role, report-only, with no production code changes.
- `sed -n '1,260p' README.md manifest.yaml resources.md notes.md reviews/README.md`: confirmed control-plane rules, report format, legacy Python runtime owner, and that `game_note_word_alignment` is `reimplemented` with required `data_algorithm_reviewer`.
- `sed -n '1,260p' records/0005-split-game-alignment-unit.md dependencies/game_note_word_alignment.yaml bootstrap/game_note_word_alignment.md`: confirmed the reviewed boundary is limited to `align_notes_to_words`, with NumPy `cumsum`/`abs`/`argmin` represented by a narrow `Vec<f64>` implementation and no bridge dependency.
- `nl -ba inference/game/alignment_utils.py | sed -n '92,195p'`: reviewed Python reference algorithm for boundary cumsum, `np.argmin`, clamp, slicing, rest insertion, `apply_word_uv`, note advancement, merging, and slur flags.
- `nl -ba rewrite-in-rust/rust/crates/v2m-core/src/game/note_word.rs | sed -n '1,240p'`: reviewed Rust implementation and tests. `nearest_boundary_index` uses strict `<` at lines 131-141; the direct tie unit test is at lines 226-229.
- `nl -ba rewrite-in-rust/fixtures/game_note_word_alignment.tsv | sed -n '1,80p'`: confirmed fixture coverage for empty inputs, exact boundaries, multi-note slur reset, snapping, no-snap splitting, repeated-note merge, rest insertion, `apply_word_uv`, monotonic clamp, and the exact binary argmin tie at line 16.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml note_word`: passed. 2 tests passed, 0 failed, 17 filtered out.
- `uv run python rewrite-in-rust/bootstrap/check_game_note_word_alignment.py`: passed with no output.

## Residual Risk

The review is fixture-bound to the documented small-list compatibility surface. It does not prove behavior for malformed inputs such as mismatched `word_dur`/`word_vuv` lengths, non-finite floats, negative durations, or production bridge integration; those are outside this unit's stated boundary and remain legacy-owned until a later promotion/bridge record.

Algorithmic complexity is acceptable for the recorded assumption: boundary snapping is O(word_count * note_count), and note slicing advances monotonically through note intervals. That matches the dependency/bootstrap decision to avoid NumPy/ndarray for this small deterministic helper.

## Promotion Note

This role does not block promotion. Coordinator state-update readiness: ready for this role to be counted as passing promotion evidence; the coordinator still owns any manifest state update and any required remaining review gates.
