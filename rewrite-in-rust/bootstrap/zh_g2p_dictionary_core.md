# zh_g2p_dictionary_core Bootstrap

## Boundary

`zh_g2p_dictionary_core` covers the deterministic dictionary-conversion helpers
in `inference/LyricFA/tools/ZhG2p.py`.

The unit covers:

- `tone_to_normal`;
- module-level `split_string`;
- `ZhG2p.split_string_no_regex`;
- `ZhG2p.convert`;
- `ZhG2p.convert_list`;
- dictionary lookup through `word_dict`, `phrases_dict`, `phrases_map`, and
  `trans_dict`;
- ASCII digit conversion through `number_map`;
- phrase-window matching and replacement behavior;
- include-tone and tone-stripping behavior;
- unknown, Latin, kana, and punctuation passthrough through `reset_zh`.

The unit explicitly does not cover `LanguageProcessor`, `LyricMatcher`,
`inference/API/lfa_api.py` orchestration, `.lab`/JSON persistence, Japanese G2P,
model execution, GUI/Web/CLI routing, production Rust routing, or bundled
dictionary packaging for runtime promotion.

## Dependency Expansion

`ZhG2p.py` imports only `os`; the conversion logic itself is pure Python over
in-memory dictionaries and strings. The source file loads dictionary text files
from `inference/LyricFA/Dicts/mandarin` or
`inference/LyricFA/Dicts/cantonese`, but the selected Rust seam accepts already
loaded dictionary maps. This keeps the current production file loading and
packaging behavior legacy-owned.

Dependency evidence:

- `pyproject.toml`, `requirements*.txt`, and `uv.lock` include model, UI, Web,
  numeric, and optional Japanese `pyopenjtalk` dependencies, but no package is
  imported by `ZhG2p.py` for Chinese G2P conversion.
- `third_party/sources/manifest.json`,
  `third_party/sources/MISSING_SOURCES.md`,
  `third_party/native_sources/manifest.json`, and
  `third_party/source_audit.json` confirm vendored dependency source coverage,
  but this unit does not need a package or native/FFI replacement.
- `inference/LyricFA/tools/language_processors.py` uses Mandarin `ZhG2p` for
  Chinese lyric processing and reuses module-level `split_string` for English
  token splitting.
- `inference/API/lfa_api.py` calls Mandarin `ZhG2p` for pinyin display and lab
  fallback paths.

## Seam

Use the default rewrite seam:

- kind: independent Rust library
- crate: `rewrite-in-rust/rust/crates/v2m-core`
- module: `zh_g2p`
- runtime owner: legacy Python
- bridge dependencies: none

No PyO3, subprocess bridge, CLI bridge, HTTP service, runtime router, language
processor bridge, full-dictionary packaging, or model runtime dependency is
introduced.

## Fixture Harness

Python/Rust parity uses:

```text
rewrite-in-rust/fixtures/zh_g2p_dictionary_core.jsonl
```

The legacy Python side is checked by:

```bash
uv run python rewrite-in-rust/bootstrap/check_zh_g2p_dictionary_core.py
```

The fixture table injects small dictionaries and covers:

- tone mark, v/umlaut, and uppercase passthrough behavior;
- module split behavior with ASCII letters, apostrophes, hyphen, ASCII digit,
  hanzi, kana, small kana, and skipped punctuation;
- `split_string_no_regex` exclusion of apostrophe/hyphen from letter runs;
- hanzi conversion with unknown, Latin, kana, and punctuation passthrough;
- include-tone false/true behavior;
- traditional-to-simplified lookup;
- ASCII digit conversion and non-conversion;
- polyphonic fallback to the default word reading;
- direct length-4 phrase precedence;
- previous-current phrase replacement;
- previous-current-with-next phrase replacement;
- previous-current-next phrase replacement;
- `convert(text)` using no-regex splitting before conversion.

The Rust side should be checked by:

```bash
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml zh_g2p
```

## Repeated-Call Behavior

The selected helpers are deterministic for fixed input tokens, options, and
dictionary maps. They do not depend on model state, audio state, process state,
global caches, GUI/Web state, network state, or filesystem state once the maps
are supplied.

## Rollback

Rollback is keeping production imports unchanged:

```text
inference.LyricFA.tools.ZhG2p.ZhG2p
inference.LyricFA.tools.ZhG2p.split_string
inference.LyricFA.tools.ZhG2p.tone_to_normal
```

No production caller should import Rust Chinese G2P helpers until a promotion
record defines dictionary packaging, payload validation, logging text, and
Python-facing error mapping.
