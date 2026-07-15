# game_phone_word_parsing Bootstrap

## Boundary

`game_phone_word_parsing` covers these helpers from
`inference/game/alignment_utils.py`:

- `validate_phones`
- `parse_words`
- `merge_consecutive_uv_words`

The public compatibility surface is:

- `validate_phones` success shape: `(True, None)`
- `validate_phones` failure shape and exact error messages
- word duration summing according to `ph_num`
- voiced/unvoiced flag derivation for `uv_vocab=None`, `uv_cond="lead"`, and
  `uv_cond="all"`
- optional merging of consecutive unvoiced words

`align_notes_to_words`, GAME ONNX inference, NumPy array preparation, librosa
note conversion, and production API integration stay legacy-owned.

## Dependency Expansion

`inference/game/alignment_utils.py` imports:

- stdlib: `typing.Literal`
- third party: `numpy`

The selected functions do not call NumPy. They use Python lists, strings,
floats, integer spans, optional sets, and simple aggregation. Therefore this
unit should not add a Rust ndarray/NumPy-equivalent dependency.

## Seam

Use the default rewrite seam:

- kind: independent Rust library
- crate: `rewrite-in-rust/rust/crates/v2m-core`
- runtime owner: legacy Python
- bridge dependencies: none

Do not add PyO3, subprocess, CLI, HTTP, NumPy, ONNX Runtime, or runtime-router
code for this unit.

## Fixture Harness

Rust tests should consume these durable parity tables:

```text
rewrite-in-rust/fixtures/game_phone_word_validation.tsv
rewrite-in-rust/fixtures/game_parse_words.tsv
rewrite-in-rust/fixtures/game_merge_uv.tsv
```

The legacy Python side of the tables is checked by:

```bash
uv run python rewrite-in-rust/bootstrap/check_game_phone_word_parsing.py
```

## Repeated-Call Behavior

The selected helpers are stateless. Repeated calls with the same inputs must
return the same result and must not depend on model, NumPy, GUI, Web, or adapter
state.

## Rollback

Rollback is keeping all production imports unchanged:

```text
inference.game.alignment_utils
```

No production Python caller should import Rust output until a later promotion
record chooses and verifies a bridge.
