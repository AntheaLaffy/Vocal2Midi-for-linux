# game_note_word_alignment Bootstrap

## Boundary

`game_note_word_alignment` covers only
`inference/game/alignment_utils.py::align_notes_to_words`.

The public compatibility surface is:

- empty input behavior
- cumulative word and note boundaries
- snapping internal word boundaries to nearby note boundaries within `tol`
- NumPy `argmin` first-index tie behavior
- `_ALIGN_MIN_GAP` monotonic clamping and segment filtering
- slicing notes into word spans
- merging repeated adjacent note names within one word
- inserting rests for word spans with no note overlap
- `apply_word_uv=True` rest conversion for unvoiced words
- `[0, 1, ...]` slur flags for multi-note words

`validate_phones`, `parse_words`, `merge_consecutive_uv_words`, GAME ONNX
inference, NumPy array preparation, librosa note conversion, and production API
integration stay legacy-owned or covered by separate units.

## Dependency Expansion

`align_notes_to_words` uses NumPy for:

- `np.cumsum`
- `np.abs`
- `np.argmin`

The selected behavior is deterministic over small float lists. The fixture
surface captures the relevant NumPy behavior, including first-index `argmin`
ties and floating-point outputs. The Rust implementation should use `Vec<f64>`
and explicit scanning rather than adding ndarray or a NumPy-equivalent crate.

Dependency evidence:

- `pyproject.toml` declares `numpy<2.0.0`.
- `uv.lock` pins `numpy` to `1.26.4`.
- `third_party/sources/manifest.json` vendors `third_party/sources/numpy-1.26.4`.
- `third_party/source_audit.json` reports all foreign runtime native binaries
  covered and zero `third_party` binary artifacts.

## Seam

Use the default rewrite seam:

- kind: independent Rust library
- crate: `rewrite-in-rust/rust/crates/v2m-core`
- runtime owner: legacy Python
- bridge dependencies: none

Do not add PyO3, subprocess, CLI, HTTP, NumPy, ndarray, ONNX Runtime, or
runtime-router code for this unit.

## Fixture Harness

Rust tests should consume the durable parity table at:

```text
rewrite-in-rust/fixtures/game_note_word_alignment.tsv
```

The legacy Python side of the table is checked by:

```bash
uv run python rewrite-in-rust/bootstrap/check_game_note_word_alignment.py
```

## Repeated-Call Behavior

The selected helper is stateless for a fixed set of input lists, `tol`, and
`apply_word_uv`. Repeated calls with the same inputs must return the same result
and must not depend on model, ONNX Runtime, GUI, Web, or adapter state.

## Rollback

Rollback is keeping production imports unchanged:

```text
inference.game.alignment_utils.align_notes_to_words
```

No production Python caller should import Rust output until a later promotion
record chooses and verifies a bridge.
