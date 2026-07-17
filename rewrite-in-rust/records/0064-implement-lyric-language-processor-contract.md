# 0064 - Implement Lyric Language Processor Contract

Date: 2026-07-17

## Context

`lyric_language_processor_contract` was confirmed by record 0063 as a
fixture-bound unit for deterministic language processor selection and
clean/split/phonetic orchestration from
`inference/LyricFA/tools/language_processors.py`.

The unit composes already verified helper seams:

- `zh_g2p_dictionary_core` for Chinese dictionary conversion over injected
  maps;
- `ja_g2p_fallback_core` for Japanese fallback conversion with `pyopenjtalk`
  absent.

## Decision

Implement the unit as `v2m-core::lyric_language` without production routing.

The Rust implementation preserves:

- supported language ordering and case-insensitive factory selection;
- exact unsupported-language message text;
- Python regex `\s` whitespace behavior in clean-text paths;
- English clean/split/identity-phonetic behavior;
- current Chinese bracket-adjacent regex cleaning behavior;
- Chinese split and injected-map phonetic conversion;
- the legacy Chinese empty-input phonetic shape of `[""]`;
- Japanese whitespace-only cleaning;
- Japanese kana display-token splitting;
- Japanese romaji phonetic conversion;
- Japanese `build_reference_lyric` tuple behavior.

## Verification

```bash
uv run python rewrite-in-rust/bootstrap/check_lyric_language_processor_contract.py
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml lyric_language
```

## Residual Risk

This is a no-bridge library seam. Runtime promotion must still define Chinese
dictionary asset payloads, OpenJTalk ownership or exclusion, Python-facing
error mapping, logging text, and rollback.

## Reversal

Rollback remains keeping all language processor runtime calls in Python. No
production bridge was introduced.
