# 0063 - Confirm Lyric Language Processor Boundary

Date: 2026-07-17

## Context

The next manifest unit after `lyric_matching_file_contract_core` is
`lyric_language_processor_contract`. It was still `planned` and `provisional`,
so dependency/bootstrap discovery was required before writer work.

The selected source is `inference/LyricFA/tools/language_processors.py`, with
adjacent helper behavior from `inference/LyricFA/tools/ZhG2p.py` and
`inference/LyricFA/tools/JaG2p.py`.

## Discovery

`language_processors.py` imports only standard-library modules and local helper
modules:

- `re`;
- `abc.ABC` and `abstractmethod`;
- `dataclasses.dataclass`;
- `typing.List`, `typing.Dict`, and `typing.Type`;
- `ZhG2p` and `split_string`;
- `JaG2p`.

The adjacent helper seams are already verified:

- `zh_g2p_dictionary_core` verifies Chinese dictionary conversion over
  injected maps;
- `ja_g2p_fallback_core` verifies Japanese fallback conversion with
  `pyopenjtalk` absent;
- `lyric_matching_file_contract_core` verifies file/state/JSON orchestration
  while keeping language processors separate.

Project dependency files include `pyopenjtalk` only under a Windows platform
marker. The current Linux stage should therefore use the verified
pyopenjtalk-absent Japanese fallback and keep OpenJTalk runtime ownership for a
separate promotion decision.

## Decision

Confirm the existing `lyric_language_processor_contract` unit boundary.

Use a narrow hand-written Rust implementation in a future
`v2m-core::lyric_language` module. That module should compose processor
selection and clean/split/phonetic orchestration with the already verified
`zh_g2p` and `ja_g2p` helper seams.

Do not add package-level language processing dependencies, PyO3, subprocess,
HTTP, CLI, runtime-router, OpenJTalk, model-runtime, GUI, or Web dependencies.

Keep these behaviors legacy-owned:

- bundled Chinese dictionary file IO and packaging;
- `pyopenjtalk.run_frontend` output parity and OpenJTalk runtime ownership;
- `LyricMatcher` file/state/JSON orchestration;
- sequence alignment internals;
- model execution;
- GUI/Web/CLI routing;
- production bridge wiring.

## Verification

Dependency/bootstrap artifacts:

```text
rewrite-in-rust/dependencies/lyric_language_processor_contract.yaml
rewrite-in-rust/bootstrap/lyric_language_processor_contract.md
rewrite-in-rust/fixtures/lyric_language_processor_contract.jsonl
rewrite-in-rust/bootstrap/check_lyric_language_processor_contract.py
```

Current Python parity command:

```bash
uv run python rewrite-in-rust/bootstrap/check_lyric_language_processor_contract.py
```

The fixture gate preserves two non-obvious legacy shapes:

- Python regex `\s` whitespace includes U+001C..U+001F in the clean-text
  paths;
- `ChineseProcessor.clean_text("a]b[c^d-e\\f中  文")` returns
  `"b[c^d-e\\f中 文"` because of the current bracket-adjacent regex pattern;
- `ChineseProcessor.get_phonetic_list([])` returns `[""]`.

Expected Rust command after writer implementation:

```bash
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml lyric_language
```

## Reversal

Rollback remains keeping `ProcessorFactory`, all Python `LanguageProcessor`
implementations, and `LyricData` in `inference.LyricFA.tools.language_processors`
as the runtime owners. No production bridge was introduced.
