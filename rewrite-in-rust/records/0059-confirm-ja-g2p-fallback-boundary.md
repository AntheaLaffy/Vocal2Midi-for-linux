# 0059 - Confirm Japanese G2P Fallback Boundary

Date: 2026-07-17

## Context

The next manifest unit after `zh_g2p_dictionary_core` is
`ja_g2p_fallback_core`. It was still `planned` and `provisional`, so
dependency/bootstrap discovery was required before writer work.

The selected source is `inference/LyricFA/tools/JaG2p.py`.

## Discovery

`JaG2p.py` imports `re` and optional `pyopenjtalk`. The project dependency
files include `pyopenjtalk` only behind a Windows platform marker. In the Linux
uv environment for this repository snapshot, the public path must tolerate
`pyopenjtalk = None`.

The fallback logic is deterministic string and table control flow over:

- ASCII letter and special-letter classification;
- Python `str.isdigit()` digit recognition plus `〇`;
- kanji, kana, small-kana, long-mark, and Japanese symbol ranges;
- input token splitting;
- Japanese segment splitting into contracted kana and long-vowel groups;
- hiragana/katakana conversion;
- `KATA_TO_ROMAJI` and long-vowel mora expansion;
- ASCII, full-width, and `〇` number mapping;
- fallback entries for Latin, kanji, unknown, and unsupported numeric tokens;
- public romaji and kana output helpers.

The current production callers use `JaG2p` from `language_processors.py` and
`inference/API/lfa_api.py`. These callers remain legacy-owned.

## Decision

Confirm the existing `ja_g2p_fallback_core` unit boundary, scoped to the
pyopenjtalk-absent fallback path.

Do not migrate `pyopenjtalk.run_frontend` or introduce OpenJTalk runtime
ownership in this unit. If frontend-backed Japanese G2P is promoted later, it
must be a separate bridge/promotion decision with explicit package ownership,
payload validation, error mapping, and rollback.

Do not add PyO3, subprocess, HTTP, CLI, runtime-router, language-processor, or
model-runtime dependencies.

## Verification

Dependency/bootstrap artifacts:

```text
rewrite-in-rust/dependencies/ja_g2p_fallback_core.yaml
rewrite-in-rust/bootstrap/ja_g2p_fallback_core.md
rewrite-in-rust/fixtures/ja_g2p_fallback_core.jsonl
rewrite-in-rust/bootstrap/check_ja_g2p_fallback_core.py
```

Current Python parity command:

```bash
uv run python rewrite-in-rust/bootstrap/check_ja_g2p_fallback_core.py
```

Expected Rust command after writer implementation:

```bash
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml ja_g2p
```

## Reversal

Rollback remains keeping `JaG2p` and `KATA_TO_ROMAJI` in
`inference.LyricFA.tools.JaG2p` as the runtime owners. No production bridge was
introduced.
