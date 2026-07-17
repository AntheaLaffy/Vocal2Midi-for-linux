# 0057 - Confirm Chinese G2P Dictionary Boundary

Date: 2026-07-17

## Context

The next manifest unit after `lyric_sequence_alignment_core` was
`zh_g2p_dictionary_core`. It was still `planned` and `provisional`, so
dependency/bootstrap discovery was required before writer work.

The selected source is `inference/LyricFA/tools/ZhG2p.py`, with dictionary data
under `inference/LyricFA/Dicts/`.

## Discovery

`ZhG2p.py` imports only `os`. The conversion logic is deterministic string and
dictionary control flow over:

- tone-mark normalization;
- ASCII/special/kana/hanzi/digit token splitting;
- Mandarin/Cantonese dictionary maps;
- traditional-to-simplified lookup;
- phrase dictionary matching around polyphonic characters;
- ASCII digit conversion;
- include-tone and tone-stripping behavior;
- original-token reset for unknown, Latin, kana, and punctuation values.

The bundled dictionary directories are large text assets:

- `Dicts/mandarin`: `word.txt`, `phrases_dict.txt`, `phrases_map.txt`,
  `trans_word.txt`, `user_dict.txt`;
- `Dicts/cantonese`: the same dictionary shape, with an empty `user_dict.txt`.

The current production callers use Mandarin `ZhG2p` from
`language_processors.py` and `inference/API/lfa_api.py`. `split_string` is also
used by the English processor for legacy token splitting.

## Decision

Confirm the existing `zh_g2p_dictionary_core` unit boundary, with a narrow Rust
implementation over injected dictionary maps.

Do not bundle the full dictionary files into Rust production code in this unit.
Keep dictionary file IO and packaging legacy-owned until a promotion record
defines how those assets cross the Python/Rust boundary.

Do not add PyO3, subprocess, HTTP, CLI, runtime-router, language-processor, or
model-runtime dependencies.

## Verification

Dependency/bootstrap artifacts:

```text
rewrite-in-rust/dependencies/zh_g2p_dictionary_core.yaml
rewrite-in-rust/bootstrap/zh_g2p_dictionary_core.md
rewrite-in-rust/fixtures/zh_g2p_dictionary_core.jsonl
rewrite-in-rust/bootstrap/check_zh_g2p_dictionary_core.py
```

Current Python parity command:

```bash
uv run python rewrite-in-rust/bootstrap/check_zh_g2p_dictionary_core.py
```

Expected Rust command after writer implementation:

```bash
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml zh_g2p
```

## Reversal

Rollback remains keeping `ZhG2p`, `split_string`, and `tone_to_normal` in
`inference.LyricFA.tools.ZhG2p` as the runtime owners. No production bridge was
introduced.
