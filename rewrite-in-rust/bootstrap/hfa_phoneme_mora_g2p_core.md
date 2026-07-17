# hfa_phoneme_mora_g2p_core Bootstrap

## Boundary

Cover `BaseG2P.__call__`, `PhonemeG2P`, and
`JapanesePhonemeMoraG2P` from `inference/HubertFA/tools/g2p.py` as one pure
library unit. They share the exact `(ph_seq, word_seq, ph_idx_to_word_idx)`
shape, SP assertions, and language-prefix policy. Dictionary files, warnings,
dataset/lab IO, config, export, and model execution are excluded.

## Seam

- crate/module: `v2m-core::hfa_g2p`
- kind: independent Rust library
- runtime owner: legacy Python
- bridge dependencies: none
- input: `mode`, nullable `language`, and UTF-8 `input_text`
- output: three ordered arrays or an exact assertion failure

No third-party package is needed. Use the local tables and control-flow order as
the hand-written reference.

## Fixture Contract

`rewrite-in-rust/fixtures/hfa_phoneme_mora_g2p_core.jsonl` contains fields
`case_id`, `kind`, `language`, `text` or injected `output`, and `expect` with
either the three arrays or an exact legacy error projection. The legacy checker
imports the real classes and compares exact JSON:

```bash
uv run python rewrite-in-rust/bootstrap/check_hfa_phoneme_mora_g2p_core.py
```

Required cases: empty and repeated literal spaces, only/interior SP, AP/EP,
nullable/empty/string languages, N/cl/I/U, joined spellings, separate consonant
and vowel tokens, hu/fy/palatal forms, unknown/case variants, and repeated
calls. Preserve the observed empty-input distinction: phoneme mode produces one
empty word/phone, while mora mode produces only SP.

The table also binds Python 3.12's Unicode 15 lowercase contract. It covers the
four groups of 55 code points that newer Rust Unicode tables lowercase but
Python 3.12 leaves unchanged, retains contextual lowercase such as Greek final
sigma, and compares an MD5 over all 1,112,064 valid Unicode scalar mappings.

The Rust implementation is checked with:

```bash
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_phoneme_mora_g2p_core
```

The public Rust seam returns `HfaG2pOutput` and maps empty-phoneme access to
`IndexError` plus invalid SP layout to an empty-message `AssertionError`. It
also preserves Python 3.12 `str.strip()` handling for U+001C..U+001F and uses
literal ASCII-space splitting. The lowercase helper chunks around post-Unicode
15 mappings so ordinary contextual Unicode lowercase remains delegated to Rust.

## Rollback

Keep `BaseG2P`, `PhonemeG2P`, and `JapanesePhonemeMoraG2P` as production owners.
No caller imports Rust until a later promotion record defines payload and error
mapping.
