# lyric_language_processor_contract Bootstrap

## Boundary

`lyric_language_processor_contract` covers the deterministic language processor
selection and clean/split/phonetic flow in
`inference/LyricFA/tools/language_processors.py`.

The unit covers:

- `ProcessorFactory.create_processor` case-insensitive selection;
- `ProcessorFactory.get_supported_languages` ordering;
- exact unsupported-language `ValueError` text;
- `LyricData` text/phonetic/raw-text output shape;
- base `LanguageProcessor.clean_text` regex filtering and Python `re`\s
  whitespace compaction, including the Chinese bracket-adjacent regex artifact;
- `ChineseProcessor` legacy bracket-regex cleaning, split, and phonetic flow
  over injected Mandarin maps;
- `EnglishProcessor` lowercase split and identity phonetic behavior;
- `JapaneseProcessor` whitespace-only cleaning;
- Japanese kana display-token split;
- Japanese romaji phonetic conversion;
- Japanese `build_reference_lyric` text/phonetic tuple behavior;
- empty-input behavior.

The unit explicitly does not cover:

- Chinese dictionary file IO or packaging;
- Chinese dictionary algorithm internals already verified by
  `zh_g2p_dictionary_core`;
- Japanese fallback tokenizer/table internals already verified by
  `ja_g2p_fallback_core`;
- `pyopenjtalk.run_frontend` output parity or OpenJTalk runtime ownership;
- `LyricMatcher` file/state/JSON behavior already covered by
  `lyric_matching_file_contract_core`;
- `SequenceAligner` internals;
- model execution;
- GUI/Web/CLI routing;
- production Rust routing.

## Dependency Expansion

`language_processors.py` imports only Python standard-library modules plus local
helpers:

- `re`;
- `abc.ABC` and `abstractmethod`;
- `dataclasses.dataclass`;
- `typing.List`, `typing.Dict`, and `typing.Type`;
- `ZhG2p` and `split_string` from `ZhG2p.py`;
- `JaG2p` from `JaG2p.py`.

Dependency evidence:

- `pyproject.toml` and `requirements.txt` include `pyopenjtalk` only under a
  Windows platform marker.
- `uv.lock` contains `pyopenjtalk`, but the current Linux boundary is the
  already verified pyopenjtalk-absent fallback seam.
- `third_party/source_audit.json` reports zero source-audit errors and zero
  third-party binary artifacts.
- `zh_g2p_dictionary_core` verifies deterministic Chinese G2P behavior over
  injected maps.
- `ja_g2p_fallback_core` verifies deterministic Japanese fallback behavior with
  `pyopenjtalk = None`.
- `lyric_matching_file_contract_core` keeps file/state/JSON orchestration out
  of this unit.

The selected Rust seam should therefore compose narrow processor logic with the
existing Rust G2P helper modules. It should not add PyO3, subprocess, HTTP,
CLI, runtime-router, OpenJTalk, Flask, PyQt, ONNX Runtime, or model inference
dependencies.

## Seam

Use the default rewrite seam:

- kind: independent Rust library
- crate: `rewrite-in-rust/rust/crates/v2m-core`
- expected module: `lyric_language`
- runtime owner: legacy Python
- bridge dependencies: none

No production caller imports Rust language processor helpers in this unit.

## Fixture Harness

Python/Rust parity uses:

```text
rewrite-in-rust/fixtures/lyric_language_processor_contract.jsonl
```

The legacy Python side is checked by:

```bash
uv run python rewrite-in-rust/bootstrap/check_lyric_language_processor_contract.py
```

The checker forces:

```python
inference.LyricFA.tools.JaG2p.pyopenjtalk = None
```

For Chinese cases, the checker injects minimal Mandarin dictionary maps into a
`ZhG2p` instance so the fixture proves processor orchestration without relying
on bundled dictionary asset loading.

The fixture table covers:

- supported-language order and case-insensitive factory selection;
- unsupported-language exception type and message;
- English cleaning, lowercase splitting, apostrophe/hyphen retention, digit
  splitting, Python `re`\s whitespace, and identity phonetics;
- Chinese legacy bracket-regex cleaning, splitting, injected-map phonetic
  conversion, Python `re`\s whitespace, and the legacy empty-input phonetic
  shape `[""]`;
- `LyricData` text/phonetic/raw-text output shape;
- Japanese whitespace normalization, kana display-token splitting, romaji
  phonetic conversion, Python `re`\s whitespace, `build_reference_lyric`,
  number handling in fallback mode, and empty input.

The Rust side should be checked after writer implementation by:

```bash
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml lyric_language
```

## Repeated-Call Behavior

The selected processor behavior is deterministic for fixed input text,
language code, injected Mandarin maps, and the pyopenjtalk-absent Japanese
fallback. It does not depend on model state, network state, GUI/Web state, or
global runtime caches.

## Rollback

Rollback is keeping production imports unchanged:

```text
inference.LyricFA.tools.language_processors.ProcessorFactory
inference.LyricFA.tools.language_processors.LanguageProcessor
inference.LyricFA.tools.language_processors.ChineseProcessor
inference.LyricFA.tools.language_processors.EnglishProcessor
inference.LyricFA.tools.language_processors.JapaneseProcessor
inference.LyricFA.tools.language_processors.LyricData
```

No production caller should import Rust language processor helpers until a
promotion record defines dictionary asset payloads, OpenJTalk ownership,
Python-facing error mapping, logging text, and rollback.
