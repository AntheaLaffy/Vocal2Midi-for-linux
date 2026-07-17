# ja_g2p_fallback_core - behavior_reviewer

Date: 2026-07-17
Decision: pass

## Findings

No behavior-parity findings.

- Severity: none
- Location: rewrite-in-rust/rust/crates/v2m-core/src/ja_g2p.rs:1
- Issue: No blocking mismatch found for the scoped pyopenjtalk-absent fallback behavior.
- Evidence: The checker forces `pyopenjtalk = None` before constructing the legacy fixture outputs (`rewrite-in-rust/bootstrap/check_ja_g2p_fallback_core.py:21`). The Rust module states and implements only deterministic fallback behavior with Python remaining the runtime owner for OpenJTalk and production callers (`rewrite-in-rust/rust/crates/v2m-core/src/ja_g2p.rs:1`). The parity fixture exercises classification, whitespace normalization, token splitting, Japanese segment splitting, number mapping and non-mapping, kana contractions, long vowels, `cl`/`n` long-vowel skipping, uncommon kana fallback, Latin lowercase behavior, kanji fallback, and romaji-vs-kana public outputs (`rewrite-in-rust/fixtures/ja_g2p_fallback_core.jsonl:1`).
- Required fix: None.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_ja_g2p_fallback_core.py`: passed.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml ja_g2p`: passed; 1 ja_g2p test passed, 95 filtered in `v2m-core`, 0 tests in `v2m_quant_bridge`.
- `cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check`: passed.
- `git diff --check`: passed.
- `uv run python - <<'PY' ... compare Python str.isdigit() codepoints against Rust is_digit table ... PY`: passed; Python and Rust both covered 808 digit codepoints with 0 missing and 0 extra.

## Residual Risk

The unit intentionally excludes `pyopenjtalk.run_frontend` output parity and OpenJTalk runtime ownership. That exclusion is documented in the bootstrap boundary (`rewrite-in-rust/bootstrap/ja_g2p_fallback_core.md:26`) and boundary record (`rewrite-in-rust/records/0059-confirm-ja-g2p-fallback-boundary.md:41`). If a later promotion connects Japanese G2P to production callers, it still needs a separate record for OpenJTalk ownership, fallback ordering, payload validation, logging text, Python-facing error mapping, and rollback (`rewrite-in-rust/bootstrap/ja_g2p_fallback_core.md:128`).

The Rust crate remains an independent test surface and is not wired into the Python runtime (`rewrite-in-rust/rust/crates/v2m-core/src/lib.rs:1`). Production callers in `language_processors.py` and `lfa_api.py` remain legacy-owned as required by the boundary record (`rewrite-in-rust/records/0059-confirm-ja-g2p-fallback-boundary.md:33`).

## Promotion Note

This behavior review does not block promotion of the scoped fallback unit. Do not mark the manifest verified from this report alone; coordinator state updates should wait for the required review set and preserve the no-production-bridge constraint.
