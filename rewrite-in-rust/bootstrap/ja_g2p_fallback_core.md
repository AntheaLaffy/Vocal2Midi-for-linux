# ja_g2p_fallback_core Bootstrap

## Boundary

`ja_g2p_fallback_core` covers the deterministic Japanese G2P fallback helpers
in `inference/LyricFA/tools/JaG2p.py` when `pyopenjtalk` is absent or returns no
usable frontend analysis.

The unit covers:

- character classification helpers used by the tokenizer;
- `JaG2p._normalize_text`;
- `JaG2p.split_input_string_no_regex`;
- `JaG2p._split_japanese_segment`;
- fallback entry behavior for Latin, Japanese, numeric, unknown, and empty
  tokens;
- hiragana/katakana conversion tables;
- `KATA_TO_ROMAJI` and long-vowel mora expansion;
- ASCII, full-width, and `〇` number mapping;
- unsupported Unicode digit fallback when a digit is not in `number_map`;
- `JaG2p.convert`;
- `JaG2p.convert_list`;
- `JaG2p.split_string_no_regex`;
- `JaG2p.split_kana_no_regex`.

The unit explicitly does not cover `pyopenjtalk.run_frontend` output parity,
OpenJTalk dictionary/model ownership, language processor orchestration,
`LyricMatcher`, `inference/API/lfa_api.py` orchestration, `.lab`/JSON
persistence, ASR phoneme post-processing, model execution, GUI/Web/CLI routing,
or production Rust routing.

## Dependency Expansion

`JaG2p.py` imports `re` and optional `pyopenjtalk`:

```python
try:
    import pyopenjtalk
except ImportError:
    pyopenjtalk = None
```

Dependency evidence:

- `pyproject.toml` and `requirements.txt` include `pyopenjtalk` only under a
  Windows platform marker.
- `uv.lock` contains a `pyopenjtalk` sdist entry, but the Linux uv environment
  does not install it for the current platform marker.
- `third_party/source_audit.json` shows the installed package source audit has
  no errors and zero third-party binary artifacts.
- `inference/LyricFA/tools/language_processors.py` uses `JaG2p` for Japanese
  lyric splitting and phonetic list creation.
- `inference/API/lfa_api.py` uses `JaG2p` for Japanese display token generation
  and keeps language/API orchestration in Python.

The Rust unit therefore uses a hand-written fallback implementation over the
local tables in `JaG2p.py`. It does not add an OpenJTalk, PyO3, subprocess,
HTTP, CLI, or runtime-router dependency.

## Seam

Use the default rewrite seam:

- kind: independent Rust library
- crate: `rewrite-in-rust/rust/crates/v2m-core`
- module: `ja_g2p`
- runtime owner: legacy Python
- bridge dependencies: none

No production caller imports Rust Japanese G2P helpers in this unit.

## Fixture Harness

Python/Rust parity uses:

```text
rewrite-in-rust/fixtures/ja_g2p_fallback_core.jsonl
```

The legacy Python side is checked by:

```bash
uv run python rewrite-in-rust/bootstrap/check_ja_g2p_fallback_core.py
```

The checker forces:

```python
inference.LyricFA.tools.JaG2p.pyopenjtalk = None
```

The fixture table covers:

- helper classification for ASCII letters, special letters, Python
  `str.isdigit()` digits, `〇`, kanji, kana, small kana, long mark, Japanese
  symbols, and skipped punctuation;
- whitespace normalization through Python regex `\s+`;
- input splitting for mixed Latin, numeric-like, Japanese, and punctuation
  text;
- numeric-like splitting for Unicode digits that are not in `number_map`;
- Japanese segment splitting for contracted kana and long vowels;
- kana-to-romaji and kana-mora tables, including long vowels, `cl`, `n`, and
  uncommon `ヵ`/`ヶ` fallback behavior;
- `convert` and `convert_list` with `convert_number` true and false;
- public romaji and kana split outputs.

The Rust side should be checked by:

```bash
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml ja_g2p
```

## Repeated-Call Behavior

The selected fallback helpers are deterministic for fixed text and options.
They do not depend on model state, process state, network state, filesystem
state, GUI/Web state, or global caches once `pyopenjtalk` is excluded.

## Rollback

Rollback is keeping production imports unchanged:

```text
inference.LyricFA.tools.JaG2p.JaG2p
inference.LyricFA.tools.JaG2p.KATA_TO_ROMAJI
```

No production caller should import Rust Japanese G2P helpers until a promotion
record defines OpenJTalk ownership, fallback ordering, payload validation,
logging text, and Python-facing error mapping.
