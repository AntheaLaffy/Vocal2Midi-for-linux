# game_phone_word_parsing - behavior_reviewer

Date: 2026-07-15
Decision: pass

## Findings

No findings after checking full role scope.

## Checks

- `sed -n '1,240p' /home/fuurin/.claude/skills/vocal2midi-rs-review-gate/SKILL.md`: read the review-gate process before reviewing.
- `sed -n '1,220p' rewrite-in-rust/README.md`, `sed -n '1,260p' rewrite-in-rust/manifest.yaml`, `sed -n '1,220p' rewrite-in-rust/resources.md`, `sed -n '1,220p' rewrite-in-rust/notes.md`, `sed -n '1,220p' rewrite-in-rust/reviews/README.md`: read required control-plane context.
- `sed -n '1,220p' rewrite-in-rust/records/0001-initialize-rust-rewrite.md` through `records/0005-split-game-alignment-unit.md`: confirmed legacy runtime ownership, provisional inventory policy, split unit boundary, and rollback discipline.
- `sed -n '1,260p' rewrite-in-rust/dependencies/game_phone_word_parsing.yaml` and `sed -n '1,280p' rewrite-in-rust/bootstrap/game_phone_word_parsing.md`: confirmed behavior scope covers `validate_phones`, `parse_words`, and `merge_consecutive_uv_words`; `align_notes_to_words` and runtime integration remain out of scope.
- `sed -n '1,320p' inference/game/alignment_utils.py`: confirmed the Python compatibility source. `validate_phones` returns `(bool, str | None)` at lines 9-25; `parse_words` derives duration/vuv output at lines 28-66; `merge_consecutive_uv_words` merges consecutive unvoiced words at lines 69-89.
- `sed -n '1,420p' rewrite-in-rust/rust/crates/v2m-core/src/game/phone_word.rs`: confirmed Rust parity implementation. `validate_phones_py_shape` returns `(bool, Option<String>)` at lines 86-95, resolving the prior return-shape follow-up; fixture tests assert that shape at lines 272-282.
- `nl -ba rewrite-in-rust/fixtures/game_parse_words.tsv | sed -n '1,120p' | cat -vet`: confirmed fixture line 7 is the true `parse_words([], [], [])` row with empty duration and vuv outputs, distinct from the zero-span row on line 6.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml phone_word`: passed; 4 tests passed, 0 failed, 13 filtered out.
- `uv run python rewrite-in-rust/bootstrap/check_game_phone_word_parsing.py`: passed.
- `rg -n "v2m_core|phone_word|validate_phones\\(|validate_phones_py_shape|parse_words\\(|merge_consecutive_uv_words\\(" -S --glob '!rewrite-in-rust/rust/target/**' --glob '!rewrite-in-rust/reviews/*.md'`: inspected; no production Python/Rust bridge or runtime-owner route to this Rust unit was found. Hits were limited to the legacy Python implementation, rewrite docs, fixtures/checkers, and independent Rust code.

## Residual Risk

The reviewed fixture surface covers valid and invalid phone spans, exact validation messages including Python list formatting, `validate_phones` Python-shaped return values, `uv_vocab=None`, `uv_cond=lead`, `uv_cond=all`, zero-span empty behavior, true no-span empty behavior, and leading/middle/trailing unvoiced merge runs.

Remaining risk is outside the documented accepted fixture inputs: malformed spans such as zero-length words with non-`None` `uv_vocab`, negative Python `ph_num` values that Rust `usize` cannot represent, and mismatched `word_dur`/`word_vuv` lengths in `merge_consecutive_uv_words`. These are not blockers for this role because the unit's current public policy is fixture-bound and Python remains the runtime owner.

## Promotion Note

This behavior review does not block coordinator state update. The prior behavior follow-up is resolved by `validate_phones_py_shape`, the prior empty parse fixture gap is resolved by the all-empty fixture row, the required checks pass, and rollback remains keeping `inference.game.alignment_utils` as runtime owner.
